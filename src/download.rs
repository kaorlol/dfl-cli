// use std::{env, error::Error, fs::{File, create_dir_all}, time::Instant, process::Command, io::Write};
use std::{env, error::Error, fs::{create_dir_all, File, OpenOptions}, time::Instant};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use std::io::Write;

async fn download_m3u8(url: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Download the m3u8 file
    let base_url = url::Url::parse(url)?;
    let response = reqwest::get(base_url.join(url)?).await?;
    let body = response.text().await?;
    let mut lines = body.lines();

    // Parse the m3u8 file
    let mut ts_files = Vec::new();
    while let Some(line) = lines.next() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            // Skip the next line, which contains the URL of the variant playlist
            lines.next();
        } else if line.ends_with(".ts") {
            // Add the ts file to the list
            ts_files.push(base_url.join(line)?);
        }
    }

    // Download and combine the ts files
    let mut output_file = OpenOptions::new().write(true).create(true).truncate(true).open(output_file)?;
    let pb = ProgressBar::new(ts_files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.blue}] {bytes}/{total_bytes} ({eta})")
        .expect("Failed to set progress bar style")
        .progress_chars("=> "));

    for ts_file in ts_files {
        let ts_response = reqwest::get(ts_file).await?;
        let ts_body = ts_response.bytes().await?;
        pb.inc(1);
        output_file.write_all(&ts_body)?;
    }

    pb.finish_and_clear();

    Ok(())
}

const DIRECTORIES: [&str; 4] = ["twitch\\clips", "twitch\\videos", "youtube\\videos", "youtube\\shorts"];

// Removes all non-alphanumeric characters from a string
fn remove_not_characters(text: &str) -> String {
    return text.chars().filter(|&c| c.is_alphanumeric() || c.is_whitespace()).collect();
}

// Create required directories
pub async fn setup_files() -> Result<(), Box<dyn Error>> {
    // Create directories
    for directory in DIRECTORIES.iter() {
        create_dir_all(directory)?;
    }

    Ok(())
}

// Downloads the type of video from the url using ffmpeg
pub async fn download(type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    println!("{} {}", format!("Downloading {}:", type_).blue(), title);

    let title = remove_not_characters(title);
    let output = format!("twitch\\{}s\\{}.mp4", type_, title);
    let start = Instant::now();

    if type_ == "video" {
        download_m3u8(url, &output).await?;
    } else {
        let mut file = File::create(&output)?;
        let mut response = reqwest::get(url).await?;
        let length = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(length);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Failed to set progress bar style")
            .progress_chars("=> "));

        while let Some(chunk) = response.chunk().await? {
            pb.inc(chunk.len() as u64);
            std::io::Write::write_all(&mut file, &chunk)?;
        }

        pb.finish_and_clear();
    }

    println!("{} {:?}ms", format!("Downloaded {} in:", type_).blue(), start.elapsed().as_millis());

    let path = env::current_dir()?.join(output);
    println!("{} {}", "Saved to:".blue(), format!("\x1B]8;;file://{}\x07{}\x1B]8;;\x07", path.display(), path.display()));

    Ok(())
}