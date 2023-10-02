use std::{error::Error, time::Instant};
use regex::Regex;
use serde_json::Value;
use urlencoding::encode;
use colored::*;

fn get_type(url: &str) -> (bool, &str) {
    // Regex patterns
    let regex_patterns = [
        (r"https://clips.twitch.tv/[A-Za-z0-9]+(-[A-Za-z0-9]+)*", "clip"),
        (r"https://www.twitch.tv/videos/[0-9]+", "video"),
    ];

    // Check if URL matches any regex patterns
    for (pattern, r#type) in regex_patterns.iter() {
        let regex = Regex::new(pattern).unwrap();
        if regex.is_match(url) {
            return (true, r#type);
        }
    }

    return (false, "invalid");
}

async fn get_highest_bandwidth_url(playlist_body: &str) -> Result<String, Box<dyn Error>> {
    // Regex pattern
    let re = Regex::new(r"#EXT-X-STREAM-INF:BANDWIDTH=(\d+),.*\n(.*)\n")?;

    // Find highest bandwidth URL
    let mut highest_bandwidth = 0;
    let mut highest_bandwidth_url = String::new();

    for cap in re.captures_iter(&playlist_body) {
        let bandwidth: i32 = cap[1].parse().unwrap();
        if bandwidth > highest_bandwidth {
            highest_bandwidth = bandwidth;
            highest_bandwidth_url = cap[2].to_string();
        }
    }

    Ok(highest_bandwidth_url)
}

async fn send_gql_request(query: String) -> Result<Value, Box<dyn Error>> {
    // Send GQL request
    let client = reqwest::Client::new();
    let response = client.post("https://gql.twitch.tv/gql").header("Client-ID", "kimne78kx3ncx6brgo4mv6wki5h1ko").body(query).send().await?;

    // Check if response is successful
    if !response.status().is_success() {
        return Err("Unsuccessful response (GQL API)".into());
    }

    // Parse response
    let json_response: Value = serde_json::from_str(&response.text().await?)?;
    Ok(json_response)
}

pub async fn check_url(url: &str) -> Result<String, Box<dyn Error>> {
    // Check if URL is valid
    let (is_valid, url_type) = get_type(url);
    if !is_valid {
        return Err("Url does not match regex pattern".into());
    }

    // Return URL type
    Ok(url_type.into())
}

async fn fetch_clip_url(id: &str) -> Result<(String, String), Box<dyn Error>> {
    // Fetch clip info
    let query = format!(r#"{{"operationName":"VideoAccessToken_Clip","variables":{{"slug":"{}"}},"extensions":{{"persistedQuery":{{"version":1,"sha256Hash":"36b89d2507fce29e5ca551df756d27c1cfe079e2609642b4390aa4c35796eb11"}}}}}}"#, id);
    let json_response = send_gql_request(query).await?;
    let data = &json_response["data"]["clip"];
    if data.is_null() {
        return Err("Clip not found".into());
    }

    // Fetch clip download URL
    let info_query = format!(r#"{{"query":"query{{clip(slug:\"{}\"){{title}}}}","variables":{{}}}}"#, id);
    let info_response_json = send_gql_request(info_query).await?;
    let title = info_response_json["data"]["clip"]["title"].to_string().replace("\"", "");
    let download_url = format!(
        "{}?sig={}&token={}",
        data["videoQualities"][0]["sourceURL"].as_str().unwrap_or_default(),
        data["playbackAccessToken"]["signature"].as_str().unwrap_or_default(),
        encode(data["playbackAccessToken"]["value"].as_str().unwrap_or_default())
    );

    Ok((download_url, title))
}

async fn fetch_video_url(id: &str) -> Result<(String, String), Box<dyn Error>> {
    let client = reqwest::Client::new();

    // Fetch video info
    let info_query = format!(r#"{{"query":"query{{video(id:\"{}\"){{title}}}}","variables":{{}}}}"#, id);
    let info_response = send_gql_request(info_query).await?;
    let title = info_response["data"]["video"]["title"].to_string().replace("\"", "");

    // Fetch video playback access token
    let token_query = format!(r#"{{"operationName":"PlaybackAccessToken_Template","query":"query PlaybackAccessToken_Template($vodID: ID!, $playerType: String!) {{  videoPlaybackAccessToken(id: $vodID, params: {{platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}}) @include(if: true) {{    value    signature    __typename  }}}}", "variables":{{"vodID":"{}","playerType":"embed"}}}}"#, id);
    let token_response = send_gql_request(token_query).await?;
    let token = token_response["data"]["videoPlaybackAccessToken"]["value"].as_str().unwrap_or_default();
    let signature = token_response["data"]["videoPlaybackAccessToken"]["signature"].as_str().unwrap_or_default();

    // Fetch video playlist
    let playlist_url = format!("http://usher.ttvnw.net/vod/{}?nauth={}&nauthsig={}&allow_source=true&player=twitchweb", id, token, signature);
    let playlist_response = client.get(&playlist_url).send().await?;
    if !playlist_response.status().is_success() {
        return Err("Unsuccessful response (Usher API)".into());
    }

    // Fetch highest bandwidth URL
    let playlist_body = playlist_response.text().await?;
    let playlist_url = get_highest_bandwidth_url(&playlist_body).await?;

    Ok((playlist_url, title))
}

pub async fn fetch(type_: &str, id: &str) -> Result<(String, String), Box<dyn Error>> {
    let start = Instant::now();

    let result = match type_ {
        "clip" => fetch_clip_url(id).await,
        "video" => fetch_video_url(id).await,
        _ => Err("Invalid type".into()),
    };

    println!("{} {}ms", "Fetched URL in:".blue(), start.elapsed().as_millis());

    return result;
}