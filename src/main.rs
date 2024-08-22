use std::process::exit;
use std::time::Duration;
use log::{error, info};
use tokio::join;
use tokio::time::{interval, Interval};
use crate::configuration::Configuration;
use crate::publish::publish_host_name_periodically;

mod configuration;
mod publish;
mod receive;
mod read_host_name;

async fn start() -> anyhow::Result<()> {
    info!("loading configuration");
    let configuration = Configuration::from_env()?;
    let publish_future = publish_host_name_periodically(interval(Duration::from_secs(60)), configuration.port);
    let receive_future = receive::receive_host_names(&configuration.listen_address, configuration.port);
    let (publish_result, receive_result) = join!(publish_future, receive_future);
    receive_result?;
    publish_result?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    match start().await {
        Ok(_) => info!("Shutting down"),
        Err(e) => {
            error!("{:?}", e);
            exit(1);
        },
    }
}


