use crate::bosses::Boss;
use crate::campaign::Zone;
use crate::notes::Note;
use crate::recipes::Recipe;

use super::links_loader::LinkRow;
use super::model::Result;

const MAX_NOTES: usize = 8;
const MAX_PER_GROUP: usize = 6;

/// Filter the available data sources by `query` (case-insensitive substring)
/// and return groups in display order. Empty groups are omitted.
pub fn compute_groups(
    notes: Vec<Note>,
    zones: Vec<Zone>,
    bosses: Vec<Boss>,
    recipes: Vec<Recipe>,
    links: Vec<LinkRow>,
    query: &str,
) -> Vec<(&'static str, Vec<Result>)> {
    let lowered = query.trim().to_lowercase();
    let needle = lowered.as_str();
    let mut groups: Vec<(&'static str, Vec<Result>)> = Vec::new();

    let notes_matches: Vec<Result> = notes
        .into_iter()
        .filter(|note| needle.is_empty() || note.matches_query(needle))
        .take(MAX_NOTES)
        .map(|note| Result::Note {
            id: note.id.clone(),
            title: note.display_title(),
        })
        .collect();
    push_group(&mut groups, "Notes", notes_matches);

    push_group(&mut groups, "Campaign zones", zone_matches(needle, &zones));
    push_group(&mut groups, "Bosses", boss_matches(needle, &bosses));
    push_group(&mut groups, "Recipes", recipe_matches(needle, recipes));
    push_group(&mut groups, "Links", link_matches(needle, links));

    groups
}

fn push_group(
    groups: &mut Vec<(&'static str, Vec<Result>)>,
    label: &'static str,
    items: Vec<Result>,
) {
    if !items.is_empty() {
        groups.push((label, items));
    }
}

fn zone_matches(needle: &str, zones: &[Zone]) -> Vec<Result> {
    let mut out = Vec::new();
    for zone in zones {
        if needle.is_empty() || zone.name.to_lowercase().contains(needle) {
            out.push(Result::Zone {
                name: zone.name.clone(),
                act_name: zone.act.clone(),
                zone_id: zone.id.clone(),
            });
            if out.len() >= MAX_PER_GROUP {
                return out;
            }
        }
    }
    out
}

fn boss_matches(needle: &str, bosses: &[Boss]) -> Vec<Result> {
    let mut out = Vec::new();
    for boss in bosses {
        if needle.is_empty()
            || boss.name.to_lowercase().contains(needle)
            || boss.zone.to_lowercase().contains(needle)
            || boss.category.to_lowercase().contains(needle)
        {
            out.push(Result::Boss {
                name: boss.name.clone(),
                zone_name: boss.zone.clone(),
                act_name: boss.category.clone(),
            });
            if out.len() >= MAX_PER_GROUP {
                return out;
            }
        }
    }
    out
}

fn recipe_matches(needle: &str, recipes: Vec<Recipe>) -> Vec<Result> {
    recipes
        .into_iter()
        .filter(|recipe| {
            needle.is_empty()
                || recipe.name.to_lowercase().contains(needle)
                || recipe.category.to_lowercase().contains(needle)
                || recipe.result.to_lowercase().contains(needle)
        })
        .take(MAX_PER_GROUP)
        .map(|recipe| Result::Recipe {
            name: recipe.name,
            category: recipe.category,
        })
        .collect()
}

fn link_matches(needle: &str, links: Vec<LinkRow>) -> Vec<Result> {
    links
        .into_iter()
        .filter(|link| {
            needle.is_empty()
                || link.title.to_lowercase().contains(needle)
                || link.url.to_lowercase().contains(needle)
                || link.description.to_lowercase().contains(needle)
        })
        .take(MAX_PER_GROUP)
        .map(|link| Result::Link {
            domain: link.domain().to_string(),
            url: link.url,
            title: link.title,
        })
        .collect()
}
