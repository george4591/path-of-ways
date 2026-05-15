use super::model::Boss;

/// Default starter bosses seeded into IndexedDB on first load. After seeding,
/// bosses live entirely in the database and the user manages them through the
/// UI (add / edit / delete).
pub fn seed_bosses() -> Vec<Boss> {
    let now = js_sys::Date::now();
    let mut bosses = vec![
        b("The Devourer", "Act 1", "Mud Burrow", ""),
        b("The Bloated Miller", "Act 1", "Tomb of the Consort", ""),
        b("Count Geonor", "Act 1", "The Manor Ramparts", "Act 1 final boss."),
        b("Jamanra", "Act 2", "The Halani Gates", ""),
        b("Zalmarath", "Act 2", "The Titan Grotto", ""),
        b("Mighty Silverfist", "Act 3", "Apex of Filth", ""),
        b("Doryani", "Act 3", "Temple of Chaos", "Act 3 final boss."),
        b("Count Geonor", "Cruel Act 1", "The Manor Ramparts", "Cruel difficulty repeat."),
        b("Doryani", "Cruel Act 3", "Temple of Chaos", "Cruel difficulty repeat."),
    ];
    for (index, boss) in bosses.iter_mut().enumerate() {
        boss.created_at = now + index as f64;
    }
    bosses
}

fn b(name: &str, category: &str, zone: &str, description: &str) -> Boss {
    Boss {
        id: format!("seed_boss_{}_{}", category, name)
            .to_lowercase()
            .replace(' ', "_"),
        name: name.to_string(),
        category: category.to_string(),
        zone: zone.to_string(),
        description: description.to_string(),
        created_at: 0.0,
    }
}
