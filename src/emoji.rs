use core::str;
use std::collections::HashMap;

use serde::Deserialize;

fn lowercase_serialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.to_lowercase())
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct EmojiSkinVariation {
    #[serde(rename(deserialize = "unified"))]
    #[serde(deserialize_with = "lowercase_serialize")]
    pub codepoint: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Emoji {
    #[serde(rename(deserialize = "unified"))]
    #[serde(deserialize_with = "lowercase_serialize")]
    pub codepoint: String,
    pub name: String,
    #[serde(deserialize_with = "lowercase_serialize")]
    pub short_name: String,
    pub category: String,
    pub subcategory: String,
    pub sort_order: u16,
    #[serde(default)]
    pub skin_variations: HashMap<String, EmojiSkinVariation>,
}

pub fn get_emoji_map() -> HashMap<String, Emoji> {
    let json: Vec<Emoji> = serde_json::from_slice(include_bytes!("./emoji.json"))
        .expect("Failed to find or parse JSON!");
    let mut map = HashMap::new();
    for emoji in json {
        map.insert(emoji.codepoint.to_string(), emoji.clone());
    }
    map
}

#[derive(Debug)]
pub struct EmojiPair {
    pub d: String,
    pub base: Emoji,
    pub pair: Emoji,
    pub name: String,
    pub image_url: String,
    pub filename: String,
    pub sort_order: u16,
}

impl EmojiPair {
    fn generate_name(base: &Emoji, pair: &Emoji) -> String {
        format!("{}_{}", base.short_name, pair.short_name)
    }

    fn normalize_image_url_codepoint(date: i64, codepoint: &str) -> String {
        const BEFORE_DATE: i64 = 20220500;
        match codepoint.replace("-", "-u").as_str() {
            // https://github.com/alcor/emoji-supply/blob/main/docs/kitchen/script.js#L47-L49
            "00a9-ufe0f" if date >= BEFORE_DATE => "a9-ufe0f".to_string(),
            "00ae-ufe0f" if date >= BEFORE_DATE => "ae-ufe0f".to_string(),
            value => value.to_string(),
        }
    }

    fn generate_image_url(d: &str, base: &Emoji, pair: &Emoji) -> String {
        let date = i64::from_str_radix(d, 16).unwrap() + 20200000;
        let c1 = EmojiPair::normalize_image_url_codepoint(date, &base.codepoint);
        let c2 = EmojiPair::normalize_image_url_codepoint(date, &pair.codepoint);
        format!(
            "https://www.gstatic.com/android/keyboard/emojikitchen/{date}/u{c1}/u{c1}_u{c2}.png"
        )
    }

    fn generate_filename(d: &str, base: &Emoji, pair: &Emoji, sort_order: u16) -> String {
        format!(
            "{}.{}.{}.{}.{}.{}.png",
            sort_order, d, base.codepoint, pair.codepoint, base.short_name, pair.short_name
        )
    }

    pub fn new(d: &str, base: &Emoji, pair: &Emoji) -> Self {
        let sort_order = base.sort_order + pair.sort_order;
        EmojiPair {
            d: d.to_string(),
            base: base.clone(),
            pair: pair.clone(),
            name: EmojiPair::generate_name(&base, &pair),
            image_url: EmojiPair::generate_image_url(&d, &base, &pair),
            filename: EmojiPair::generate_filename(&d, &base, &pair, sort_order),
            sort_order,
        }
    }

    fn normalize_codepoint(codepoint: &str) -> String {
        match codepoint.to_lowercase().as_str() {
            // pairs.txt doesn't include "00" for some codepoints — so this adds them
            "a9-fe0f" => "00a9-fe0f".to_string(),
            "ae-fe0f" => "00ae-fe0f".to_string(),
            value => value.to_string(),
        }
    }

    pub fn from_pair_string(pair: &str, map: &HashMap<String, Emoji>) -> Self {
        // pair may end with "/", so 4 is used
        let mut split = pair.splitn(4, '/');
        let d = split.next().unwrap();
        let codepoint1 = EmojiPair::normalize_codepoint(split.next().unwrap());
        let codepoint2 = EmojiPair::normalize_codepoint(split.next().unwrap());
        let base = map.get(&codepoint1).expect(&format!(
            "⚠️ [BASE] emoji data for {} not found!",
            codepoint1
        ));
        let pair = map.get(&codepoint2).expect(&format!(
            "⚠️ [PAIR] emoji data for {} not found!",
            codepoint1
        ));
        EmojiPair::new(d, &base, &pair)
    }
}

pub struct EmojiDB {
    pub pairs: Vec<EmojiPair>,
}

pub struct Options {
    pub name: Option<String>,
}

pub fn init(options: Options) -> EmojiDB {
    let pairs_list: Vec<&str> = str::from_utf8(include_bytes!("./pairs.txt"))
        .unwrap()
        .trim()
        .split("\n")
        .collect();

    let emoji_map = get_emoji_map();

    let mut pairs: Vec<EmojiPair> = Vec::new();

    for pair in pairs_list {
        let emoji_pair = EmojiPair::from_pair_string(pair, &emoji_map);

        match &options.name {
            Some(name) => {
                if emoji_pair.base.short_name.eq(name) || emoji_pair.pair.short_name.eq(name) {
                    pairs.push(emoji_pair);
                }
            }
            None => {
                pairs.push(emoji_pair);
            }
        }
    }

    pairs.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

    EmojiDB { pairs }
}
