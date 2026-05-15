use leptos::web_sys;
use rexie::TransactionMode;
use wasm_bindgen::JsValue;

use crate::db::{open_db, RECIPES_STORE as STORE};

use super::data::seed_recipes;
use super::model::Recipe;

const SEEDED_KEY: &str = "recipes_seeded_v1";

/// Returns recipes from IndexedDB. On the first ever load, seeds the store
/// with default starter recipes and returns those.
pub async fn load_recipes() -> Vec<Recipe> {
    if !has_seeded() {
        let seed = seed_recipes();
        for recipe in &seed {
            let _ = save_inner(recipe).await;
        }
        mark_seeded();
        return seed;
    }
    load_from_idb().await.unwrap_or_default()
}

async fn load_from_idb() -> Result<Vec<Recipe>, String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadOnly)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let pairs = store
        .scan(None, None, None, None)
        .await
        .map_err(|err| err.to_string())?;
    let recipes = pairs
        .into_iter()
        .filter_map(|(_, value)| serde_wasm_bindgen::from_value::<Recipe>(value).ok())
        .collect();
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(recipes)
}

pub async fn save_recipe(recipe: Recipe) {
    let _ = save_inner(&recipe).await;
}

async fn save_inner(recipe: &Recipe) -> Result<(), String> {
    let rexie = open_db().await?;
    let txn = rexie
        .transaction(&[STORE], TransactionMode::ReadWrite)
        .map_err(|err| err.to_string())?;
    let store = txn.store(STORE).map_err(|err| err.to_string())?;
    let value = serde_wasm_bindgen::to_value(recipe).map_err(|err| err.to_string())?;
    store.put(&value, None).await.map_err(|err| err.to_string())?;
    txn.done().await.map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_recipe(id: String) {
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
