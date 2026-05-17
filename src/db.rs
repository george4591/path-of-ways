use rexie::{ObjectStore, Rexie};

pub const DB_NAME: &str = "path-of-ways";
pub const NOTES_STORE: &str = "notes";
pub const CAMPAIGN_STORE: &str = "campaign";
pub const IMAGES_STORE: &str = "images";
pub const LINKS_STORE: &str = "links";
pub const RECIPES_STORE: &str = "recipes";
pub const ZONES_STORE: &str = "zones";

// Note: existing IDBs created at version 7 may still contain a `bosses`
// object store from before the Bosses tab was removed. It's orphan dead
// weight but harmless; nothing in the app references it anymore. To clean
// it up properly would require bumping the version and an explicit drop
// — not worth the complexity for a personal app.

pub async fn open_db() -> Result<Rexie, String> {
    Rexie::builder(DB_NAME)
        .version(7)
        .add_object_store(ObjectStore::new(NOTES_STORE).key_path("id"))
        .add_object_store(ObjectStore::new(CAMPAIGN_STORE).key_path("zone_id"))
        .add_object_store(ObjectStore::new(IMAGES_STORE).key_path("id"))
        .add_object_store(ObjectStore::new(LINKS_STORE).key_path("id"))
        .add_object_store(ObjectStore::new(RECIPES_STORE).key_path("id"))
        .add_object_store(ObjectStore::new(ZONES_STORE).key_path("id"))
        .build()
        .await
        .map_err(|err| err.to_string())
}
