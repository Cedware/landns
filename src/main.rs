use std::process::exit;
use std::time::Duration;
use anyhow::Context;
use log::{error, info};
use sha2::Sha256;
use tokio::join;
use tokio::time::{interval};
use crate::configuration::Configuration;
use crate::publish::publish_host_name_periodically;
use crate::sig::{HmacSigner, Signer, UnsecureSigner};

mod configuration;
mod publish;
mod receive;
mod read_host_name;
mod sig;


async fn create_signer(key_path: &Option<String>) -> anyhow::Result<Box<dyn Signer>> {
    match key_path {
        Some(key_path) => {
            let signer = HmacSigner::<Sha256>::new_from_key_file(&key_path)
                .await.context(format!("failed to initialize signer from key {}",key_path))?;
            Ok(Box::new(signer) as Box<dyn Signer>)
        },
        None => {
            Ok(Box::new(UnsecureSigner))
        }
    }
}

async fn start() -> anyhow::Result<()> {
    info!("loading configuration");
    let configuration = Configuration::from_env()?;

    let signera = create_signer(&Some("/home/cedrick/.landns/key".to_string())).await.context("Failed to create signer")?;
    let signerb = create_signer(&Some("/home/cedrick/.landns/key2".to_string())).await.context("Failed to create signer")?;
    let publish_future = publish_host_name_periodically(interval(Duration::from_secs(60)), configuration.port, &*signera);
    let receive_future = receive::receive_host_names(&configuration.local_address, configuration.port, &*signerb);
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


