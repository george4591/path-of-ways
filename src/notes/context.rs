use leptos::prelude::*;

/// Cross-component signals scoped to the Notes page. Provided by the root
/// `Notes` component and consumed by `Sidebar`, `Editor`, and `Lightbox`.
///
/// Import/export state lives on the global `AppState` instead — those flows
/// can be triggered from the title bar's File menu regardless of which tab
/// is currently active, so they don't fit a Notes-only context.
#[derive(Clone, Copy)]
pub struct NotesContext {
    pub pending_delete: ReadSignal<Option<String>>,
    pub set_pending_delete: WriteSignal<Option<String>>,
    pub viewing_image: ReadSignal<Option<String>>,
    pub set_viewing_image: WriteSignal<Option<String>>,
}

pub fn provide_notes_context() -> NotesContext {
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (viewing_image, set_viewing_image) = signal::<Option<String>>(None);

    let ctx = NotesContext {
        pending_delete,
        set_pending_delete,
        viewing_image,
        set_viewing_image,
    };
    provide_context(ctx);
    ctx
}

pub fn use_notes_context() -> NotesContext {
    use_context::<NotesContext>().expect("NotesContext must be provided by the Notes root")
}
