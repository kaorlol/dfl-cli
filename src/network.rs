use std::{error::Error, time::Instant};
use regex::Regex;
use colored::*;

fn get_type(url: &str) -> (bool, &str) {
    // Regex patterns
    let regex_patterns = [
        (r"https://clips.twitch.tv/[A-Za-z0-9]+(-[A-Za-z0-9]+)*", "twitch-clip"),
        (r"https://www.twitch.tv/videos/[0-9]+", "twitch-video"),
        (r"https://www.youtube.com/watch\?v=[A-Za-z0-9_-]+", "youtube-video"),
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

pub async fn check_url(url: &str) -> Result<String, Box<dyn Error>> {
    // Check if URL is valid
    let (is_valid, url_type) = get_type(url);
    if !is_valid {
        return Err("Url does not match regex pattern".into());
    }

    // Return URL type
    Ok(url_type.into())
}

mod twitch {
    use regex::Regex;
    use serde_json::Value;
    use urlencoding::encode;

    fn remove_last_non_char(text: &str) -> String {
        let mut chars = text.chars();
        let mut last_char = chars.next_back().unwrap();
        while !last_char.is_alphanumeric() || last_char.is_whitespace() {
            last_char = chars.next_back().unwrap();
        }
        return text.chars().take(text.len() - 1).collect();
    }

    async fn send_gql_request(query: String) -> Result<Value, Box<dyn std::error::Error>> {
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

    async fn get_highest_bandwidth_url(playlist_body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let re = Regex::new(r"#EXT-X-STREAM-INF:BANDWIDTH=(\d+),.*\n(.*)\n")?;
    
        // Find highest bandwidth URL
        let mut highest_bandwidth_url = String::new();
        let mut highest_bandwidth = 0;
    
        for cap in re.captures_iter(playlist_body) {
            let bandwidth: i32 = cap[1].parse()?;
            if bandwidth > highest_bandwidth {
                highest_bandwidth = bandwidth;
                highest_bandwidth_url = cap[2].to_string();
            }
        }
    
        Ok(highest_bandwidth_url)
    }

    pub async fn fetch_clip_url(id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
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
        let title = remove_last_non_char(&info_response_json["data"]["clip"]["title"].to_string().replace("\"", ""));
        let download_url = format!(
            "{}?sig={}&token={}",
            data["videoQualities"][0]["sourceURL"].as_str().unwrap_or_default(),
            data["playbackAccessToken"]["signature"].as_str().unwrap_or_default(),
            encode(data["playbackAccessToken"]["value"].as_str().unwrap_or_default())
        );

        Ok((download_url, title))
    }

    pub async fn fetch_video_url(id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Fetch video info
        let info_query = format!(r#"{{"query":"query{{video(id:\"{}\"){{title}}}}","variables":{{}}}}"#, id);
        let info_response = send_gql_request(info_query).await?;
        let title = remove_last_non_char(&info_response["data"]["video"]["title"].to_string().replace("\"", ""));

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
}

mod youtube {
    pub async fn fetch_video_url(id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        // Fetch video info
        let info_url = format!("https://www.youtube.com/get_video_info?video_id={}", id);
        let client = reqwest::Client::new();
        let info_response = client.get(&info_url).send().await?;
        if !info_response.status().is_success() {
            return Err("Unsuccessful response (Youtube API)".into());
        }

        // Parse video info
        let info_body = info_response.text().await?;
        println!("{}", info_body);

        Ok((String::new(), String::new()))
    }
}

pub async fn fetch(type_: &str, id: &str) -> Result<(String, String), Box<dyn Error>> {
    let start = Instant::now();

    let result = match type_ {
        "twitch-clip" => twitch::fetch_clip_url(id).await,
        "twitch-video" => twitch::fetch_video_url(id).await,
        "youtube-video" => youtube::fetch_video_url(id).await,
        _ => Err("Invalid type".into()),
    };

    println!("{} {}ms", "Fetched URL in:".blue(), start.elapsed().as_millis());

    return result;
}