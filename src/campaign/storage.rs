use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, CAMPAIGN_STORE, ZONES_STORE};

use super::data::seed_zones;
use super::model::{Zone, ZoneProgress};

const ZONES_SEEDED_KEY: &str = "zones_seeded_v1";

// ─── Zone metadata ───────────────────────────────────────────────────────

pub async fn load_zones() -> Vec<Zone> {
    if !has_seeded() {
        let seed = seed_zones();
        for zone in &seed {
            let _ = save_zone_inner(zone).await;
        }
        mark_seeded();
        return seed;
    }
    load_zones_from_idb().await.unwrap_or_default()
}

async fn load_zones_from_idb() -> Result<Vec<Zone>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[ZONES_STORE], TransactionMode::ReadOnly)
        .map_err(|err| err.to_string())?;
    let store = txn.store(ZONES_STORE).map_err(|err| err.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|err| err.to_string())?;
    let zones = pairs
        .into_iter()
        .filter_map(|(_, value)| serde_wasm_bindgen::from_value::<Zone>(value).ok())
        .collect();
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(zones)
}

pub async fn save_zone(zone: Zone) {
    let _ = save_zone_inner(&zone).await;
}

async fn save_zone_inner(zone: &Zone) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[ZONES_STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(ZONES_STORE).map_err(|err| err.to_string())?;
    let value = serde_wasm_bindgen::to_value(zone).map_err(|err| err.to_string())?;
    store.put(&value, None).await.map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_zone(id: String) {
    let _ = delete_zone_inner(&id).await;
}

async fn delete_zone_inner(id: &str) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[ZONES_STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(ZONES_STORE).map_err(|err| err.to_string())?;
    store
        .delete(JsValue::from_str(id))
        .await
        .map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

// ─── Zone progress (checklist state) ─────────────────────────────────────

pub async fn load_progress() -> Vec<ZoneProgress> {
    load_progress_inner().await.unwrap_or_default()
}

async fn load_progress_inner() -> Result<Vec<ZoneProgress>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[CAMPAIGN_STORE], TransactionMode::ReadOnly)
        .map_err(|err| err.to_string())?;
    let store = txn.store(CAMPAIGN_STORE).map_err(|err| err.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|err| err.to_string())?;
    let progress = pairs
        .into_iter()
        .filter_map(|(_, value)| serde_wasm_bindgen::from_value::<ZoneProgress>(value).ok())
        .collect();
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(progress)
}

pub async fn save_zone_progress(progress: ZoneProgress) {
    let _ = save_progress_inner(&progress).await;
}

async fn save_progress_inner(progress: &ZoneProgress) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[CAMPAIGN_STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(CAMPAIGN_STORE).map_err(|err| err.to_string())?;
    let value = serde_wasm_bindgen::to_value(progress).map_err(|err| err.to_string())?;
    store.put(&value, None).await.map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

// ─── First-run seeding helpers ───────────────────────────────────────────

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn has_seeded() -> bool {
    local_storage()
        .and_then(|storage| storage.get_item(ZONES_SEEDED_KEY).ok().flatten())
        .is_some()
}

fn mark_seeded() {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(ZONES_SEEDED_KEY, "1");
    }
}
