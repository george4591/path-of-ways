use super::model::Recipe;

/// Default starter recipes seeded into IndexedDB the first time the user
/// opens the app. After seeding, all recipe state lives in the database —
/// users edit / delete / add via the UI and changes persist.
pub fn seed_recipes() -> Vec<Recipe> {
    vec![
        // ─── Currency shards ─────────────────────────────────────────
        Recipe::new(
            "Transmutation Orb".into(),
            "Currency".into(),
            vec!["20 Transmutation Shards".into()],
            "1 Transmutation Orb".into(),
            Some("Shards combine automatically once you reach the stack size.".into()),
        ),
        Recipe::new(
            "Augmentation Orb".into(),
            "Currency".into(),
            vec!["20 Augmentation Shards".into()],
            "1 Augmentation Orb".into(),
            None,
        ),
        Recipe::new(
            "Regal Orb".into(),
            "Currency".into(),
            vec!["20 Regal Shards".into()],
            "1 Regal Orb".into(),
            None,
        ),
        Recipe::new(
            "Chance Orb".into(),
            "Currency".into(),
            vec!["20 Chance Shards".into()],
            "1 Chance Orb".into(),
            None,
        ),
        Recipe::new(
            "Identification (Scroll of Wisdom)".into(),
            "Currency".into(),
            vec!["1 unidentified Rare item".into()],
            "1 Scroll of Wisdom".into(),
            Some("Vendor sells back wisdom scrolls when you have unidentified gear.".into()),
        ),
        // ─── Equipment ───────────────────────────────────────────────
        Recipe::new(
            "Quality flask".into(),
            "Equipment".into(),
            vec!["Flasks summing to 40% quality".into()],
            "1 Glassblower's Bauble".into(),
            Some("Sell several quality flasks together — the qualities are summed.".into()),
        ),
        Recipe::new(
            "Quality gem".into(),
            "Equipment".into(),
            vec!["Gems summing to 40% quality".into()],
            "1 Gemcutter's Prism".into(),
            None,
        ),
        // ─── Maps / endgame ─────────────────────────────────────────
        Recipe::new(
            "Chisels (map quality)".into(),
            "Maps".into(),
            vec!["Stone Hammer / Rock Breaker / Gavel of 20% quality + map".into()],
            "Quality applied to map".into(),
            Some("Use chisels on a waystone before running it for more loot.".into()),
        ),
        Recipe::new(
            "Portal Scroll".into(),
            "Misc".into(),
            vec!["1 Stack of Wisdom Scrolls (varies)".into()],
            "1 Portal Scroll".into(),
            Some("Less common in PoE 2 — most portals are free at waypoints.".into()),
        ),
    ]
}
