use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::use_app_state;

use super::context::{provide_notes_context, use_notes_context};
use super::editor::Editor;
use super::lightbox::Lightbox;
use super::modals::DeleteModal;
use super::sidebar::Sidebar;
use super::storage::delete_one;

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

    view! {
        <section class="flex gap-4 p-4 h-[calc(100vh-2.25rem)] min-h-[28rem]">
            <Sidebar/>
            <Editor/>
            <Show when=move || use_notes_context().pending_delete.get().is_some()>
                <DeleteModal cancel=cancel_delete confirm=confirm_delete/>
            </Show>
            <Show when=move || use_notes_context().viewing_image.get().is_some()>
                <Lightbox/>
            </Show>
        </section>
    }
}
