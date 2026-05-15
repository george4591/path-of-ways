use leptos::web_sys;
use rexie::TransactionMode;
use serde::{Deserialize, Serialize};

use crate::db::{open_db, LINKS_STORE as STORE};

#[derive(Clone, Serialize, Deserialize)]
pub struct LinkRow {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
}

impl LinkRow {
    pub fn domain(&self) -> &str {
        self.url
            .strip_prefix("https://")
            .or_else(|| self.url.strip_prefix("http://"))
            .unwrap_or(&self.url)
            .split('/')
            .next()
            .unwrap_or("")
    }
}

fn console_error(msg: &str) {
    web_sys::console::error_1(&msg.into());
}

pub async fn load_links() -> Vec<LinkRow> {
    let Ok(rexie) = open_db().await else {
        console_error("load_links: failed to open db");
        return Vec::new();
    };

    let Ok(txn) = rexie.transaction(&[STORE], TransactionMode::ReadOnly) else {
        console_error("load_links: failed to create transaction");
        return Vec::new();
    };

    let Ok(store) = txn.store(STORE) else {
        console_error("load_links: failed to get store");
        return Vec::new();
    };

    let Ok(pairs) = store.scan(None, None, None, None).await else {
        console_error("load_links: failed to scan");
        return Vec::new();
    };

    pairs
        .into_iter()
        .filter_map(|(_, v)| {
            serde_wasm_bindgen::from_value::<LinkRow>(v)
                .map_err(|e| console_error(&format!("load_links: skipping bad record: {e}")))
                .ok()
        })
        .collect()
}

pub fn open_url(url: &str) {
    crate::external::open_external(url);
}
