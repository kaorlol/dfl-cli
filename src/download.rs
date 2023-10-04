// use std::{env, error::Error, fs::{File, create_dir_all}, time::Instant, process::Command, io::Write};
use std::{env, error::Error, fs::create_dir_all, time::Instant, process::Command};
use colored::*;
// extern crate ffmpeg_next as ffmpeg;

// Download clip with ffmpeg
// ffmpeg::init().unwrap();
// let stream = ffmpeg::format::input(&url).unwrap().streams().best(ffmpeg::media::Type::Video).unwrap();

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
    Command::new("./ffmpeg")
        .args(&["-i", url, "-codec", "copy", &output])
        .output()?;

    // ffmpeg::init().unwrap();
    // let mut stream = ffmpeg::format::input(&url)?.streams().best(ffmpeg::media::Type::Video).unwrap();

    println!("{} {:?}ms", format!("Downloaded {} in:", type_).blue(), start.elapsed().as_millis());

    let path = env::current_dir()?.join(output);
    println!("{} {}", "Saved to:".blue(), format!("\x1B]8;;file://{}\x07{}\x1B]8;;\x07", path.display(), path.display()));
    Ok(())
}