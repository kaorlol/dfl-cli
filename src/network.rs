use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::Value;
use urlencoding::encode;
use std::error::Error;
use std::time::Instant;

pub async fn fetch_clip_info(clip_id: &str) -> Result<Value, Box<dyn Error>> {
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
        println!("Error: {}", res.status());
        return Ok("Unable to fetch clip info".into());
    }
    
    let body = res.text().await?;
    let json_response: Value = serde_json::from_str(&body)?;
    let clip_data = &json_response[0]["data"]["clip"];
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
