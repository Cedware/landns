use std::fmt::format;
use anyhow::Context;
use bytes::BytesMut;
use log::info;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::UdpSocket;
use tokio::time::Interval;
use crate::read_host_name::read_host_name;

pub async fn publish_host_name_periodically(mut interval: Interval, port: u16) -> anyhow::Result<()> {

    let target = format!("255.255.255.255:{}", port);
    loop {
        let host_name = read_host_name().await.context("Failed to read hostname")?;
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .context("Failed to bind to udp socket")?;
        socket.set_broadcast(true).context("Failed to set broadcast")?;
        let mut data = BytesMut::from(host_name.as_bytes());
        data.extend_from_slice(b"\r\n");
        info!("sending host name: {} to: {}", host_name, target);
        let mut bytes_send = 0;
        while bytes_send < data.len() {
            bytes_send = socket.send_to(&data[bytes_send..], &target).await
                .context("Failed to send data")?;
        }
        interval.tick().await;
    }
}