use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single campaign zone (e.g. "Clearfell"). Belongs to a named act ("Act 1:
/// The Riverbank"). The optional `quest_reward` and `boss` fields, when set,
/// add corresponding checkboxes to the zone's progress card.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Zone {
    pub id: String,
    pub act: String,
    pub name: String,
    #[serde(default)]
    pub has_waypoint: bool,
    #[serde(default)]
    pub has_side_area: bool,
    #[serde(default)]
    pub quest_reward: Option<String>,
    #[serde(default)]
    pub boss: Option<String>,
    pub created_at: f64,
}

impl Zone {
    pub fn new(
        act: String,
        name: String,
        has_waypoint: bool,
        has_side_area: bool,
        quest_reward: Option<String>,
        boss: Option<String>,
    ) -> Self {
        Self {
            id: format!("z_{}", Uuid::new_v4().simple()),
            act,
            name,
            has_waypoint,
            has_side_area,
            quest_reward,
            boss,
            created_at: js_sys::Date::now(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ZoneProgress {
    pub zone_id: String,
    #[serde(default)]
    pub waypoint_done: bool,
    #[serde(default)]
    pub side_area_done: bool,
    #[serde(default)]
    pub quest_reward_done: bool,
    #[serde(default)]
    pub boss_done: bool,
}

impl ZoneProgress {
    pub fn for_zone(zone_id: &str) -> Self {
        Self {
            zone_id: zone_id.to_string(),
            ..Default::default()
        }
    }
}
