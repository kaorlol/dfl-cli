mod sources;

use std::{
    env,
    error::Error,
    fs::create_dir_all,
    time::{
        Instant,
        Duration
    }
};
use sources::Source;

const DIRECTORIES: &[&str] = &[
    "downloads/twitch/clips", 
    "downloads/twitch/videos", 
    "downloads/youtube/videos", 
    "downloads/youtube/shorts"
];

fn setup_dir() -> Result<(), Box<dyn Error>> {
    for directory in DIRECTORIES.iter() {
        create_dir_all(directory)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_dir()?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!( "Usage: {} <url>", args[0] );

        return Ok(());
    }

    
    // parsing url instead of using regex to detect the 
    let source = match Source::parse( &args[1] ) {
        Ok(source) => source,
        Err(e) => {
            println!( "Failed to parse source url: {}", e );

            return Ok(());
        }
    };

    let start = Instant::now();
    let data  = source.fetch();

    Ok(())
}
