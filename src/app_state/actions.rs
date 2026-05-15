use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::notes::{save_one, Note, Template};

use super::page::Page;
use super::state::AppState;

/// Open the note linked to a campaign zone — creating one with the given
/// template if no zone-linked note exists yet. Always lands on the Notes page.
pub fn open_note_for_zone(
    state: AppState,
    zone_id: String,
    zone_title: String,
    template: Template,
) {
    if let Some(existing) = state
        .notes
        .get_untracked()
        .into_iter()
        .find(|note| note.zone_id.as_ref() == Some(&zone_id))
    {
        state.set_selected_note_id.set(Some(existing.id));
    } else {
        let mut note = Note::new_for_zone(zone_id, zone_title);
        template.apply(&mut note);
        let id = note.id.clone();
        let to_save = note.clone();
        state.set_notes.update(|list| list.push(note));
        state.set_selected_note_id.set(Some(id));
        spawn_local(async move {
            save_one(to_save).await;
        });
    }
    state.set_page.set(Page::Notes);
}

/// Select a note by id and switch to the Notes page.
pub fn open_note(state: AppState, id: String) {
    state.set_selected_note_id.set(Some(id));
    state.set_page.set(Page::Notes);
}

/// Create a blank note, select it, switch to Notes, and drop into edit mode.
/// Used by the Ctrl+N keyboard shortcut.
pub fn create_blank_note(state: AppState) {
    let note = Note::new_blank();
    let id = note.id.clone();
    let to_save = note.clone();
    state.set_notes.update(|list| list.push(note));
    state.set_selected_note_id.set(Some(id));
    state.set_page.set(Page::Notes);
    state.set_edit_mode.set(true);
    spawn_local(async move {
        save_one(to_save).await;
    });
}

/// Open a note by its title, matched case-insensitively. If no note matches,
/// create a new one with that title. Used for `[[wiki-link]]` navigation.
pub fn open_note_by_title(state: AppState, title: String) {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return;
    }

    let note_id = find_note_by_title(state, trimmed)
        .unwrap_or_else(|| create_note_with_title(state, trimmed));

    state.set_selected_note_id.set(Some(note_id));
    state.set_page.set(Page::Notes);
    state.set_edit_mode.set(false);
}

fn find_note_by_title(state: AppState, title: &str) -> Option<String> {
    state
        .notes
        .get_untracked()
        .iter()
        .find(|note| note.display_title().eq_ignore_ascii_case(title))
        .map(|note| note.id.clone())
}

fn create_note_with_title(state: AppState, title: &str) -> String {
    let mut note = Note::new_blank();
    note.title = title.to_string();
    let id = note.id.clone();

    let to_save = note.clone();
    state.set_notes.update(|list| list.push(note));

    spawn_local(async move {
        save_one(to_save).await;
    });

    id
}
