use std::{env, error::Error, fs::create_dir_all, time::Instant, path::Path};
use colored::*;
use crate::elapsed::get_elapsed_time;

const DIRECTORIES: [&str; 4] = ["twitch\\clips", "twitch\\videos", "youtube\\videos", "youtube\\shorts"];

pub fn remove_not_characters(text: &str) -> String {
    return text.chars().filter(|&c| c.is_alphanumeric() || c.is_whitespace()).collect();
}

pub async fn setup_files() -> Result<(), Box<dyn Error>> {
    for directory in DIRECTORIES.iter() {
        create_dir_all(directory)?;
    }

    Ok(())
}

mod downloader {
    use std::{error::Error, fs::File, io::prelude::*, path::Path};
    use indicatif::{ProgressBar, ProgressStyle};
    use url::Url;

    fn create_progress_bar(length: u64) -> ProgressBar {
        let pb = ProgressBar::new(length);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{bar:40.blue}] ETA: {eta}")
            .expect("Failed to set progress bar style")
            .progress_chars("=> "));
        return pb;
    }

    pub async fn download_m3u8(url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
        let base_url = Url::parse(url)?;
        let response = reqwest::get(base_url.join(url)?).await?;
        let body = response.text().await?;
        let mut lines = body.lines();

        let mut ts_files = Vec::new();
        while let Some(line) = lines.next() {
            if line.starts_with("#EXT-X-STREAM-INF") {
                lines.next();
            } else if line.ends_with(".ts") {
                ts_files.push(base_url.join(line)?);
            }
        }

        let mut output_file = File::create(output_file)?;
        let pb = create_progress_bar(ts_files.len() as u64);

        for ts_file in ts_files {
            let ts_response = reqwest::get(ts_file).await?;
            let ts_body = ts_response.bytes().await?;
            pb.inc(1);
            output_file.write_all(&ts_body)?;
        }

        // let mut tasks = Vec::new();
        // for ts_file in ts_files {
        //     let mut output_file = output_file.try_clone()?;
        //     let pb = pb.clone();
        //     tasks.push(tokio::spawn(async move {
        //         let ts_response = reqwest::get(ts_file).await.unwrap();
        //         let ts_body = ts_response.bytes().await.unwrap();
        //         pb.inc(1);
        //         output_file.write_all(&ts_body).unwrap();
        //     }));
        // }

        // for task in tasks {
        //     task.await?;
        // }

        pb.finish_and_clear();

        Ok(())
    }

    pub async fn download_mp4(url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_file)?;
        let mut response = reqwest::get(url).await?;

        let length = response.content_length().unwrap_or(0);
        let pb = create_progress_bar(length);
    
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }
    
        pb.finish_and_clear();
    
        Ok(())
    }
}

pub async fn download(type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    println!("{} {}", format!("Downloading {}:", type_).blue(), title);

    let url_type: Vec<&str> = type_.split('-').collect();
    let title = remove_not_characters(title);
    let output = format!("{}\\{}s\\{}.mp4", url_type[0], url_type[1], title);
    let start = Instant::now();

    match type_ {
        "twitch-clip" => downloader::download_mp4(url, Path::new(&output)).await?,
        "twitch-video" => downloader::download_m3u8(url, Path::new(&output)).await?,
        "youtube-video" => downloader::download_mp4(url, Path::new(&output)).await?,
        "youtube-short" => downloader::download_mp4(url, Path::new(&output)).await?,
        _ => return Err("Invalid type".into())
    }

    println!("{} {}", format!("Downloaded {} in:", url_type[1]).blue(), get_elapsed_time(start));

    let path = env::current_dir()?.join(output);
    println!("{} {}", "Saved to:".blue(), format!("\x1B]8;;file://{}\x07{}\x1B]8;;\x07", path.display(), path.display()));

    Ok(())
}