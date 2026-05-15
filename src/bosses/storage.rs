use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, BOSSES_STORE as STORE};

use super::data::seed_bosses;
use super::model::Boss;

const SEEDED_KEY: &str = "bosses_seeded_v1";

pub async fn load_bosses() -> Vec<Boss> {
    if !has_seeded() {
        let seed = seed_bosses();
        for boss in &seed {
            let _ = save_inner(boss).await;
        }
        mark_seeded();
        return seed;
    }
    load_from_idb().await.unwrap_or_default()
}

async fn load_from_idb() -> Result<Vec<Boss>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadOnly)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|err| err.to_string())?;
    let bosses = pairs
        .into_iter()
        .filter_map(|(_, value)| serde_wasm_bindgen::from_value::<Boss>(value).ok())
        .collect();
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(bosses)
}

pub async fn save_boss(boss: Boss) {
    let _ = save_inner(&boss).await;
}

async fn save_inner(boss: &Boss) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let value = serde_wasm_bindgen::to_value(boss).map_err(|err| err.to_string())?;
    store.put(&value, None).await.map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_boss(id: String) {
    let _ = delete_inner(&id).await;
}

async fn delete_inner(id: &str) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    store
        .delete(JsValue::from_str(id))
        .await
        .map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn has_seeded() -> bool {
    local_storage()
        .and_then(|storage| storage.get_item(SEEDED_KEY).ok().flatten())
        .is_some()
}

fn mark_seeded() {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SEEDED_KEY, "1");
    }
}
