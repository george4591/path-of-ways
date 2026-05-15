use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Link {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub created_at: f64,
}

impl Link {
    pub fn new(url: String, title: String, description: String) -> Self {
        let now = js_sys::Date::now();
        Self {
            id: format!("l_{}", Uuid::new_v4().simple()),
            url,
            title,
            description,
            created_at: now,
        }
    }

    pub fn domain(&self) -> &str {
        let trimmed = self
            .url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .unwrap_or(&self.url);
        trimmed.split('/').next().unwrap_or(trimmed)
    }

    pub fn favicon_url(&self) -> String {
        format!(
            "https://www.google.com/s2/favicons?domain={}&sz=64",
            self.domain()
        )
    }
}
