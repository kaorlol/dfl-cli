use std::{env, error::Error, fs::{File, create_dir_all}, io::{copy, BufWriter, Cursor}, time::Instant};
use reqwest::Client;

// Chatgpted :money_mouth:
fn create_link(text: &str, path: &str) -> String {
    format!("\x1B]8;;file://{path}\x07{text}\x1B]8;;\x07", text = text, path = path)
}

pub async fn download(url: &str, id: &str, dir: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err("Unable to access url".into());
    }

    create_dir_all(dir)?;

    let dest = File::create(format!("{}/{}.mp4", dir, id))?;
    let stream = response.bytes().await?.into_iter().collect::<Vec<_>>();
    let start = Instant::now();
    copy(&mut Cursor::new(stream), &mut BufWriter::new(dest))?;

    let duration = start.elapsed();
    println!("Downloaded in {:?}\nSaved to {}", duration, create_link(dir, env::current_dir()?.join(dir).to_str().unwrap()));

    Ok(())
}