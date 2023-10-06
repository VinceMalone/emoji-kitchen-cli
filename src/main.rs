use std::error::Error;
use std::path::Path;

use std::fmt::Display;

mod emoji;
mod utils;

#[tokio::main]
async fn main() {
    let emoji = emoji::init();
    download(emoji.pairs).await;
}

async fn download(pairs: Vec<emoji::EmojiPair>) {
    println!("{} pairs found", pairs.len());

    let mut errors: Vec<(Box<dyn Display>, Box<dyn Error>)> = Vec::new();

    for pair in pairs {
        let dest_path = Path::new("dist").join(&pair.filename);
        match utils::download_and_save_image(&pair.image_url, &dest_path).await {
            Err(err) => {
                println!("🚫 {} {} {}", pair.filename, pair.image_url, err);
                errors.push((
                    Box::new(format!("{} {}", pair.filename, pair.image_url)),
                    err,
                ));
            }
            Ok(_) => println!("✅ {} {}", pair.filename, pair.image_url),
        }
    }

    println!("ℹ️ Completed with {} errors", errors.len());

    for (message, error) in errors {
        println!("🚫 {} {}", message, error);
    }
}
