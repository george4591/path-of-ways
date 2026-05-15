use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub category: String,
    pub ingredients: Vec<String>,
    pub result: String,
    #[serde(default)]
    pub note: Option<String>,
    pub created_at: f64,
}

impl Recipe {
    pub fn new(
        name: String,
        category: String,
        ingredients: Vec<String>,
        result: String,
        note: Option<String>,
    ) -> Self {
        Self {
            id: format!("r_{}", Uuid::new_v4().simple()),
            name,
            category,
            ingredients,
            result,
            note,
            created_at: js_sys::Date::now(),
        }
    }
}
