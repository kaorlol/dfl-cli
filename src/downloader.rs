use std::{env, error::Error, time::Instant, path::Path, fs::File, io::prelude::*, io::BufWriter};
use colored::*;
use crate::utils::*;
use url::Url;
use indicatif::ProgressBar;

pub struct SimpleDownloader;

impl SimpleDownloader {
    pub async fn download(&self, url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
        let mut response = reqwest::get(url).await?;

        let length = response.content_length().unwrap_or(0);
        let pb = create_progress_bar(length);
    
        let file = File::create(output_file)?;
        let mut output_file = BufWriter::new(file);
        while let Some(chunk) = response.chunk().await? {
            output_file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }
        output_file.flush()?;
    
        pb.finish_and_clear();
    
        Ok(())
    }
}

pub struct M3U8Downloader;

impl M3U8Downloader {
    pub async fn download(&self, url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
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

        let mut pb = create_progress_bar(ts_files.len() as u64);
        let mut output_file = BufWriter::new(File::create(output_file)?);
        for ts_file in ts_files {
            Self::download_ts_file(&ts_file.to_string(), &mut output_file, &mut pb).await?;
        }
        output_file.flush()?;

        pb.finish_and_clear();

        Ok(())
    }

    async fn download_ts_file(url: &str, output_file: &mut BufWriter<File>, pb: &mut ProgressBar) -> Result<(), Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        let body = response.bytes().await?;

        pb.inc(1);
        output_file.write_all(&body)?;

        Ok(())
    }
}

pub struct Downloader;

impl Downloader {
    pub async fn download(&self, type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
        println!("{} {}", format!("Downloading {}:", type_).blue(), title);

        let url_type: Vec<&str> = type_.split('-').collect();
        let title = remove_not_characters(title);
        let output = format!("{}\\{}s\\{}.mp4", url_type[0], url_type[1], title);
        let start = Instant::now();

        match type_ {
            "twitch-clip" => SimpleDownloader.download(url, Path::new(&output)).await?,
            "twitch-video" => M3U8Downloader.download(url, Path::new(&output)).await?,
            "youtube-video" | "youtube-short" => SimpleDownloader.download(url, Path::new(&output)).await?,
            _ => return Err("Invalid type".into())
        }

        println!("{} {}", format!("Downloaded {} in:", url_type[1]).blue(), get_elapsed_time(start));

        let path = env::current_dir()?.join(output);
        println!("{} {}", "Saved to:".blue(), format!("\x1B]8;;file://{}\x07{}\x1B]8;;\x07", path.display(), path.display()));

        Ok(())
    }
}

pub async fn download(type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
    let downloader = Downloader;
    downloader.download(type_, url, title).await?;

    Ok(())
}