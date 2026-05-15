use leptos::prelude::*;

use super::model::Note;

/// Cross-component signals scoped to the Notes page. Provided by the root
/// `Notes` component and consumed by `Sidebar`, `Editor`, and `Lightbox`.
#[derive(Clone, Copy)]
pub struct NotesContext {
    pub pending_delete: ReadSignal<Option<String>>,
    pub set_pending_delete: WriteSignal<Option<String>>,
    pub pending_import: ReadSignal<Option<Vec<Note>>>,
    pub set_pending_import: WriteSignal<Option<Vec<Note>>>,
    pub viewing_image: ReadSignal<Option<String>>,
    pub set_viewing_image: WriteSignal<Option<String>>,
}

pub fn provide_notes_context() -> NotesContext {
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (pending_import, set_pending_import) = signal::<Option<Vec<Note>>>(None);
    let (viewing_image, set_viewing_image) = signal::<Option<String>>(None);

    let ctx = NotesContext {
        pending_delete,
        set_pending_delete,
        pending_import,
        set_pending_import,
        viewing_image,
        set_viewing_image,
    };
    provide_context(ctx);
    ctx
}

pub fn use_notes_context() -> NotesContext {
    use_context::<NotesContext>().expect("NotesContext must be provided by the Notes root")
}
