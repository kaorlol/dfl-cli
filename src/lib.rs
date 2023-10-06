pub mod elapsed;
pub mod network;
pub mod downloader;

pub use elapsed::get_elapsed_time;
pub use network::{fetch, check_url};
pub use downloader::{download, setup_files};