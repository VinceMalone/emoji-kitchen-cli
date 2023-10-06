#[macro_use]
extern crate dotenv_codegen;

use std::error::Error;
use std::fmt::Display;
use std::path::Path;

use clap::{Parser, Subcommand};

mod animations;
mod discord;
mod emoji;
mod upload;
mod utils;
mod write;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Animations {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        o: String,
        #[arg(short, long)]
        size: u32,
    },
    Discord {
        #[arg(short, long)]
        name: Option<String>,
    },
    Download {
        #[arg(short, long)]
        name: Option<String>,
    },
    Json {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        output: String,
    },
    Show {
        #[arg(short, long)]
        count: bool,
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        preview: bool,
    },
    ShowAnimated {
        #[arg(short, long)]
        input: String,
    },
    Upload {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Animations { name, o, size }) => {
            animations::animations(Path::new(o), name, size).await;
        }
        Some(Commands::Discord { name }) => {
            discord::discord(name).await;
        }
        Some(Commands::Download { name }) => {
            let options = emoji::Options { name: name.clone() };
            let emoji = emoji::init(options);
            download(emoji.pairs).await;
        }
        Some(Commands::Json { name, output }) => {
            let now = std::time::SystemTime::now();
            println!("{:#?}", &now);
            let options = emoji::Options { name: name.clone() };
            let emoji = emoji::init(options);
            write::json(emoji.pairs, Path::new(output)).await;
            println!("{:#?}", now.elapsed().unwrap());
        }
        Some(Commands::Show {
            count,
            input,
            name,
            preview,
        }) => {
            let options = emoji::Options { name: name.clone() };
            let emoji = emoji::init(options);
            show(emoji.pairs, Path::new(input), count, preview);
        }
        Some(Commands::ShowAnimated { input }) => {
            show_animated(Path::new(input));
        }
        Some(Commands::Upload { input, name }) => {
            upload::upload(Path::new(input), name).await;
        }
        None => {
            println!("you fucked up. specify a command. TODO: print help output");
        }
    }
}

fn show_animated(input_path: &Path) {
    let paths = std::fs::read_dir(input_path).unwrap();
    let emoji_map = emoji::get_emoji_map();

    let mut emoji_list = Vec::new();

    for dir_result in paths {
        let path = dir_result.unwrap().path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        let mut split = filename.splitn(3, '.');
        split.next();
        let codepoint = split.next().unwrap().replace("_", "-");

        if !emoji_map.contains_key(&codepoint) {
            continue;
        }

        let emoji = emoji_map.get(&codepoint).unwrap();

        emoji_list.push((
            emoji.sort_order,
            format!(":{}_animated: ", &emoji.short_name),
        ));
    }

    emoji_list.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, name) in emoji_list {
        print!("{}", &name);
    }
}

fn show(pairs: Vec<emoji::EmojiPair>, input_path: &Path, count: &bool, preview: &bool) {
    if *count {
        println!("{}", pairs.len());
    } else {
        for pair in &pairs {
            println!("{}", pair.name);
            if *preview {
                let filename = input_path.join(&pair.filename);
                let config = viuer::Config {
                    transparent: true,
                    absolute_offset: false,
                    width: Some(16),
                    ..Default::default()
                };
                match viuer::print_from_file(filename, &config) {
                    Err(_) => {}
                    Ok(_) => {}
                }
            }
        }
    }
}

async fn download(pairs: Vec<emoji::EmojiPair>) {
    println!("{} pairs found", pairs.len());

    let mut errors: Vec<(Box<dyn Display>, Box<dyn Error>)> = Vec::new();

    for pair in &pairs {
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
