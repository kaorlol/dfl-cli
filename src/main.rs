use regex::Regex;
use std::env;

mod network;
mod download;

use network::fetch_clip_info;
use download::download_clip;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <clip url>", args[0]);
        return Ok(());
    }

    let clip_url = &args[1];
    if !is_valid_url(clip_url) {
        println!("Invalid clip url");
        return Ok(());
    }

    let clip_id = clip_url.split('/').last().unwrap();
    println!("Downloading clip twitch-clips/{}", clip_id);

    let clip_data = fetch_clip_info(clip_id).await?;
    let download_url = clip_data.as_str().unwrap();
    download_clip(download_url, clip_id).await?;

    Ok(())
}