use env_logger;
use log::{info};
use anyhow::Result;

use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let api = APIBuilder::new().build();

    let config = RTCConfiguration {
        ..Default::default()
    };

    let peer_connection = api.new_peer_connection(config).await?;

    info!("Starting WebRTC peer-to-peer audio connection...");
    
    Ok(())
}
