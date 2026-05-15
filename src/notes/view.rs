use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::use_app_state;

use super::context::{provide_notes_context, use_notes_context};
use super::editor::Editor;
use super::lightbox::Lightbox;
use super::modals::{DeleteModal, ImportModal};
use super::sidebar::Sidebar;
use super::storage::{delete_one, save_many};

#[component]
pub fn Notes() -> impl IntoView {
    let app = use_app_state();
    let ctx = provide_notes_context();

    let cancel_delete = move || ctx.set_pending_delete.set(None);

    let confirm_delete = move || {
        let Some(id) = ctx.pending_delete.get_untracked() else {
            return;
        };
        app.set_notes
            .update(|list| list.retain(|note| note.id != id));
        let next = app.notes.get_untracked().first().map(|note| note.id.clone());
        app.set_selected_note_id.set(next);
        ctx.set_pending_delete.set(None);
        spawn_local(async move {
            delete_one(id).await;
        });
    };

    let cancel_import = move || ctx.set_pending_import.set(None);

    let confirm_import = move || {
        let Some(incoming) = ctx.pending_import.get_untracked() else {
            return;
        };
        let mut merged = app.notes.get_untracked();
        let mut to_save = Vec::with_capacity(incoming.len());
        for inc in incoming {
            if let Some(pos) = merged.iter().position(|note| note.id == inc.id) {
                merged[pos] = inc.clone();
            } else {
                merged.push(inc.clone());
            }
            to_save.push(inc);
        }
        app.set_notes.set(merged);
        spawn_local(async move {
            save_many(to_save).await;
        });
        ctx.set_pending_import.set(None);
    };

    let import_summary = move || {
        let Some(incoming) = ctx.pending_import.get() else {
            return (0usize, 0usize);
        };
        let existing = app.notes.get();
        let mut new_count = 0;
        let mut update_count = 0;
        for inc in &incoming {
            if existing.iter().any(|note| note.id == inc.id) {
                update_count += 1;
            } else {
                new_count += 1;
            }
        }
        (new_count, update_count)
    };

    view! {
        <section class="flex gap-4 rounded-xl border border-border bg-bg-elevated p-4 h-[calc(100vh-7.75rem)] min-h-[28rem]">
            <Sidebar/>
            <Editor/>
            <Show when=move || use_notes_context().pending_delete.get().is_some()>
                <DeleteModal cancel=cancel_delete confirm=confirm_delete/>
            </Show>
            <Show when=move || use_notes_context().pending_import.get().is_some()>
                <ImportModal
                    summary=Signal::derive(import_summary)
                    cancel=cancel_import
                    confirm=confirm_import
                />
            </Show>
            <Show when=move || use_notes_context().viewing_image.get().is_some()>
                <Lightbox/>
            </Show>
        </section>
    }
}
