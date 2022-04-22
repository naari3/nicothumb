use anyhow::Result;
use clap::Parser;
use kuchiki::traits::TendrilSink;
use regex::Regex;
use tokio::{
    fs::{create_dir_all, File},
    io,
};

#[derive(Parser, Debug)]
#[clap(
    name = "nicothumb",
    author = "naari3",
    version = "v0.1.0",
    about = "Download nicovideo thumbnail."
)]
struct AppArg {
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let arg: AppArg = AppArg::parse();

    let re = Regex::new(r"(?P<id>[ns][mo]\d+)").unwrap();
    let id = match re.captures(&arg.url) {
        Some(c) => c.name("id").expect("video id not found").as_str(),
        None => panic!("video id not found"),
    };

    let response = reqwest::get(format!("https://www.nicovideo.jp/watch/{id}")).await?;
    let html = response.text().await?;

    let document = kuchiki::parse_html().one(html);
    let thumbnail_meta = document
        .select_first("head > meta[name='thumbnail']")
        .unwrap();

    let thumbnail_meta = thumbnail_meta.attributes.borrow();
    let thumbnail_content = thumbnail_meta.get("content");

    let thumbnail_url = thumbnail_content.expect("thumbnail does not exist.");
    let response = reqwest::get(thumbnail_url).await?;
    let bytes = response.bytes().await?;

    create_dir_all("./thumbnails").await?;
    let mut out = File::create(format!("./thumbnails/{id}.jpg")).await?;
    io::copy(&mut bytes.as_ref(), &mut out).await?;

    Ok(())
}
