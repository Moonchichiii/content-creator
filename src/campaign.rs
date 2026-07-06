use std::{fs, path::Path};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Campaign {
    pub name: String,
    pub audience: String,
    pub tone: String,
    pub cta: String,
    pub image_url: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct CampaignsConfig {
    pub campaigns: Vec<Campaign>,
}

#[derive(Debug, Error)]
pub enum CampaignError {
    #[error("failed to read campaigns config: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse campaigns config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("campaigns config must contain at least one campaign")]
    EmptyCampaigns,
    #[error("campaign {index} is missing required field: {field}")]
    MissingField { index: usize, field: &'static str },
    #[error("campaign {index} image_url must start with https://")]
    InvalidImageUrl { index: usize },
}

impl CampaignsConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, CampaignError> {
        let contents = fs::read_to_string(path)?;
        Self::from_str(&contents)
    }

    pub fn from_str(contents: &str) -> Result<Self, CampaignError> {
        let config: Self = toml::from_str(contents)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), CampaignError> {
        if self.campaigns.is_empty() {
            return Err(CampaignError::EmptyCampaigns);
        }

        for (index, campaign) in self.campaigns.iter().enumerate() {
            required(index, "name", &campaign.name)?;
            required(index, "audience", &campaign.audience)?;
            required(index, "tone", &campaign.tone)?;
            required(index, "cta", &campaign.cta)?;
            required(index, "image_url", &campaign.image_url)?;

            if !campaign.image_url.starts_with("https://") {
                return Err(CampaignError::InvalidImageUrl { index });
            }
        }

        Ok(())
    }
}

pub fn static_caption(campaign: &Campaign) -> String {
    format!(
        "Håll bättre ordning med {}. {}. #fordon #servicebok #digitalt",
        campaign.name, campaign.cta
    )
}

fn required(index: usize, field: &'static str, value: &str) -> Result<(), CampaignError> {
    if value.trim().is_empty() {
        return Err(CampaignError::MissingField { index, field });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_valid_posts_toml() {
        let config = CampaignsConfig::from_file("assets/posts.toml").expect("valid posts config");

        assert_eq!(config.campaigns.len(), 2);
        assert_eq!(config.campaigns[0].name, "Valunds ServiceBok");
    }

    #[test]
    fn fails_on_empty_campaign_list() {
        let error = CampaignsConfig::from_str("campaigns = []").expect_err("should fail");

        assert!(matches!(error, CampaignError::EmptyCampaigns));
    }

    #[test]
    fn fails_on_image_url_without_https() {
        let contents = r#"
[[campaigns]]
name = "Bad"
audience = "audience"
tone = "tone"
cta = "cta"
image_url = "http://example.com/image.webp"
"#;

        let error = CampaignsConfig::from_str(contents).expect_err("should fail");

        assert!(matches!(error, CampaignError::InvalidImageUrl { index: 0 }));
    }
}
