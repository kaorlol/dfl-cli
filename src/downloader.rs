use std::{env, error::Error, path::Path, fs::File, io::prelude::*, io::BufWriter, time::Instant};
use colored::*;
use url::Url;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::utils::*;

#[async_trait]
pub trait Downloader {
    async fn download(&self, url: &str, output_file: &Path) -> Result<(), Box<dyn Error>>;
    async fn download_ts_file(&self, url: &str, output_file: &mut BufWriter<File>, pb: &mut ProgressBar) -> Result<(), Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        let body = response.bytes().await?;

        pb.inc(1);
        output_file.write_all(&body)?;

        Ok(())
    }
}

pub struct SimpleDownloader;

#[async_trait]
impl Downloader for SimpleDownloader {
    async fn download(&self, url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
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

#[async_trait]
impl Downloader for M3U8Downloader {
    async fn download(&self, url: &str, output_file: &Path) -> Result<(), Box<dyn Error>> {
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
            Self::download_ts_file(self, &ts_file.to_string(), &mut output_file, &mut pb).await?;
        }
        output_file.flush()?;

        pb.finish_and_clear();

        Ok(())
    }
}

pub struct DownloadManager;

impl DownloadManager {
    pub fn get_downloader(type_: &str) -> Option<Box<dyn Downloader>> {
        match type_ {
            "twitch-clip" => Some(Box::new(SimpleDownloader)),
            "twitch-video" => Some(Box::new(M3U8Downloader)),
            "youtube-video" | "youtube-short" => Some(Box::new(SimpleDownloader)),
            _ => None,
        }
    }

    pub async fn download(&self, type_: &str, url: &str, title: &str) -> Result<(), Box<dyn Error>> {
        println!("{} {}", format!("Downloading {}:", type_).blue(), title);

        let url_type: Vec<&str> = type_.split('-').collect();
        let title = remove_not_characters(title);
        let output = format!("{}\\{}s\\{}.mp4", url_type[0], url_type[1], title);
        let start = Instant::now();

        // Get the appropriate downloader based on the type
        if let Some(downloader) = Self::get_downloader(type_) {
            downloader.download(url, Path::new(&output)).await?;
        } else {
            return Err("Invalid type".into());
        }

        println!("{} {}", format!("Downloaded {} in:", url_type[1]).blue(), get_elapsed_time(start));

        let path = env::current_dir()?.join(output);
        println!("{} {}", "Saved to:".blue(), format!("\x1B]8;;file://{}\x07{}\x1B]8;;\x07", path.display(), path.display()));

        Ok(())
    }
}