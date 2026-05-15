use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    #[serde(default)]
    pub title: String,
    pub body: String,
    pub created_at: f64,
    pub updated_at: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub zone_id: Option<String>,
}

impl Note {
    pub fn new_blank() -> Self {
        let now = now_ms();
        Self {
            id: new_id(),
            title: String::new(),
            body: String::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            pinned: false,
            zone_id: None,
        }
    }

    pub fn new_for_zone(zone_id: String, zone_title: String) -> Self {
        let mut note = Self::new_blank();
        note.zone_id = Some(zone_id);
        note.title = zone_title;
        note
    }

    pub fn display_title(&self) -> String {
        let trimmed = self.title.trim();
        if trimmed.is_empty() {
            "Untitled".to_string()
        } else {
            trimmed.to_string()
        }
    }

    pub fn body_preview(&self) -> String {
        for line in self.body.lines() {
            let stripped = strip_image_md(line).trim().to_string();
            if !stripped.is_empty() {
                return stripped;
            }
        }
        String::new()
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let needle = query.to_lowercase();
        self.title.to_lowercase().contains(&needle)
            || self.body.to_lowercase().contains(&needle)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&needle))
    }

    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        if tags.is_empty() {
            return true;
        }
        tags.iter().any(|t| self.tags.iter().any(|nt| nt == t))
    }
}

/// Remove markdown image references (`![alt](url)`) from a line, leaving any
/// surrounding text. Used by the sidebar preview so embedded images don't
/// render as broken-image placeholders next to text.
fn strip_image_md(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(idx) = rest.find("![") {
        out.push_str(&rest[..idx]);
        let after_bang = &rest[idx + 2..];
        if let Some(alt_end) = after_bang.find("](") {
            let url_start = idx + 2 + alt_end + 2;
            if let Some(url_end_rel) = rest[url_start..].find(')') {
                rest = &rest[url_start + url_end_rel + 1..];
                continue;
            }
        }
        // Unmatched `![` — keep it literal and move on.
        out.push_str("![");
        rest = &rest[idx + 2..];
    }
    out.push_str(rest);
    out
}

pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

pub fn new_id() -> String {
    format!("n_{}", now_ms() as u64)
}

pub fn format_relative(timestamp_ms: f64) -> String {
    let diff_secs = ((now_ms() - timestamp_ms) / 1000.0).max(0.0);
    if diff_secs < 60.0 {
        "just now".to_string()
    } else if diff_secs < 3600.0 {
        format!("{}m ago", (diff_secs / 60.0) as u64)
    } else if diff_secs < 86400.0 {
        format!("{}h ago", (diff_secs / 3600.0) as u64)
    } else if diff_secs < 86400.0 * 30.0 {
        format!("{}d ago", (diff_secs / 86400.0) as u64)
    } else if diff_secs < 86400.0 * 365.0 {
        format!("{}mo ago", (diff_secs / (86400.0 * 30.0)) as u64)
    } else {
        format!("{}y ago", (diff_secs / (86400.0 * 365.0)) as u64)
    }
}

pub fn parse_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
