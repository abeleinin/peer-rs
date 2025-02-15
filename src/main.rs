use env_logger;
use log::{info};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting WebRTC peer-to-peer audio connection...");
    
    Ok(())
}

