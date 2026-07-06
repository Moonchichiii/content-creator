mod campaign;
mod config;
mod logs;

use anyhow::{bail, Context, Result};
use campaign::{static_caption, CampaignsConfig};
use chrono::Utc;
use config::Config;
use logs::has_recent_duplicate;

fn main() -> Result<()> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| help().to_string());

    match command.as_str() {
        "validate-config" => validate_config(),
        "dry-run" => dry_run(),
        _ => bail!("unknown command: {command}\n{}", help()),
    }
}

fn validate_config() -> Result<()> {
    let config = Config::from_env()?;
    CampaignsConfig::from_file(&config.posts_config).with_context(|| {
        format!(
            "failed to validate posts config at {}",
            config.posts_config.display()
        )
    })?;

    println!("Config OK");
    Ok(())
}

fn dry_run() -> Result<()> {
    let config = Config::from_env()?;
    let campaigns = CampaignsConfig::from_file(&config.posts_config)?;
    let campaign = campaigns
        .campaigns
        .iter()
        .find(|campaign| {
            !has_recent_duplicate(&config.post_log, &campaign.name, Utc::now()).unwrap_or(false)
        })
        .context("no campaign available without a recent duplicate")?;

    println!("DRY RUN ONLY");
    println!("Campaign: {}", campaign.name);
    println!("Image: {}", campaign.image_url);
    println!("Caption: {}", static_caption(campaign));

    Ok(())
}

fn help() -> &'static str {
    "usage: ig-agent <validate-config|dry-run>"
}
