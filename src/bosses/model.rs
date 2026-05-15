use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Boss {
    pub id: String,
    pub name: String,
    /// Free-form grouping label, e.g. "Act 1", "Pinnacle", "Atlas".
    pub category: String,
    /// Optional zone hint (e.g. "Manor Ramparts"). Empty when not applicable.
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub description: String,
    pub created_at: f64,
}

impl Boss {
    pub fn new(name: String, category: String, zone: String, description: String) -> Self {
        Self {
            id: format!("b_{}", Uuid::new_v4().simple()),
            name,
            category,
            zone,
            description,
            created_at: js_sys::Date::now(),
        }
    }
}
