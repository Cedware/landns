use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use anyhow::Context;
use bytes::{BytesMut};
use log::{error, info};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UdpSocket;
use crate::sig::Signer;

const HOSTS_FILE: &str = "/etc/hosts";

fn get_host_name(buffer: &mut BytesMut, signer: &dyn Signer) -> Option<anyhow::Result<String>> {
    buffer.windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| {
            let mut message = buffer.split_to(p + 2);
            message.truncate(message.len() - 2);
            let payload = signer.verify(&message)
                .context("Failed to verify message signature")?;
            String::from_utf8(payload.into())
                .context("Failed to convert message from utf8")
        })
}

async fn update_hosts(hostname: &str, ip: &IpAddr) -> anyhow::Result<()> {
    info!("updating hosts file");
    let hosts_file = File::open(HOSTS_FILE).await
        .context(format!("Failed to open hosts file: {}", HOSTS_FILE))?;
    let reader = BufReader::new(hosts_file);
    let mut lines = reader.lines();

    let mut updated_content = String::new();
    let mut modified_lines: bool = false;

    while let Some(line) = lines.next_line().await.context("Failed to read line")? {
        if line.contains(&hostname) {
            updated_content.push_str(&format!("{} {}\n", ip, hostname));
            modified_lines = true;
        } else {
            updated_content.push_str(&format!("{}\n", line));
        }
    }

    if !modified_lines {
        updated_content.push_str(&format!("{} {}", ip, hostname));
    }

    let mut hosts_file = File::create(HOSTS_FILE).await
        .context(format!("Failed to create hosts file: {}", HOSTS_FILE))?;


    hosts_file.write_all(updated_content.as_bytes()).await
        .context(format!("Failed to write to hosts file: {}", HOSTS_FILE))?;
    info!("updated hosts file");
    Ok(())
}

pub async fn receive_host_names(own_hostname: &str, local_addr: &IpAddr, port: u16, signer: &dyn Signer) -> anyhow::Result<()>
{
    let endpoint = format!("{}:{}", local_addr, port);
    let socket = UdpSocket::bind(&endpoint).await
        .context(format!("Failed to create udp socket: {}", endpoint))?;
    let mut buffer_collection: HashMap<SocketAddr, BytesMut> = HashMap::new();
    loop {
        let mut buffer = [0; 1024];
        let (len, addr) = socket.recv_from(&mut buffer).await
            .context("Failed to receive data")?;
        let selected_buffer = buffer_collection.entry(addr).or_default();
        selected_buffer.extend_from_slice(&buffer[..len]);
        if let Some(hostname) = get_host_name(selected_buffer, signer) {
            buffer_collection.remove(&addr);
            let hostname = match hostname {
                Err(e) => {
                    error!("failed to get hostname: {}", e);
                    continue;
                },
                Ok(hostname) => hostname
            };
            info!("received host name: {} from: {}", hostname, addr);
            if own_hostname == hostname {
                info!("received own host name, skipping update");
            }
            else {
                update_hosts(&hostname, &addr.ip()).await
                    .context("Failed to update hosts file")?;
            }
        }
    }
}