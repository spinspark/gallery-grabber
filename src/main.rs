use anyhow::{Context, anyhow};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::{Url, blocking::Client};
use std::{env, fs, fs::File, io::copy, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Base URL for gallery images
    #[arg(short, long, value_parser = parse_url)]
    url: Url,

    /// Number of pages to download
    #[arg(short, long)]
    pages: u32,

    /// Write downloaded files to <OUTPUT> instead of the current working directory
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn parse_url(url: &str) -> Result<Url, String> {
    let mut url = String::from(url);
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{url}");
    }
    if !url.ends_with('/') {
        url.push('/');
    }
    Url::parse(&url).map_err(|e| e.to_string())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let base_url = args.url;

    let download_dir = args
        .output
        .ok_or_else(|| anyhow!("No output directory specified"))
        .or_else(|_| env::current_dir())
        .context("Failed to find current working directory")?;

    fs::create_dir_all(&download_dir)?;

    let client = Client::builder()
        .build()
        .context("Failed to build HTTP Client")?;

    let bar_style = ProgressStyle::with_template("[{elapsed_precise}] {bar} {pos}/{len}")?;
    let bar = ProgressBar::new(args.pages as u64).with_style(bar_style);

    (1..=args.pages)
        .into_par_iter()
        .map(|i| {
            let url = base_url.join(&format!("{i}.webp"))?;
            let mut response = client.get(url).send()?.error_for_status()?;

            let filename = format!("{i:03}.webp");
            let path = download_dir.join(&filename);
            let mut file = File::create(&path)
                .with_context(|| format!("failed to create file {}", path.display()))?;
            copy(&mut response, &mut file)
                .with_context(|| format!("Failed to copy bytes to file {}", path.display()))?;
            bar.inc(1);
            Ok(())
        })
        .collect::<Result<(), anyhow::Error>>()?;

    bar.finish();
    Ok(())
}
