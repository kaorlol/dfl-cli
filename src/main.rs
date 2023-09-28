use regex::Regex;
use std::env;

mod network;
mod download;

use network::fetch_clip_url;
use download::download;

const VAILD_REGEX: [&str; 2] = [
    r"https://clips.twitch.tv/[A-Za-z0-9]+(-[A-Za-z0-9]+)*",
    r"https://www.twitch.tv/videos/[0-9]+"
];

fn is_valid_url(url: &str) -> bool {
    for regex in VAILD_REGEX.iter() {
        return Regex::new(regex).unwrap().is_match(url);
    }
    
    return false;
}

fn is_clip(url: &str) -> bool {
    return Regex::new(VAILD_REGEX[0]).unwrap().is_match(url);
}

// fn is_video(url: &str) -> bool {
//     return Regex::new(VAILD_REGEX[1]).unwrap().is_match(url);
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <url>", args[0]);
        return Ok(());
    }

    let url = &args[1];
    if !is_valid_url(url) {
        println!("Invalid url");
        return Ok(());
    }

    if is_clip(url) {
        let clip_id = url.split('/').last().unwrap();
        println!("Downloading clip {}", clip_id);

        let clip_url = fetch_clip_url(clip_id).await.and_then(|v| {
            Ok(v.as_str().unwrap().to_string())
        })?;
    
        download(&clip_url, clip_id, "twitch-clips").await?;
    }

    Ok(())
}