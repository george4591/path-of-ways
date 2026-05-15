use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, NOTES_STORE as STORE};

use super::model::Note;

const LEGACY_KEY: &str = "notes_v1";

async fn load_from_idb() -> Result<Vec<Note>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadOnly)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE).map_err(|e| e.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|e| e.to_string())?;
    let notes = pairs
        .into_iter()
        .filter_map(|(_, v)| serde_wasm_bindgen::from_value::<Note>(v).ok())
        .collect();
    txn.done().await.map_err(|e| e.to_string())?;
    Ok(notes)
}

async fn save_to_idb(list: &[Note]) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE).map_err(|e| e.to_string())?;
    store.clear().await.map_err(|e| e.to_string())?;
    for note in list {
        let val = serde_wasm_bindgen::to_value(note).map_err(|e| e.to_string())?;
        store.put(&val, None).await.map_err(|e| e.to_string())?;
    }
    txn.done().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn load_notes() -> Vec<Note> {
    if let Ok(notes) = load_from_idb().await {
        if !notes.is_empty() {
            return notes;
        }
    }
    let migrated = migrate_from_localstorage();
    if !migrated.is_empty() {
        let _ = save_to_idb(&migrated).await;
        clear_localstorage_legacy();
    }
    migrated
}

pub async fn save_one(note: Note) {
    let _ = save_one_inner(&note).await;
}

async fn save_one_inner(note: &Note) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE).map_err(|e| e.to_string())?;
    let val = serde_wasm_bindgen::to_value(note).map_err(|e| e.to_string())?;
    store.put(&val, None).await.map_err(|e| e.to_string())?;
    txn.done().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_one(id: String) {
    let _ = delete_one_inner(&id).await;
}

async fn delete_one_inner(id: &str) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE).map_err(|e| e.to_string())?;
    let key = JsValue::from_str(id);
    store.delete(key).await.map_err(|e| e.to_string())?;
    txn.done().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn save_many(list: Vec<Note>) {
    for n in list {
        let _ = save_one_inner(&n).await;
    }
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn migrate_from_localstorage() -> Vec<Note> {
    let Some(storage) = local_storage() else {
        return Vec::new();
    };
    if let Ok(Some(json)) = storage.get_item(LEGACY_KEY) {
        if let Ok(list) = serde_json::from_str::<Vec<Note>>(&json) {
            return list;
        }
    }
    Vec::new()
}

fn clear_localstorage_legacy() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(LEGACY_KEY);
        let _ = storage.remove_item("notes");
    }
}

pub fn export_json(list: &[Note]) -> String {
    serde_json::to_string_pretty(list).unwrap_or_else(|_| "[]".to_string())
}

pub fn import_json(json: &str) -> Result<Vec<Note>, String> {
    serde_json::from_str::<Vec<Note>>(json).map_err(|e| e.to_string())
}

pub fn trigger_download(filename: &str, contents: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(contents));
    let bag = web_sys::BlobPropertyBag::new();
    bag.set_type("application/json");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &bag) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    let Ok(anchor) = document.create_element("a") else { return };
    let anchor: web_sys::HtmlAnchorElement = match anchor.dyn_into() {
        Ok(a) => a,
        Err(_) => return,
    };
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    let _ = web_sys::Url::revoke_object_url(&url);
}

use wasm_bindgen::JsCast;
