use std::{env, error::Error};
use regex::Regex;
use colored::*;

mod elapsed;
mod network;
mod downloader;

pub use elapsed::get_elapsed_time;
use crate::network::{fetch, check_url};
use crate::downloader::{download, setup_files};

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

    // Get Youtube id
    let watch_regex = Regex::new(r"watch\?v=([A-Za-z0-9_-]+)")?;
    let id = match url_type.as_str() {
        "youtube-video" => watch_regex.captures(id).unwrap()[1].to_string(),
        _ => id.to_string(),
    };

    setup_files().await?;

    println!("{} {}", "Fetching:".blue(), url);

    
    let (url, title) = fetch(&url_type, &id).await?;
    download(&url_type, &url, &title).await?;

    Ok(())
}