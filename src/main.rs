use std::{env, error::Error};
use colored::*;

mod network;
mod download;

use network::{fetch, check_url};
use download::{download, setup_files};

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

    setup_files().await?;

    println!("{} {}", "Fetching:".blue(), url);

    let (url, title) = fetch(&url_type, id).await?;
    download(&url_type, &url, &title).await?;

    Ok(())
}