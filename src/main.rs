mod signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = 8080u16;
    let mut inbox = signal::http_sdp_server(port).await;

    println!("Server up: POST base64 blobs to http://localhost:{port}");

    while let Some(msg) = inbox.recv().await {
        println!("received: {msg}");
    }
    Ok(())
}
