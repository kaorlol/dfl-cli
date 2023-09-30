use std::{env, error::Error};

mod network;
mod download;

use network::{fetch_clip_url, check_url};
use download::download_clip;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <url>", args[0]);
        return Ok(());
    }

    let url = &args[1];
    let url_type = check_url(url).await?;
    let id = url.split('/').last().unwrap();

    if url_type == "clip" {
        println!("Downloading clip {}", id);

        let clip_url = fetch_clip_url(id).await.and_then(|v| {
            Ok(v.as_str().unwrap().to_string())
        })?;
    
        download_clip(&clip_url, id).await?;
    }

    // if url_type == "video" {
    //     let (video_parts, title) = fetch_video_parts(id).await?;

    //     println!("Downloading {}", title);

    //     download_video(video_parts, title).await?;
    // }

    Ok(())
}