mod signal;

use anyhow::Result;
use clap::{AppSettings, Arg, Command};
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

#[tokio::main]
async fn main() -> Result<()> {
    // ────────────────────────── CLI ─────────────────────────────────────────
    let app = Command::new("peer-rs")
        .version("0.1.0")
        .author("Abe Leininger")
        .about("peer-to-peer")
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Enable verbose logging"),
        )
        .arg(
            Arg::new("port")
                .takes_value(true)
                .default_value("8080")
                .long("port")
                .help("HTTP signalling port"),
        );
    let matches = app.get_matches();

    if matches.is_present("debug") {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .init();
    }

    let port = matches.value_of("port").unwrap().parse::<u16>()?;

    // ───────────────────── Signalling (HTTP copy‑paste) ─────────────────────
    let mut sdp_rx = signal::http_sdp_server(port).await;
    println!("Paste browser SDP offer into http://localhost:{port}");

    // Wait for remote offer (browser)
    let offer_b64 = sdp_rx.recv().await.expect("no SDP received");
    let offer_json = signal::decode(&offer_b64)?;
    let offer: RTCSessionDescription = serde_json::from_str(&offer_json)?;

    // ─────────────────── WebRTC Peer‑Connection setup ───────────────────────
    let mut m = MediaEngine::default();
    m.register_default_codecs()?; // includes Opus

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".into()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let pc = Arc::new(api.new_peer_connection(config).await?);

    pc.on_peer_connection_state_change(Box::new(|st| {
        println!("PC state: {st}");
        Box::pin(async {})
    }));

    pc.set_remote_description(offer).await?;
    let answer = pc.create_answer(None).await?;
    let mut gather = pc.gathering_complete_promise().await;
    pc.set_local_description(answer).await?;
    gather.recv().await;

    if let Some(ld) = pc.local_description().await {
        println!("\n=== COPY the following base64 and paste into the *remote* page ===\n");
        let js = serde_json::to_string(&ld)?;
        println!("{}", signal::encode(&js));
    }

    futures::future::pending::<()>().await;

    Ok(())
}
