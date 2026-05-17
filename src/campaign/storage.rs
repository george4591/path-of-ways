use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, CAMPAIGN_STORE, ZONES_STORE};

use super::data::seed_zones;
use super::model::{StoredProgress, StoredZone, Zone, ZoneProgress};

/// Bump this when changing `seed_zones` in a way that should overwrite a
/// user's existing zones/progress. The mismatch with the value in
/// `localStorage` triggers a wipe-and-reseed on next load.
///
/// v1 → v2 — initial migration to checklist-item zones
/// v2 → v3 — campaign rewritten to current PoE2 structure (4 acts +
///           3 interludes, no Cruel difficulty)
const ZONES_SEEDED_KEY: &str = "zones_seeded_v3";

// ─── Zone metadata ───────────────────────────────────────────────────────

pub async fn load_zones() -> Vec<Zone> {
    if !has_seeded() {
        // The seed key was bumped (or this is a fresh install) — wipe the
        // existing zone + progress stores so the new defaults don't merge
        // with stale data from a previous game version. If a clear fails,
        // log it and bail out *without* marking seeded, so we'll retry next
        // load instead of leaving the user with a mixed bag.
        if let Err(err) = clear_store(ZONES_STORE).await {
            web_sys::console::error_1(
                &format!("campaign: failed to clear zones store: {err}").into(),
            );
            return load_zones_from_idb().await.unwrap_or_default();
        }
        if let Err(err) = clear_store(CAMPAIGN_STORE).await {
            web_sys::console::error_1(
                &format!("campaign: failed to clear progress store: {err}").into(),
            );
            return load_zones_from_idb().await.unwrap_or_default();
        }
        let seed = seed_zones();
        for zone in &seed {
            let _ = save_zone_inner(zone).await;
        }
        mark_seeded();
        return seed;
    }
    load_zones_from_idb().await.unwrap_or_default()
}

async fn clear_store(store_name: &str) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[store_name], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(store_name).map_err(|err| err.to_string())?;
    store.clear().await.map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
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
        .filter_map(|(_, value)| {
            serde_wasm_bindgen::from_value::<StoredZone>(value)
                .ok()
                .map(StoredZone::into_zone)
        })
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
        .filter_map(|(_, value)| {
            serde_wasm_bindgen::from_value::<StoredProgress>(value)
                .ok()
                .map(StoredProgress::into_progress)
        })
        .collect();
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(progress)
}

pub async fn save_zone_progress(progress: ZoneProgress) {
    let _ = save_progress_inner(&progress).await;
}

/// Wipe every ZoneProgress record. Used by the "Reset progress" action — does
/// not touch the Zone records themselves so user-authored zones, checklist
/// items, and tags survive.
pub async fn clear_all_progress() {
    let _ = clear_store(CAMPAIGN_STORE).await;
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
