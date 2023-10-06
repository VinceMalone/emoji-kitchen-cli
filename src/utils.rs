use std::error::Error;
use std::path::Path;

pub async fn download_image(url: &str) -> Result<image::DynamicImage, Box<dyn Error>> {
    let res = reqwest::get(url).await?;

    match res.status() {
        status if status.is_success() => {
            let bytes = res.bytes().await?;
            let image = image::load_from_memory(&bytes).unwrap();
            Ok(image)
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

pub async fn download_and_save_image(
    url: &str,
    path: &Path,
) -> Result<image::DynamicImage, Box<dyn Error>> {
    match download_image(url).await {
        Err(err) => Err(err),
        Ok(image) => {
            if let Err(err) = image.save(path) {
                return Err(Box::<dyn Error>::from(err));
            }
            Ok(image)
        }
    }
}
