use std::{time::Instant, error::Error, fs::create_dir_all};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

const DIRECTORIES: [&str; 5] = ["twitch\\clips", "twitch\\videos", "youtube\\videos", "youtube\\shorts", "tiktok\\videos"];
const TIMES: [u128; 3] = [1000, 60000, 3600000];

pub fn get_elapsed_time(time: Instant) -> String {
    let elapsed = time.elapsed().as_millis();
    let formatted_elapsed = match elapsed {
        0..=999 => format!("{}ms", elapsed),
        1000..=59999 => format!("{}s", elapsed / TIMES[0]),
        60000..=3599999 => format!("{}m {}s", elapsed / TIMES[1], (elapsed % TIMES[1]) / TIMES[0]),
        _ => format!("{}h {}m", elapsed / TIMES[2], (elapsed % TIMES[2]) / TIMES[1])
    };

    return formatted_elapsed;
}

pub fn remove_not_characters(text: &str) -> String {
    return text.chars().filter(|&c| c.is_alphanumeric() || c.is_whitespace()).collect();
}

pub async fn setup_files() -> Result<(), Box<dyn Error>> {
    for directory in DIRECTORIES.iter() {
        create_dir_all(directory)?;
    }

    Ok(())
}

pub fn create_progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{bar:40.blue}] ETA: {eta}")
        .expect("Failed to set progress bar style")
        .progress_chars("=> "));
    return pb;
}

pub fn get_type(url: &str) -> (bool, &str) {
    let regex_patterns = [
        (r"https://www.twitch.tv/[A-Za-z0-9]+/clip/[A-Za-z0-9]+(-[A-Za-z0-9]+)*", "twitch-clip"),
        (r"https://clips.twitch.tv/[A-Za-z0-9]+(-[A-Za-z0-9]+)*", "twitch-clip"),
        (r"https://www.twitch.tv/videos/[0-9]+", "twitch-video"),
        (r"https://www.youtube.com/watch\?v=[A-Za-z0-9_-]+", "youtube-video"),
        (r"https://www.youtube.com/shorts/[A-Za-z0-9_-]+", "youtube-short"),
        (r"https://www.tiktok.com/@[A-Za-z0-9_-]+/video/[0-9]+", "tiktok-video")
    ];

    for (pattern, r#type) in regex_patterns.iter() {
        let regex = Regex::new(pattern).unwrap();
        if regex.is_match(url) {
            return (true, r#type);
        }
    }

    return (false, "invalid");
}

pub async fn check_url(url: &str) -> Result<String, Box<dyn Error>> {
    let (is_valid, url_type) = get_type(url);
    if !is_valid {
        return Err("Url does not match regex pattern".into());
    }

    Ok(url_type.into())
}