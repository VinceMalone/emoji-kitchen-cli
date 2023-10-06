use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::emoji;

#[derive(Serialize)]
struct JsonEmoji {
    codepoint: String,
    name: String,
    short_name: String,
    category: String,
    subcategory: String,
    sort_order: u16,
    skin_variations: HashMap<String, String>,
}

#[derive(Serialize)]
struct JsonEmojiPair {
    name: String,
    src: String,
    sort_order: u16,
    d: String,
    base: JsonEmoji,
    pair: JsonEmoji,
}

pub async fn json(pairs: Vec<emoji::EmojiPair>, output: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(output_path) = output.parent() {
        fs::create_dir_all(output_path).expect(&format!(
            "could not create output dir: {}",
            output.display()
        ));
    }

    let emoji_data: Vec<JsonEmojiPair> = pairs
        .iter()
        .map(|e| JsonEmojiPair {
            name: e.name.to_string(),
            src: e.image_url.to_string(),
            sort_order: e.sort_order,
            d: e.d.to_string(),
            base: JsonEmoji {
                codepoint: e.base.codepoint.to_string(),
                name: e.base.name.to_string(),
                short_name: e.base.short_name.to_string(),
                category: e.base.category.to_string(),
                subcategory: e.base.subcategory.to_string(),
                sort_order: e.base.sort_order,
                skin_variations: e
                    .base
                    .skin_variations
                    .iter()
                    .map(|(k, v)| (k.to_owned(), v.codepoint.to_owned()))
                    .collect(),
            },
            pair: JsonEmoji {
                codepoint: e.pair.codepoint.to_string(),
                name: e.pair.name.to_string(),
                short_name: e.pair.short_name.to_string(),
                category: e.pair.category.to_string(),
                subcategory: e.pair.subcategory.to_string(),
                sort_order: e.pair.sort_order,
                skin_variations: e
                    .pair
                    .skin_variations
                    .iter()
                    .map(|(k, v)| (k.to_owned(), v.codepoint.to_owned()))
                    .collect(),
            },
        })
        .collect();

    let json = serde_json::to_string_pretty(&emoji_data)?;

    fs::write(output, json)?;

    Ok(())
}
