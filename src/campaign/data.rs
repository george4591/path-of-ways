use super::model::Zone;

/// Default starter zones seeded into IndexedDB on first load. IDs are kept
/// stable (e.g. `act1.clearfell`) so existing ZoneProgress and Note.zone_id
/// links keep resolving after the static→dynamic migration.
///
/// Acts are just string labels on each zone — grouping happens by `act` field
/// at display time, sorted alphabetically. Pick act names that sort how you'd
/// like them to appear (e.g. add a numeric prefix if you want exact order).
pub fn seed_zones() -> Vec<Zone> {
    let now = js_sys::Date::now();
    let mut zones = vec![
        // Act 1
        z("act1.clearfell", "Act 1: The Riverbank", "Clearfell", true, false, None, None),
        z("act1.mud_burrow", "Act 1: The Riverbank", "Mud Burrow", false, false, Some("Skill gem"), Some("The Devourer")),
        z("act1.grelwood", "Act 1: The Riverbank", "The Grelwood", true, true, None, None),
        z("act1.red_vale", "Act 1: The Riverbank", "The Red Vale", true, false, None, None),
        z("act1.cemetery", "Act 1: The Riverbank", "Cemetery of the Eternals", true, false, None, None),
        z("act1.mausoleum", "Act 1: The Riverbank", "Mausoleum of the Praetor", false, true, None, None),
        z("act1.tomb_of_consort", "Act 1: The Riverbank", "Tomb of the Consort", false, false, None, Some("The Bloated Miller")),
        z("act1.hunting_grounds", "Act 1: The Riverbank", "The Hunting Grounds", true, true, None, None),
        z("act1.ogham_farmlands", "Act 1: The Riverbank", "Ogham Farmlands", true, false, None, None),
        z("act1.ogham_village", "Act 1: The Riverbank", "Ogham Village", false, false, Some("Skill gem"), None),
        z("act1.ogham_manor", "Act 1: The Riverbank", "Ogham Manor", false, true, None, None),
        z("act1.manor_ramparts", "Act 1: The Riverbank", "The Manor Ramparts", false, false, None, Some("Count Geonor")),
        // Act 2
        z("act2.vastiri_outskirts", "Act 2: The Vastiri Desert", "Vastiri Outskirts", true, false, None, None),
        z("act2.traitors_passage", "Act 2: The Vastiri Desert", "Traitor's Passage", false, false, None, None),
        z("act2.bone_pits", "Act 2: The Vastiri Desert", "The Bone Pits", true, true, None, None),
        z("act2.halani_gates", "Act 2: The Vastiri Desert", "The Halani Gates", false, false, None, Some("Jamanra")),
        z("act2.keth", "Act 2: The Vastiri Desert", "Keth", true, true, None, None),
        z("act2.lost_city", "Act 2: The Vastiri Desert", "The Lost City", false, false, None, None),
        z("act2.titan_grotto", "Act 2: The Vastiri Desert", "The Titan Grotto", false, false, None, Some("Zalmarath")),
        z("act2.valley_of_titans", "Act 2: The Vastiri Desert", "Valley of the Titans", true, true, None, None),
        z("act2.mawdun_quarry", "Act 2: The Vastiri Desert", "Mawdun Quarry", false, false, None, None),
        z("act2.mastodon_badlands", "Act 2: The Vastiri Desert", "Mastodon Badlands", true, false, None, None),
        // Act 3
        z("act3.sandswept_marsh", "Act 3: The Drowned City", "Sandswept Marsh", true, true, None, None),
        z("act3.infested_barrens", "Act 3: The Drowned City", "Infested Barrens", true, false, None, None),
        z("act3.azak_bog", "Act 3: The Drowned City", "The Azak Bog", false, false, None, None),
        z("act3.jungle_ruins", "Act 3: The Drowned City", "Jungle Ruins", true, true, None, None),
        z("act3.matlan_waterways", "Act 3: The Drowned City", "Matlan Waterways", true, false, None, None),
        z("act3.drowned_city", "Act 3: The Drowned City", "The Drowned City", true, false, None, None),
        z("act3.apex_of_filth", "Act 3: The Drowned City", "Apex of Filth", false, false, None, Some("Mighty Silverfist")),
        z("act3.utzaal", "Act 3: The Drowned City", "Utzaal", true, false, None, None),
        z("act3.aggorat", "Act 3: The Drowned City", "Aggorat", false, true, None, None),
        z("act3.temple_of_chaos", "Act 3: The Drowned City", "Temple of Chaos", false, false, None, Some("Doryani")),
        // Cruel Act 1
        z("act1c.clearfell", "Cruel Act 1", "Clearfell", true, false, None, None),
        z("act1c.grelwood", "Cruel Act 1", "The Grelwood", true, true, None, None),
        z("act1c.red_vale", "Cruel Act 1", "The Red Vale", true, false, None, None),
        z("act1c.manor_ramparts", "Cruel Act 1", "The Manor Ramparts", false, false, None, Some("Count Geonor")),
        // Cruel Act 2
        z("act2c.vastiri_outskirts", "Cruel Act 2", "Vastiri Outskirts", true, false, None, None),
        z("act2c.keth", "Cruel Act 2", "Keth", true, true, None, None),
        z("act2c.valley_of_titans", "Cruel Act 2", "Valley of the Titans", true, true, None, None),
        // Cruel Act 3
        z("act3c.sandswept_marsh", "Cruel Act 3", "Sandswept Marsh", true, true, None, None),
        z("act3c.matlan_waterways", "Cruel Act 3", "Matlan Waterways", true, false, None, None),
        z("act3c.temple_of_chaos", "Cruel Act 3", "Temple of Chaos", false, false, None, Some("Doryani")),
    ];
    // Stagger created_at so seed order is preserved when sorting by it.
    for (index, zone) in zones.iter_mut().enumerate() {
        zone.created_at = now + index as f64;
    }
    zones
}

fn z(
    id: &str,
    act: &str,
    name: &str,
    has_waypoint: bool,
    has_side_area: bool,
    quest_reward: Option<&str>,
    boss: Option<&str>,
) -> Zone {
    Zone {
        id: id.to_string(),
        act: act.to_string(),
        name: name.to_string(),
        has_waypoint,
        has_side_area,
        quest_reward: quest_reward.map(str::to_string),
        boss: boss.map(str::to_string),
        created_at: 0.0,
    }
}
