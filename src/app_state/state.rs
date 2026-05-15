use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::images::{extract_image_ids, gc_orphans};
use crate::notes::{load_notes, Note};

use super::page::{
    read_last_page, read_last_selected_id, write_last_page, write_last_selected_id, Page,
};

#[derive(Clone, Copy)]
pub struct AppState {
    pub page: ReadSignal<Page>,
    pub set_page: WriteSignal<Page>,
    pub notes: ReadSignal<Vec<Note>>,
    pub set_notes: WriteSignal<Vec<Note>>,
    pub selected_note_id: ReadSignal<Option<String>>,
    pub set_selected_note_id: WriteSignal<Option<String>>,
    pub edit_mode: ReadSignal<bool>,
    pub set_edit_mode: WriteSignal<bool>,
    pub show_quick_switcher: ReadSignal<bool>,
    pub set_show_quick_switcher: WriteSignal<bool>,
    pub show_help: ReadSignal<bool>,
    pub set_show_help: WriteSignal<bool>,
}

pub fn provide_app_state() -> AppState {
    let initial_page = read_last_page().unwrap_or(Page::Notes);
    let (page, set_page) = signal(initial_page);
    let (notes, set_notes) = signal(Vec::<Note>::new());
    let (selected_note_id, set_selected_note_id) = signal(None::<String>);
    let (edit_mode, set_edit_mode) = signal(false);
    let (show_quick_switcher, set_show_quick_switcher) = signal(false);
    let (show_help, set_show_help) = signal(false);

    let stored_id = read_last_selected_id();
    spawn_local(async move {
        let list = load_notes().await;
        let target = stored_id
            .filter(|id| list.iter().any(|note| &note.id == id))
            .or_else(|| list.first().map(|note| note.id.clone()));
        let used: HashSet<String> = list
            .iter()
            .flat_map(|note| extract_image_ids(&note.body))
            .collect();
        set_notes.set(list);
        set_selected_note_id.set(target);
        let _ = gc_orphans(used).await;
    });

    Effect::new(move |prev: Option<Page>| {
        let current = page.get();
        if prev.is_some() {
            write_last_page(current);
        }
        current
    });
    Effect::new(move |prev: Option<Option<String>>| {
        let current = selected_note_id.get();
        if prev.is_some() {
            write_last_selected_id(current.as_deref());
        }
        current
    });

    let state = AppState {
        page,
        set_page,
        notes,
        set_notes,
        selected_note_id,
        set_selected_note_id,
        edit_mode,
        set_edit_mode,
        show_quick_switcher,
        set_show_quick_switcher,
        show_help,
        set_show_help,
    };
    provide_context(state);
    state
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>().expect("AppState must be provided at the root")
}
