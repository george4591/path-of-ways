use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard tag categories the app understands out-of-the-box. Used in two
/// places: (1) the quick-add chips in the zone edit modal, and (2) the
/// always-visible pills in the campaign filter bar (which seed with these
/// even when no zone is using them yet, so the filter row has consistent
/// presence). Keep in sync with `tag_color_classes` below.
pub const COMMON_TAGS: &[&str] = &[
    "Skill Gem",
    "Skill Point",
    "Spirit",
    "Boss",
    "Quest",
    "Side Area",
    "Waypoint",
];

/// Pick a Tailwind color set for a tag based on a small palette of well-known
/// PoE2 reward types — so "Skill Gem" tags read green at a glance, "Boss" red,
/// etc. Custom/unknown tags fall back to the app's accent color.
///
/// Returns a string of `bg-* text-* border-*` classes ready to drop next to
/// the layout classes on the tag pill. Lives in `model` so both the campaign
/// view (zone-card tags, filter bar) and the campaign edit modal (applied-tag
/// pills, quick-add chips) can share the same palette.
pub fn tag_color_classes(tag: &str) -> &'static str {
    match tag {
        "Skill Gem" => "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
        "Skill Point" => "bg-amber-500/15 text-amber-300 border-amber-500/30",
        "Spirit" => "bg-violet-500/15 text-violet-300 border-violet-500/30",
        "Boss" => "bg-red-500/15 text-red-300 border-red-500/30",
        "Quest" => "bg-sky-500/15 text-sky-300 border-sky-500/30",
        "Side Area" => "bg-slate-500/15 text-slate-300 border-slate-500/30",
        "Waypoint" => "bg-orange-500/15 text-orange-300 border-orange-500/30",
        _ => "bg-accent/15 text-accent border-accent/30",
    }
}

/// A single freeform checklist entry on a zone — arbitrary label + a stable id
/// so the corresponding `ZoneProgress.done_items` entry survives renames.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
}

impl ChecklistItem {
    pub fn new(text: String) -> Self {
        Self {
            id: format!("ci_{}", Uuid::new_v4().simple()),
            text,
        }
    }

    /// Like `new`, but uses an explicit id. Used by seed data and migration so
    /// progress for well-known items lines up.
    pub fn with_id(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

/// A single campaign zone (e.g. "Clearfell"). Belongs to a named act ("Act 1:
/// The Riverbank"). Each zone owns a list of `ChecklistItem`s — there are no
/// fixed slots like "waypoint" or "boss" anymore — and a list of free-form
/// `tags` so the campaign view can be filtered by "what does this zone offer."
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Zone {
    pub id: String,
    pub act: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    pub created_at: f64,
}

impl Zone {
    pub fn new(
        act: String,
        name: String,
        tags: Vec<String>,
        checklist: Vec<ChecklistItem>,
    ) -> Self {
        Self {
            id: format!("z_{}", Uuid::new_v4().simple()),
            act,
            name,
            tags,
            checklist,
            created_at: js_sys::Date::now(),
        }
    }
}

/// Per-zone done state. `done_items` holds the ids of checked checklist items;
/// anything not in the set is considered unchecked.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ZoneProgress {
    pub zone_id: String,
    #[serde(default)]
    pub done_items: HashSet<String>,
}

impl ZoneProgress {
    pub fn for_zone(zone_id: &str) -> Self {
        Self {
            zone_id: zone_id.to_string(),
            done_items: HashSet::new(),
        }
    }
}

// ─── Migration shim ──────────────────────────────────────────────────────
//
// Older records (pre-checklist) stored fixed flags (`has_waypoint`,
// `has_side_area`, `quest_reward`, `boss`) on the zone itself, and separate
// boolean fields on the progress (`waypoint_done`, etc.). These two types
// deserialize the old shape and convert it to the current one in-place.
// We use stable item ids ("wp", "sa", "qr", "boss") so progress survives.

/// Wire format for a zone as it might be stored on disk — fields from both the
/// old and new shapes, used only as a deserialization target.
#[derive(Deserialize)]
pub(super) struct StoredZone {
    pub id: String,
    pub act: String,
    pub name: String,
    pub created_at: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default)]
    pub has_waypoint: bool,
    #[serde(default)]
    pub has_side_area: bool,
    #[serde(default)]
    pub quest_reward: Option<String>,
    #[serde(default)]
    pub boss: Option<String>,
}

impl StoredZone {
    pub(super) fn into_zone(self) -> Zone {
        let mut checklist = self.checklist;
        let mut tags = self.tags;
        // Only migrate legacy fields if the new checklist hasn't been written
        // yet — that way we don't duplicate items on second load. While we're
        // at it, sync corresponding tags so existing data is filterable
        // without the user having to manually re-tag every zone.
        if checklist.is_empty() {
            if self.has_waypoint {
                checklist.push(ChecklistItem::with_id("wp", "Waypoint"));
                push_unique(&mut tags, "Waypoint");
            }
            if self.has_side_area {
                checklist.push(ChecklistItem::with_id("sa", "Side area"));
                push_unique(&mut tags, "Side Area");
            }
            if let Some(reward) = self.quest_reward.filter(|value| !value.is_empty()) {
                checklist.push(ChecklistItem::with_id(
                    "qr",
                    format!("Quest reward: {}", reward),
                ));
                push_unique(&mut tags, "Quest");
            }
            if let Some(boss) = self.boss.filter(|value| !value.is_empty()) {
                checklist.push(ChecklistItem::with_id("boss", format!("Boss: {}", boss)));
                push_unique(&mut tags, "Boss");
            }
        }
        Zone {
            id: self.id,
            act: self.act,
            name: self.name,
            tags,
            checklist,
            created_at: self.created_at,
        }
    }
}

fn push_unique(tags: &mut Vec<String>, value: &str) {
    if !tags.iter().any(|tag| tag == value) {
        tags.push(value.to_string());
    }
}

/// Wire format for zone progress on disk — old and new fields together.
#[derive(Deserialize)]
pub(super) struct StoredProgress {
    pub zone_id: String,
    #[serde(default)]
    pub done_items: HashSet<String>,
    #[serde(default)]
    pub waypoint_done: bool,
    #[serde(default)]
    pub side_area_done: bool,
    #[serde(default)]
    pub quest_reward_done: bool,
    #[serde(default)]
    pub boss_done: bool,
}

impl StoredProgress {
    pub(super) fn into_progress(self) -> ZoneProgress {
        let mut done_items = self.done_items;
        if done_items.is_empty() {
            if self.waypoint_done {
                done_items.insert("wp".to_string());
            }
            if self.side_area_done {
                done_items.insert("sa".to_string());
            }
            if self.quest_reward_done {
                done_items.insert("qr".to_string());
            }
            if self.boss_done {
                done_items.insert("boss".to_string());
            }
        }
        ZoneProgress {
            zone_id: self.zone_id,
            done_items,
        }
    }
}
