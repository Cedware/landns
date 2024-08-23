use anyhow::Context;
use bytes::BytesMut;
use log::info;
use tokio::net::UdpSocket;
use tokio::time::Interval;
use crate::sig::Signer;

pub async fn publish_host_name_periodically(hostname: &str, mut interval: Interval, port: u16,
                                            signer: &dyn Signer) -> anyhow::Result<()>
{
    let target = format!("255.255.255.255:{}", port);
    loop {
        interval.tick().await;
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .context("Failed to bind to udp socket")?;
        socket.set_broadcast(true).context("Failed to set broadcast")?;
        let data = signer.sign(hostname.as_bytes())
            .context("Failed to sign data")?;
        let mut data = BytesMut::from(data);
        data.extend_from_slice(b"\r\n");
        info!("sending host name: {} to: {}", hostname, target);
        let mut bytes_send = 0;
        while bytes_send < data.len() {
            bytes_send = socket.send_to(&data[bytes_send..], &target).await
                .context("Failed to send data")?;
        }
    }
}