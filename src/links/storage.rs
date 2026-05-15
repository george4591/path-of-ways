use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, LINKS_STORE as STORE};

use super::model::Link;

type DbResult<T> = Result<T, String>;

trait IntoDbResult<T> {
    fn db_err(self) -> DbResult<T>;
}

impl<T, E: std::fmt::Display> IntoDbResult<T> for Result<T, E> {
    fn db_err(self) -> DbResult<T> {
        self.map_err(|e| e.to_string())
    }
}

pub async fn load_links() -> Vec<Link> {
    load_inner().await.unwrap_or_default()
}

async fn load_inner() -> Result<Vec<Link>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadOnly)
        .db_err()?;
    let store = txn.store(STORE).db_err()?;
    let pairs = store.scan(None, None, None, None).await.db_err()?;
    let links = pairs
        .into_iter()
        .filter_map(|(_, v)| {
            serde_wasm_bindgen::from_value::<Link>(v)
                .map_err(|e| web_sys::console::warn_1(&format!("skipping bad record: {e}").into()))
                .ok()
        })
        .collect();
    txn.done().await.db_err()?;

    Ok(links)
}

pub async fn save_link(link: Link) {
    if let Err(e) = save_inner(&link).await {
        web_sys::console::error_1(&format!("save_link failed: {e}").into());
    }
}

async fn save_inner(link: &Link) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .db_err()?;
    let store = txn.store(STORE).db_err()?;
    let val = serde_wasm_bindgen::to_value(link).db_err()?;

    store.put(&val, None).await.db_err()?;
    txn.done().await.db_err()?;

    Ok(())
}

pub async fn delete_link(id: &str) {
    if let Err(e) = delete_inner(id).await {
        web_sys::console::error_1(&format!("save_link failed: {e}").into());
    }
}

async fn delete_inner(id: &str) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .db_err()?;
    let store = txn.store(STORE).db_err()?;

    store.delete(JsValue::from_str(id)).await.db_err()?;
    txn.done().await.db_err()?;

    Ok(())
}
