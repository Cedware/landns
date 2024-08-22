use anyhow::Context;
use log::info;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const HOST_NAME_FILE: &str = "/etc/hostname";
pub async fn read_host_name() -> anyhow::Result<String> {
    info!("reading host name");
    let mut file = File::open(HOST_NAME_FILE).await
        .context(format!("Failed to open file: {}", HOST_NAME_FILE))?;
    let mut host_name = String::new();
    file.read_to_string(&mut host_name).await
        .context(format!("Failed to read file: {}", HOST_NAME_FILE))?;
    let host_name = host_name.trim_end().to_string();
    info!("host name is: {}", host_name);
    Ok(host_name)
}
