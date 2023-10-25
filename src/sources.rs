use url::Url;
use std::error::Error;

pub mod youtube;

// using an enum instead of "youtube-short", "youtube-video" strings
pub enum Source {
    YoutubeVideo( Url ),
    YoutubeShort( Url ),

    TwitchVideo( Url ),
    TwitchClip( Url )
}

impl Source {

    // create Source enum from url
    pub fn parse( url: &str ) -> Result<Source, Box<dyn Error>> {
        let url = Url::parse( url )?;

        // load domain as &str
        let host = &*url.host_str()
            .map_or(
                Err("Failed to find host"),
                | host | Ok( host )
            )?
            .split(".") // all this bs is to remove www
            .filter( | f | f != &"www" )
            .collect::<Vec<_>>()
            .join(".");

        // split url path into sections
        // ie: example.com/a/b/c => [ "a", "b", "c" ]
        let path: Vec<&str> = url.path_segments()
            .map_or(
                Err("No path specified on url"),
                | path | Ok(path)
            )?
            .collect();

        // match url to specified enum type based on path segments and host
        let source = match host {
            "youtube.com" => {
                match path[0] {
                    "shorts" => Source::YoutubeShort( url ),
                    "watch" => Source::YoutubeVideo( url),

                    _ => return Err("Source not supported")?
                }
            },

            "twitch.tv" => {
                match path[0] {
                    "videos" => Source::TwitchVideo( url ),
                    
                    _ => match path[1] {
                        "clip" => Source::TwitchClip( url ),

                        _ => return Err("Source not supported")?
                    }
                }
            }

            _ => return Err("Source not supported")?
        };

        Ok( source )
    }

    pub async fn fetch( &self  ) {
        
    }

}