use super::model::{ChecklistItem, Zone};

/// Default starter zones seeded into IndexedDB on first load.
///
/// Sourced from the Path of Exile 2 wiki Locations page — 4 main acts plus
/// three smaller "interlude" acts that fill in between/after. Cruel difficulty
/// no longer exists in the current game and is intentionally absent here.
///
/// Each zone ships with an **empty checklist and no tags**. Per-zone metadata
/// (waypoints, skill gem drops, bosses, etc.) isn't on the wiki's Locations
/// page in a structured form, so this seed avoids fabricating it. Users add
/// their own checklist items + tags via the Manage UI as they play.
///
/// IDs follow the `<act-prefix>.<zone_slug>` convention. Stable IDs mean
/// ZoneProgress and any Note.zone_id link survives schema bumps.
pub fn seed_zones() -> Vec<Zone> {
    let now = js_sys::Date::now();
    let mut zones = vec![
        // ─── Act 1: Island of Ogham ───────────────────────────────────────
        z("act1.riverbank",          "Act 1: Island of Ogham", "The Riverbank"),
        z("act1.clearfell",          "Act 1: Island of Ogham", "Clearfell"),
        z("act1.clearfell_encamp",   "Act 1: Island of Ogham", "Clearfell Encampment"),
        z("act1.mud_burrow",         "Act 1: Island of Ogham", "The Mud Burrow"),
        z("act1.grelwood",           "Act 1: Island of Ogham", "The Grelwood"),
        z("act1.red_vale",           "Act 1: Island of Ogham", "The Red Vale"),
        z("act1.grim_tangle",        "Act 1: Island of Ogham", "The Grim Tangle"),
        z("act1.cemetery",           "Act 1: Island of Ogham", "Cemetery of the Eternals"),
        z("act1.freythorn",          "Act 1: Island of Ogham", "Freythorn"),
        z("act1.mausoleum",          "Act 1: Island of Ogham", "Mausoleum of the Praetor"),
        z("act1.hunting_grounds",    "Act 1: Island of Ogham", "The Hunting Grounds"),
        z("act1.ogham_village",      "Act 1: Island of Ogham", "Ogham Village"),
        z("act1.ogham_manor",        "Act 1: Island of Ogham", "Ogham Manor"),
        z("act1.ogham_farmlands",    "Act 1: Island of Ogham", "Ogham Farmlands"),
        z("act1.manor_ramparts",     "Act 1: Island of Ogham", "The Manor Ramparts"),
        z("act1.tomb_of_consort",    "Act 1: Island of Ogham", "Tomb of the Consort"),
        // ─── Act 2: The Vastiri Desert ────────────────────────────────────
        z("act2.vastiri_outskirts",  "Act 2: The Vastiri Desert", "Vastiri Outskirts"),
        z("act2.ardura_caravan",     "Act 2: The Vastiri Desert", "The Ardura Caravan"),
        z("act2.mawdun_quarry",      "Act 2: The Vastiri Desert", "Mawdun Quarry"),
        z("act2.mawdun_mine",        "Act 2: The Vastiri Desert", "Mawdun Mine"),
        z("act2.halani_gates",       "Act 2: The Vastiri Desert", "The Halani Gates"),
        z("act2.traitors_passage",   "Act 2: The Vastiri Desert", "Traitor's Passage"),
        z("act2.keth",               "Act 2: The Vastiri Desert", "Keth"),
        z("act2.lost_city",          "Act 2: The Vastiri Desert", "The Lost City"),
        z("act2.keth_sanctum",       "Act 2: The Vastiri Desert", "The Keth Sanctum"),
        z("act2.heart_of_keth",      "Act 2: The Vastiri Desert", "The Heart of Keth"),
        z("act2.valley_of_titans",   "Act 2: The Vastiri Desert", "Valley of the Titans"),
        z("act2.titan_grotto",       "Act 2: The Vastiri Desert", "The Titan Grotto"),
        z("act2.mastodon_badlands",  "Act 2: The Vastiri Desert", "The Mastodon Badlands"),
        z("act2.bone_pits",          "Act 2: The Vastiri Desert", "The Bone Pits"),
        z("act2.deshar",             "Act 2: The Vastiri Desert", "Deshar"),
        z("act2.path_of_mourning",   "Act 2: The Vastiri Desert", "Path of Mourning"),
        z("act2.dreadnought",        "Act 2: The Vastiri Desert", "The Dreadnought"),
        z("act2.dreadnought_van",    "Act 2: The Vastiri Desert", "The Dreadnought Vanguard"),
        z("act2.spires_of_deshar",   "Act 2: The Vastiri Desert", "The Spires of Deshar"),
        z("act2.buried_shrines",     "Act 2: The Vastiri Desert", "Buried Shrines"),
        // ─── Act 3: The Drowned City ──────────────────────────────────────
        z("act3.sandswept_marsh",    "Act 3: The Drowned City", "Sandswept Marsh"),
        z("act3.ziggurat_encamp",    "Act 3: The Drowned City", "Ziggurat Encampment"),
        z("act3.infested_barrens",   "Act 3: The Drowned City", "Infested Barrens"),
        z("act3.jungle_ruins",       "Act 3: The Drowned City", "Jungle Ruins"),
        z("act3.matlan_waterways",   "Act 3: The Drowned City", "The Matlan Waterways"),
        z("act3.chimeral_wetlands",  "Act 3: The Drowned City", "Chimeral Wetlands"),
        z("act3.jiquani_machin",     "Act 3: The Drowned City", "Jiquani's Machinarium"),
        z("act3.jiquani_sanctum",    "Act 3: The Drowned City", "Jiquani's Sanctum"),
        z("act3.drowned_city",       "Act 3: The Drowned City", "The Drowned City"),
        z("act3.apex_of_filth",      "Act 3: The Drowned City", "Apex of Filth"),
        z("act3.temple_of_kopec",    "Act 3: The Drowned City", "Temple of Kopec"),
        z("act3.utzaal",             "Act 3: The Drowned City", "Utzaal"),
        z("act3.aggorat",            "Act 3: The Drowned City", "Aggorat"),
        z("act3.black_chambers",     "Act 3: The Drowned City", "The Black Chambers"),
        z("act3.venom_crypts",       "Act 3: The Drowned City", "The Venom Crypts"),
        z("act3.azak_bog",           "Act 3: The Drowned City", "The Azak Bog"),
        z("act3.molten_vault",       "Act 3: The Drowned City", "The Molten Vault"),
        z("act3.temple_of_chaos",    "Act 3: The Drowned City", "The Temple of Chaos"),
        // ─── Act 4: Karui Archipelago ─────────────────────────────────────
        z("act4.kingsmarch",         "Act 4: Karui Archipelago", "Kingsmarch"),
        z("act4.abandoned_prison",   "Act 4: Karui Archipelago", "Abandoned Prison"),
        z("act4.arastas",            "Act 4: Karui Archipelago", "Arastas"),
        z("act4.eye_of_hinekora",    "Act 4: Karui Archipelago", "Eye of Hinekora"),
        z("act4.isle_of_kin",        "Act 4: Karui Archipelago", "Isle of Kin"),
        z("act4.kedge_bay",          "Act 4: Karui Archipelago", "Kedge Bay"),
        z("act4.ngakanu",            "Act 4: Karui Archipelago", "Ngakanu"),
        z("act4.plunders_point",     "Act 4: Karui Archipelago", "Plunder's Point"),
        z("act4.shoreline_hideout",  "Act 4: Karui Archipelago", "Shoreline Hideout"),
        z("act4.whakapanu_island",   "Act 4: Karui Archipelago", "Whakapanu Island"),
        z("act4.singing_caverns",    "Act 4: Karui Archipelago", "Singing Caverns"),
        z("act4.solitary_conf",      "Act 4: Karui Archipelago", "Solitary Confinement"),
        z("act4.shrike_island",      "Act 4: Karui Archipelago", "Shrike Island"),
        z("act4.excavation",         "Act 4: Karui Archipelago", "The Excavation"),
        z("act4.heart_of_tribe",     "Act 4: Karui Archipelago", "Heart of the Tribe"),
        z("act4.volcanic_warrens",   "Act 4: Karui Archipelago", "Volcanic Warrens"),
        z("act4.halls_of_dead",      "Act 4: Karui Archipelago", "Halls of the Dead"),
        z("act4.journeys_end",       "Act 4: Karui Archipelago", "Journey's End"),
        // ─── Interlude: Mount Kriar ───────────────────────────────────────
        z("intMK.glade",             "Interlude: Mount Kriar", "The Glade"),
        z("intMK.ashen_forest",      "Interlude: Mount Kriar", "Ashen Forest"),
        z("intMK.kriar_village",     "Interlude: Mount Kriar", "Kriar Village"),
        z("intMK.glacial_tarn",      "Interlude: Mount Kriar", "Glacial Tarn"),
        z("intMK.howling_caves",     "Interlude: Mount Kriar", "Howling Caves"),
        z("intMK.kriar_peaks",       "Interlude: Mount Kriar", "Kriar Peaks"),
        z("intMK.etched_ravine",     "Interlude: Mount Kriar", "Etched Ravine"),
        z("intMK.cuachic_vault",     "Interlude: Mount Kriar", "The Cuachic Vault"),
        // ─── Interlude: Ogham ─────────────────────────────────────────────
        z("intOG.refuge",            "Interlude: Ogham", "The Refuge"),
        z("intOG.scorched_farmland", "Interlude: Ogham", "Scorched Farmlands"),
        z("intOG.stones_of_serle",   "Interlude: Ogham", "Stones of Serle"),
        z("intOG.blackwood",         "Interlude: Ogham", "The Blackwood"),
        z("intOG.holten",            "Interlude: Ogham", "Holten"),
        z("intOG.wolvenhold",        "Interlude: Ogham", "Wolvenhold"),
        z("intOG.holten_estate",     "Interlude: Ogham", "Holten Estate"),
        // ─── Interlude: Vastiri ───────────────────────────────────────────
        z("intVA.khari_bazaar",      "Interlude: Vastiri", "The Khari Bazaar"),
        z("intVA.khari_crossing",    "Interlude: Vastiri", "The Khari Crossing"),
        z("intVA.pools_of_khatal",   "Interlude: Vastiri", "Pools of Khatal"),
        z("intVA.sel_khari",         "Interlude: Vastiri", "Sel Khari Sanctuary"),
        z("intVA.galai_gates",       "Interlude: Vastiri", "The Galai Gates"),
        z("intVA.qimah",             "Interlude: Vastiri", "Qimah"),
        z("intVA.qimah_reservoir",   "Interlude: Vastiri", "Qimah Reservoir"),
    ];
    // Stagger created_at so insertion order is preserved when sorting by it
    // within each act group.
    for (index, zone) in zones.iter_mut().enumerate() {
        zone.created_at = now + index as f64;
    }
    zones
}

fn z(id: &str, act: &str, name: &str) -> Zone {
    Zone {
        id: id.to_string(),
        act: act.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        checklist: Vec::<ChecklistItem>::new(),
        created_at: 0.0,
    }
}
