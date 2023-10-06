use std::error::Error;
use std::fs;
use std::path::Path;

use futures::StreamExt;
use serde::Deserialize;

use crate::utils;

#[derive(Deserialize, Debug)]
struct AnimatedEmoji {
    // name: String,
    codepoint: String,
    // categories: Vec<String>, // ["Smileys and emotions"]
    tags: Vec<String>, // [":smile:"]
}

impl AnimatedEmoji {
    const IMAGE_FORMAT: &str = "gif";

    fn short_name(&self) -> String {
        match self.tags.first() {
            None => self.codepoint.to_string(),
            Some(name) => {
                let mut chars = name.chars();
                chars.next();
                chars.next_back();
                chars.as_str().to_string()
            }
        }
    }

    fn filename(&self) -> String {
        format!(
            "{}.{}.{}",
            self.short_name(),
            self.codepoint,
            AnimatedEmoji::IMAGE_FORMAT
        )
    }

    fn image_url(&self) -> String {
        format!(
            "https://fonts.gstatic.com/s/e/notoemoji/latest/{}/512.{}",
            self.codepoint,
            AnimatedEmoji::IMAGE_FORMAT
        )
    }
}

#[derive(Deserialize, Debug)]
struct AnimationsBody {
    #[serde(rename(deserialize = "icons"))]
    emoji: Vec<AnimatedEmoji>,
}

async fn fetch_animations() -> Result<AnimationsBody, Box<dyn Error>> {
    let res =
        reqwest::get("https://googlefonts.github.io/noto-emoji-animation/data/api.json").await?;

    match res.status() {
        status if status.is_success() => {
            let body = res.json::<AnimationsBody>().await?;
            Ok(body)
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

pub async fn animations(output_path: &Path, name: &Option<String>, size: &u32) {
    match fetch_animations().await {
        Err(err) => println!("Failed to fetch animations: {}", err),
        Ok(animations) => {
            let emoji = match name {
                None => animations.emoji,
                Some(name) => animations
                    .emoji
                    .into_iter()
                    .filter(|emoji| emoji.short_name().eq(name))
                    .collect(),
            };
            download(emoji, output_path, size).await;
        }
    }
}

async fn download(animated_emoji: Vec<AnimatedEmoji>, output_path: &Path, size: &u32) {
    fs::create_dir_all(&output_path).expect(&format!(
        "could not create output dir: {}",
        output_path.display()
    ));

    let download_iter = animated_emoji.into_iter().map(|emoji| async move {
        let image_url = emoji.image_url();
        let filename = emoji.filename();
        let dest_path = &output_path.join(&filename);

        match utils::download_and_resize_animation(&image_url, &dest_path, size).await {
            Err(err) => {
                println!("🚫 {} {} {}", filename, image_url, err);
                Ok(Some((format!("{} {}", filename, image_url), err)))
            }
            Ok(_) => {
                println!("✅ {} {}", filename, image_url);
                Ok(None)
            }
        }
    });

    let downloads = futures::stream::iter(download_iter)
        .buffer_unordered(8)
        .collect::<Vec<Result<Option<(String, Box<dyn Error>)>, ()>>>();

    let errors = downloads.await;

    for error in errors {
        if let Ok(result) = error {
            if let Some((message, err)) = result {
                println!("🚫 {} {}", message, err);
            }
        }
    }

    // let mut errors: Vec<AnError> = Vec::new();

    // for emoji in &animated_emoji {
    //     let image_url = emoji.image_url();
    //     let filename = emoji.filename();
    //     let dest_path = Path::new("dist_animated").join(&filename);

    //     match utils::download_and_resize_animation(&image_url, &dest_path).await {
    //         Err(err) => {
    //             println!("🚫 {} {} {}", filename, image_url, err);
    //             errors.push((Box::new(format!("{} {}", filename, image_url)), err));
    //         }
    //         Ok(_) => println!("✅ {} {}", filename, image_url),
    //     }
    // }

    // println!("ℹ️ Completed with {} errors", errors.len());

    // for (message, error) in errors {
    //     println!("🚫 {} {}", message, error);
    // }
}
