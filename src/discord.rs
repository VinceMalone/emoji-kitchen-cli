use std::error::Error;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Deserialize, Debug)]
pub struct EmojiUser {
    pub username: String,
    pub id: String,
    // "discriminator": "0002",
    // "avatar": "5500909a3274e1812beb4e8de6631111",
    // "public_flags": 131328,
}

#[derive(Deserialize, Debug)]
pub struct Emoji {
    pub id: String,
    pub name: String,
    pub roles: Vec<String>, // ["41771983429993000", "41771983429993111"]
    pub user: EmojiUser,
    pub require_colons: bool,
    pub managed: bool,
    pub animated: bool,
}

#[derive(Serialize, Debug)]
pub struct CreateEmoji {
    pub name: String,
    // data URI scheme (128x128) (<= 256 KB)
    pub image: String,
    pub roles: Vec<String>,
}

fn get_emoji_endpoint(guild_id: &str) -> PathBuf {
    PathBuf::from(format!(
        "https://discord.com/api/v10/guilds/{}/emojis",
        guild_id
    ))
}

async fn fetch_emoji() -> Result<Vec<Emoji>, Box<dyn Error>> {
    let endpoint = get_emoji_endpoint(dotenv!("DISCORD_GUILD_ID__EC"));
    let auth = format!("Bot {}", dotenv!("DISCORD_TOKEN"));
    let client = reqwest::Client::new();
    let res = client
        .get(endpoint.to_str().unwrap())
        .header("authorization", auth)
        .send()
        .await?;

    match res.status() {
        status if status.is_success() => {
            let body = res.json::<Vec<Emoji>>().await?;
            Ok(body)
        }
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

async fn add_emoji() -> Result<Emoji, Box<dyn Error>> {
    let bytes = utils::get_file_as_byte_vec(Path::new("dist/4.3e9.1f603.1f603.smiley.smiley.png"));
    let mime_type = "image/png";
    let body = CreateEmoji {
        name: "test_delete".to_owned(),
        image: format!("data:{};base64,{}", mime_type, base64::encode(bytes)),
        roles: vec![],
    };

    println!("{:#?}", body);

    let endpoint = get_emoji_endpoint(dotenv!("DISCORD_GUILD_ID__EC"));
    let auth = format!("Bot {}", dotenv!("DISCORD_TOKEN"));
    let client = reqwest::Client::new();
    let res = client
        .post(endpoint.to_str().unwrap())
        .json(&body)
        .header("authorization", auth)
        .send()
        .await?;

    match res.status() {
        status if status.is_success() => Ok(res.json::<Emoji>().await?),
        status => Err(Box::<dyn Error>::from(status.to_string())),
    }
}

pub async fn discord(name_query: &Option<String>) -> Result<(), Box<dyn Error>> {
    // let emoji = fetch_emoji().await.unwrap();
    let emoji = add_emoji().await.unwrap();

    println!("{:#?}", &emoji);

    Ok(())
}
