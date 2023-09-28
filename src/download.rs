use reqwest::Client;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::{Cursor, BufWriter, copy};
use std::time::Instant;

pub async fn download_clip(url: &str, clip_id: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        println!("Error: {}", response.status());
        return Ok(());
    }

    create_dir_all("twitch-clips")?;

    let dest = File::create(format!("twitch-clips/{}.mp4", clip_id))?;
    let stream = response.bytes().await?.into_iter().collect::<Vec<_>>();
    let start = Instant::now();
    copy(&mut Cursor::new(stream), &mut BufWriter::new(dest))?;
    let duration = start.elapsed();
    println!("Downloaded in {:?}", duration);

    Ok(())
}