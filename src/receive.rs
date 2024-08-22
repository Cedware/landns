use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use anyhow::Context;
use bytes::{Bytes, BytesMut};
use log::info;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UdpSocket;
use crate::read_host_name::read_host_name;

const HOSTS_FILE: &str = "/etc/hosts";

fn get_host_name(buffer: &mut BytesMut) -> Option<anyhow::Result<String>> {
    buffer.windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| {
            let mut message = buffer.split_to(p + 2);
            message.truncate(message.len() - 2);
            String::from_utf8(message.to_vec())
                .context("Failed to convert message from utf8")
        })
}

async fn update_hosts(hostname: &str, ip: &IpAddr) -> anyhow::Result<()> {
    info!("updating hosts file");
    let own_host_name = read_host_name().await.context("Failed to read own host name")?;
    if own_host_name == hostname {
        info!("received own host name, skipping update");
        return Ok(());
    }
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

pub async fn receive_host_names(host: &str, port: u16) -> anyhow::Result<()> {
    let endpoint = format!("{}:{}", host, port);
    let socket = UdpSocket::bind(&endpoint).await
        .context(format!("Failed to create udp socket: {}", endpoint))?;
    let mut buffer_collection: HashMap<SocketAddr, BytesMut> = HashMap::new();
    loop {
        let mut buffer = [0; 1024];
        let (len, addr) = socket.recv_from(&mut buffer).await
            .context("Failed to receive data")?;
        let mut selected_buffer = buffer_collection.entry(addr).or_default();
        selected_buffer.extend_from_slice(&buffer[..len]);
        if let Some(host_name) = get_host_name(selected_buffer) {
            buffer_collection.remove(&addr);
            let host_name = host_name.context("Failed to extract host name from buffer")?;
            info!("received host name: {} from: {}", host_name, addr);
            update_hosts(&host_name, &addr.ip()).await
                .context("Failed to update hosts file")?;
        }
    }
}