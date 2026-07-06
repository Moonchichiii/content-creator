use std::{fs, io, path::Path};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostLogEntry {
    pub campaign_name: String,
    pub posted_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum LogError {
    #[error("failed to read post log: {0}")]
    Read(#[from] io::Error),
}

pub fn has_recent_duplicate(
    path: impl AsRef<Path>,
    campaign_name: &str,
    now: DateTime<Utc>,
) -> Result<bool, LogError> {
    let path = path.as_ref();
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(LogError::Read(error)),
    };

    let cutoff = now - Duration::days(14);

    Ok(contents.lines().filter_map(parse_line).any(|entry| {
        entry.campaign_name == campaign_name && entry.posted_at >= cutoff && entry.posted_at <= now
    }))
}

fn parse_line(line: &str) -> Option<PostLogEntry> {
    serde_json::from_str(line).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    fn temp_log(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "ig-agent-{name}-{}.jsonl",
            Utc::now().timestamp_nanos_opt().unwrap()
        ))
    }

    #[test]
    fn empty_log_gives_no_duplicate() {
        let path = temp_log("empty");
        fs::write(&path, "").expect("write log");
        let now = Utc::now();

        assert!(!has_recent_duplicate(&path, "Campaign", now).expect("check duplicate"));
    }

    #[test]
    fn campaign_posted_yesterday_gives_duplicate() {
        let path = temp_log("yesterday");
        let now = Utc::now();
        let entry = PostLogEntry {
            campaign_name: "Campaign".to_string(),
            posted_at: now - Duration::days(1),
        };
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&entry).unwrap()),
        )
        .expect("write log");

        assert!(has_recent_duplicate(&path, "Campaign", now).expect("check duplicate"));
    }

    #[test]
    fn campaign_posted_twenty_days_ago_gives_no_duplicate() {
        let path = temp_log("old");
        let now = Utc::now();
        let entry = PostLogEntry {
            campaign_name: "Campaign".to_string(),
            posted_at: now - Duration::days(20),
        };
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&entry).unwrap()),
        )
        .expect("write log");

        assert!(!has_recent_duplicate(&path, "Campaign", now).expect("check duplicate"));
    }

    #[test]
    fn broken_json_line_does_not_panic() {
        let path = temp_log("broken");
        let now = Utc::now();
        fs::write(&path, "not-json\n").expect("write log");

        assert!(!has_recent_duplicate(&path, "Campaign", now).expect("check duplicate"));
    }
}
