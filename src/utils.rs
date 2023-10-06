use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::Duration;

use image::codecs::gif::{GifDecoder, GifEncoder};
use image::AnimationDecoder;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

pub fn mime_type_from_extension(ext: &str) -> Option<String> {
    Some(match ext.to_ascii_lowercase().as_str() {
        "gif" => "image/gif".to_owned(),
        "png" => "image/png".to_owned(),
        _ => return None,
    })
}

pub fn get_file_as_byte_vec(path: &Path) -> Vec<u8> {
    let mut file = File::open(&path).expect("no file found");
    let metadata = std::fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer).expect("buffer overflow");
    buffer
}

pub async fn download_and_save_image(url: &str, path: &Path) -> Result<File, Box<dyn Error>> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client.get(url).send().await?;

    match res.status() {
        status if status.is_success() => {
            let bytes = res.bytes().await?;
            let mut content = Cursor::new(bytes);
            let mut file = File::create(path)?;
            std::io::copy(&mut content, &mut file)?;
            Ok(file)
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

pub async fn download_and_resize_animation(
    url: &str,
    path: &Path,
    size: &u32,
) -> Result<(), Box<dyn Error>> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client.get(url).send().await?;
    match res.status() {
        status if status.is_success() => {
            let bytes = res.bytes().await?;
            let content = Cursor::new(bytes);
            let decoder = GifDecoder::new(content)?;
            let frames = decoder.into_frames().collect_frames()?;
            let mut resizes_frames = Vec::new();
            for frame in &frames {
                let resized = image::imageops::resize(
                    &frame.buffer().clone(),
                    size.to_owned(),
                    size.to_owned(),
                    image::imageops::FilterType::Nearest,
                );
                resizes_frames.push(image::Frame::from_parts(
                    resized,
                    0,
                    0,
                    image::Delay::from_saturating_duration(Duration::from_millis(30)),
                ));
            }
            let file = File::create(path)?;
            let mut encoder = GifEncoder::new(file);
            encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;
            encoder.encode_frames(resizes_frames.into_iter())?;
            Ok(())
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}
