use std::{error::Error, time::Instant};
use regex::Regex;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde_json::Value;
use urlencoding::encode;

fn get_type(url: &str) -> (bool, &str) {
    let regex_patterns = [
        (r"https://clips.twitch.tv/[A-Za-z0-9]+(-[A-Za-z0-9]+)*", "clip"),
        (r"https://www.twitch.tv/videos/[0-9]+", "video"),
    ];

    for (pattern, r#type) in regex_patterns.iter() {
        let regex = Regex::new(pattern).unwrap();
        if regex.is_match(url) {
            return (true, r#type);
        }
    }

    return (false, "invalid");
}

pub async fn check_url(url: &str) -> Result<Value, Box<dyn Error>> {
    let (is_valid, url_type) = get_type(url);
    if !is_valid {
        return Err("Url does not match regex pattern".into());
    }

    Ok(url_type.into())
}

pub async fn fetch_clip_url(clip_id: &str) -> Result<Value, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("Client-ID", HeaderValue::from_static("kimne78kx3ncx6brgo4mv6wki5h1ko"));

    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    
    let start = Instant::now();
    let res = client.post("https://gql.twitch.tv/gql")
        .body(format!(
            r#"[{{"operationName":"VideoAccessToken_Clip","variables":{{"slug":"{}"}},"extensions":{{"persistedQuery":{{"version":1,"sha256Hash":"36b89d2507fce29e5ca551df756d27c1cfe079e2609642b4390aa4c35796eb11"}}}}}}]"#,
            clip_id
        ))
        .send()
        .await?;
    
    if !res.status().is_success() {
        return Err("Unsuccessful response (GQL API)".into());
    }
    
    let body = res.text().await?;
    let json_response: Value = serde_json::from_str(&body)?;
    let clip_data = &json_response[0]["data"]["clip"];
    if clip_data.is_null() {
        return Err("Clip not found".into());
    }

    let download_url = format!(
        "{}?sig={}&token={}",
        clip_data["videoQualities"][0]["sourceURL"].as_str().unwrap(),
        clip_data["playbackAccessToken"]["signature"].as_str().unwrap(),
        encode(clip_data["playbackAccessToken"]["value"].as_str().unwrap())
    );
    
    let duration = start.elapsed();
    println!("Got download url in {:?}", duration);

    Ok(download_url.into())
}