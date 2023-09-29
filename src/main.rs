use std::env;

mod network;
mod download;

use network::{fetch_clip_url, check_url};
use download::download;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <url>", args[0]);
        return Ok(());
    }

    let url = &args[1];
    let url_type = check_url(url).await?;

    if url_type == "clip" {
        let clip_id = url.split('/').last().unwrap();
        println!("Downloading clip {}", clip_id);

        let clip_url = fetch_clip_url(clip_id).await.and_then(|v| {
            Ok(v.as_str().unwrap().to_string())
        })?;
    
        download(&clip_url, clip_id, "twitch-clips").await?;
    }

    Ok(())
}