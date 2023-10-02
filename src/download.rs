use std::{env, error::Error, fs::create_dir_all, time::Instant, process::Command};
use colored::*;

// Chatgpted :money_mouth:
fn create_link(text: &str, path: &str) -> String {
    // Creates a clickable link in the terminal (win 11 or vscode only)
    format!("\x1B]8;;file://{path}\x07{text}\x1B]8;;\x07", text = text, path = path)
}

fn remove_not_characters(text: &str) -> String {
    // Removes all non-alphanumeric characters from a string
    return text.chars().filter(|&c| c.is_alphanumeric() || c.is_whitespace()).collect();
}

pub async fn setup_files() -> Result<(), Box<dyn Error>> {
    // Create directories
    create_dir_all("twitch/clips")?;
    create_dir_all("twitch/videos")?;

    Ok(())
}

async fn download_clip(url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    // Run ffmpeg to download clip and convert to mp4
    Command::new("./ffmpeg").args(&["-i", url, "-c", "copy", "-bsf:a", "aac_adtstoasc", format!("twitch/clips/{}.mp4", remove_not_characters(title)).as_str()]).output()?;

    Ok(())
}

async fn download_video(url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    // Run ffmpeg to download video and convert to mp4
    Command::new("./ffmpeg").args(&["-i", url, "-c", "copy", "-bsf:a", "aac_adtstoasc", format!("twitch/videos/{}.mp4", remove_not_characters(title)).as_str()]).output()?;

    Ok(())
}

pub async fn download(type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    println!("{} {}", format!("Downloading {}:", type_).blue(), title);

    let start = Instant::now();

    match type_ {
        "clip" => download_clip(url, title).await?,
        "video" => download_video(url, title).await?,
        _ => println!("Invalid type"),
    };

    println!("{} {:?}ms", format!("Downloaded {} in:", type_).blue(), start.elapsed().as_millis());
    println!("{} {}", "Saved to:".blue(), create_link(&format!("twitch/{}s", type_), env::current_dir()?.join(format!("twitch/{}s/{}.mp4", type_, title)).to_str().unwrap()));
    Ok(())
}