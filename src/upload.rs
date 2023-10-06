use std::error::Error;
use std::fs;
use std::path::Path;

use reqwest::multipart;
use serde::Deserialize;

use crate::emoji;
use crate::utils;

fn get_emoji_data_from_dir(input_path: &Path, name_query: &Option<String>) -> Vec<UploadEmoji> {
    let paths = fs::read_dir(input_path).unwrap();
    let emoji_map = emoji::get_emoji_map();
    let mut output = Vec::new();

    for dir_result in paths {
        let path = dir_result.unwrap().path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        let mut split = filename.splitn(3, '.');
        split.next();
        let codepoint = split.next().unwrap().replace("_", "-");

        if !emoji_map.contains_key(&codepoint) {
            continue;
        }

        let emoji = emoji_map
            .get(&codepoint)
            .expect(&format!("⚠️ emoji data for {} not found!", &codepoint));

        match name_query {
            Some(n) if !n.eq(&emoji.short_name) => continue,
            Some(_) => {}
            None => {}
        }

        output.push(UploadEmoji {
            name: format!("{}_animated", emoji.short_name),
            path: path.to_str().unwrap().to_string(),
        });
    }

    output
}

pub async fn upload(input_path: &Path, name_query: &Option<String>) {
    let config = UploadConfig {
        cookie: dotenv!("COOKIE").to_owned(),
        token: dotenv!("TOKEN").to_owned(),
        workspace_name: dotenv!("WORKSPACE_NAME").to_owned(),
    };

    let emoji_list = get_emoji_data_from_dir(input_path, name_query);

    println!("ℹ️ {} emoji found", emoji_list.len());

    let mut errors: Vec<(String, Box<dyn Error>)> = Vec::new();

    for emoji in emoji_list {
        match upload_emoji(&config, &emoji).await {
            Err(err) => {
                println!("🚫 {} {} {}", &emoji.name, &emoji.path, err);
                errors.push((format!("{} {}", &emoji.name, &emoji.path), err));
            }
            Ok(_) => {
                println!("✅ {}", &emoji.name);
            }
        }
    }

    println!("ℹ️ Completed with {} errors", errors.len());

    for (message, error) in errors {
        println!("🚫 {} {}", message, error);
    }
}

struct UploadConfig {
    cookie: String,
    token: String,
    workspace_name: String,
}

#[derive(Debug)]
struct UploadEmoji {
    name: String,
    path: String,
}

impl UploadEmoji {
    fn mime_type(&self) -> String {
        let ext = Path::new(&self.path).extension().unwrap().to_str().unwrap();
        utils::mime_type_from_extension(ext).expect(&format!("⚠️ {} extension not supported", ext))
    }
}

#[derive(Deserialize, Debug)]
struct UploadResponseBody {
    ok: bool,
    #[serde(default)]
    error: String,
}

async fn upload_emoji(config: &UploadConfig, emoji: &UploadEmoji) -> Result<(), Box<dyn Error>> {
    let image_file = tokio::fs::File::open(&emoji.path).await?;

    let form = multipart::Form::new()
        .text("mode", "data")
        .text("name", emoji.name.to_owned())
        .text("token", config.token.to_owned())
        .part(
            "image",
            multipart::Part::stream(reqwest::Body::from(image_file))
                .file_name("image.gif") // just make up a name, it doesn't seem to matter
                .mime_str(&emoji.mime_type())?,
        );

    // "reqwest_retry" doesn't work with streaming requests, which multipart is
    let client = reqwest::Client::new();
    let url = format!("https://{}.slack.com/api/emoji.add", config.workspace_name);
    let res = client
        .post(url)
        .header("cookie", config.cookie.to_owned())
        .multipart(form)
        .send()
        .await?;

    match res.status() {
        status if status.is_success() => {
            let body = res.json::<UploadResponseBody>().await?;
            if body.ok {
                return Ok(());
            }
            Err(Box::<dyn Error>::from(body.error))
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}
