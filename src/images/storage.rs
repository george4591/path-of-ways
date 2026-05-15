use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::{JsCast, JsValue};

use crate::db::{open_db, IMAGES_STORE as STORE};

thread_local! {
    static URL_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub fn new_image_id() -> String {
    format!("img_{}", js_sys::Date::now() as u64)
}

pub async fn save_image(blob: &web_sys::Blob) -> Option<String> {
    let id = new_image_id();
    save_inner(&id, blob).await.ok()?;
    Some(id)
}

async fn save_inner(id: &str, blob: &web_sys::Blob) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_str(id))
        .map_err(|_| "set id failed".to_string())?;
    js_sys::Reflect::set(&obj, &JsValue::from_str("blob"), blob.as_ref())
        .map_err(|_| "set blob failed".to_string())?;
    store
        .put(obj.as_ref(), None)
        .await
        .map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

async fn load_blob(id: &str) -> Option<web_sys::Blob> {
    let rexie = open_db().await.ok()?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadOnly)
        .ok()?;
    let store = txn.store(STORE).ok()?;
    let result = store.get(JsValue::from_str(id)).await.ok()??;
    let blob_val = js_sys::Reflect::get(&result, &JsValue::from_str("blob")).ok()?;
    blob_val.dyn_into::<web_sys::Blob>().ok()
}

pub async fn get_object_url(id: &str) -> Option<String> {
    if let Some(cached) = URL_CACHE.with(|cache| cache.borrow().get(id).cloned()) {
        return Some(cached);
    }
    let blob = load_blob(id).await?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok()?;
    URL_CACHE.with(|cache| cache.borrow_mut().insert(id.to_string(), url.clone()));
    Some(url)
}

/// Find every `image:ID` reference in a body (image markdown URLs, or
/// stray references — over-keeping is fine, under-deleting matters).
pub fn extract_image_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = body;
    while let Some(idx) = rest.find("image:") {
        let after = &rest[idx + "image:".len()..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        if end > 0 {
            ids.push(after[..end].to_string());
        }
        rest = &after[end..];
    }
    ids
}

/// Delete every image whose id isn't in `used`. Returns how many were removed.
pub async fn gc_orphans(used: HashSet<String>) -> usize {
    gc_inner(used).await.unwrap_or(0)
}

async fn gc_inner(used: HashSet<String>) -> Result<usize, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|err| err.to_string())?;
    let mut deleted = 0usize;
    for (key, _) in pairs {
        let Some(id) = key.as_string() else { continue };
        if used.contains(&id) {
            continue;
        }
        if let Err(_) = store.delete(key).await {
            continue;
        }
        if let Some(url) = URL_CACHE.with(|cache| cache.borrow_mut().remove(&id)) {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
        deleted += 1;
    }
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(deleted)
}
