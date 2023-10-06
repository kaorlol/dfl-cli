use std::{error::Error, time::Instant};
use colored::*;
use crate::utils::*;

mod twitch {
    use super::Error;
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

    async fn send_gql_request(query: String) -> Result<Value, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let response = client.post("https://gql.twitch.tv/gql").header("Client-ID", "kimne78kx3ncx6brgo4mv6wki5h1ko").body(query).send().await?;
    
        if !response.status().is_success() {
            return Err("Unsuccessful response (GQL API)".into());
        }
    
        let json_response: Value = serde_json::from_str(&response.text().await?)?;
        Ok(json_response)
    }

    pub mod clip {
        use super::*;

        pub async fn fetch(id: &str) -> Result<(String, String), Box<dyn Error>> {
            let data = get_data(id).await?;
            let title = get_title(id).await?;
            let download_url = format!(
                "{}?sig={}&token={}",
                data["videoQualities"][0]["sourceURL"].as_str().unwrap_or_default(),
                data["playbackAccessToken"]["signature"].as_str().unwrap_or_default(),
                encode(data["playbackAccessToken"]["value"].as_str().unwrap_or_default())
            );

            Ok((download_url, title))
        }

        async fn get_data(id: &str) -> Result<Value, Box<dyn Error>> {
            let query = format!(r#"{{"operationName":"VideoAccessToken_Clip","variables":{{"slug":"{}"}},"extensions":{{"persistedQuery":{{"version":1,"sha256Hash":"36b89d2507fce29e5ca551df756d27c1cfe079e2609642b4390aa4c35796eb11"}}}}}}"#, id);
            let json_response = send_gql_request(query).await?;
            let data = &json_response["data"]["clip"];
            if data.is_null() {
                return Err("Clip not found".into());
            }

            Ok(data.clone())
        }

        async fn get_title(id: &str) -> Result<String, Box<dyn Error>> {
            let query = format!(r#"{{"query":"query{{clip(slug:\"{}\"){{title}}}}","variables":{{}}}}"#, id);
            let json_response = send_gql_request(query).await?;
            let title = remove_last_non_char(&json_response["data"]["clip"]["title"].to_string().replace("\"", ""));

            Ok(title)
        }
    }

    pub mod video {
        use super::*;

        pub async fn fetch(id: &str) -> Result<(String, String), Box<dyn Error>> {
            let client = reqwest::Client::new();
            let title = get_title(id).await?;
            let playlist_url = get_playlist_url(id, &client).await?;

            Ok((playlist_url, title))
        }

        async fn get_title(id: &str) -> Result<String, Box<dyn Error>> {
            let query = format!(r#"{{"query":"query{{video(id:\"{}\"){{title}}}}","variables":{{}}}}"#, id);
            let response = send_gql_request(query).await?;
            let title = remove_last_non_char(&response["data"]["video"]["title"].to_string().replace("\"", ""));

            Ok(title)
        }

        async fn get_token_and_sig(id: &str) -> Result<(String, String), Box<dyn Error>> {
            let query = format!(r#"{{"operationName":"PlaybackAccessToken_Template","query":"query PlaybackAccessToken_Template($vodID: ID!, $playerType: String!) {{  videoPlaybackAccessToken(id: $vodID, params: {{platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}}) @include(if: true) {{    value    signature    __typename  }}}}", "variables":{{"vodID":"{}","playerType":"embed"}}}}"#, id);
            let response = send_gql_request(query).await?;
            let token = response["data"]["videoPlaybackAccessToken"]["value"].as_str().unwrap_or_default();
            let signature = response["data"]["videoPlaybackAccessToken"]["signature"].as_str().unwrap_or_default();

            Ok((token.into(), signature.into()))
        }

        async fn get_playlist_url(id: &str, client: &reqwest::Client) -> Result<String, Box<dyn Error>> {
            let (token, signature) = get_token_and_sig(id).await?;
            let playlist_url = format!("http://usher.ttvnw.net/vod/{}?nauth={}&nauthsig={}&allow_source=true&player=twitchweb", id, token, signature);
            let playlist_response = client.get(&playlist_url).send().await?;
            if !playlist_response.status().is_success() {
                return Err("Unsuccessful response (Usher API)".into());
            }

            let playlist_body = playlist_response.text().await?;
            let playlist_url = get_highest_bandwidth_url(&playlist_body).await?;

            Ok(playlist_url)
        }

        async fn get_highest_bandwidth_url(playlist_body: &str) -> Result<String, Box<dyn Error>> {
            let re = Regex::new(r"#EXT-X-STREAM-INF:BANDWIDTH=(\d+),.*\n(.*)\n")?;
        
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
    }
}

mod youtube {
    use super::Error;
    use invidious::{*, hidden::AdaptiveFormat};

    pub async fn fetch(id: &str) -> Result<(String, String), Box<dyn Error>> {
        let client = ClientAsync::default();
        let video = client.video(&id, None).await?;
        let title = video.title;
        let url = get_highest_bitrate_url(&video.adaptive_formats);
    
        Ok((url, title))
    }

    fn get_highest_bitrate_url(formats: &Vec<AdaptiveFormat>) -> String {
        let mut highest_bitrate = 0;
        let mut url = String::new();
        for format in formats {
            let bitrate = format.bitrate.parse::<u64>().unwrap();
            if bitrate > highest_bitrate {
                highest_bitrate = bitrate;
                url = format.url.clone();
            }
        }
    
        return url;
    }
}

pub async fn fetch(type_: &str, id: &str) -> Result<(String, String), Box<dyn Error>> {
    let start = Instant::now();

    let result = match type_ {
        "twitch-video" => twitch::video::fetch(id).await,
        "twitch-clip" => twitch::clip::fetch(id).await,
        "youtube-video" => youtube::fetch(id).await,
        "youtube-short" => youtube::fetch(id).await,
        _ => Err("Invalid type".into()),
    };

    println!("{} {}", "Fetched URL in:".blue(), get_elapsed_time(start));

    return result;
} 