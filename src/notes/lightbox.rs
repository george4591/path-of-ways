use leptos::prelude::*;
use leptos::web_sys;

use crate::app_state::use_app_state;
use crate::keyboard::use_escape_key;

use super::context::use_notes_context;
use super::markdown::render_inline_md;

#[component]
pub fn Lightbox() -> impl IntoView {
    let app = use_app_state();
    let ctx = use_notes_context();
    let viewing_image = ctx.viewing_image;
    let set_viewing_image = ctx.set_viewing_image;

    use_escape_key(move || set_viewing_image.set(None));

    let current = Memo::new(move |_| {
        let id = app.selected_note_id.get()?;
        app.notes.get().into_iter().find(|note| note.id == id)
    });

    view! {
        <div
            class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-3 bg-black/85 backdrop-blur-sm cursor-pointer"
            on:click=move |_| set_viewing_image.set(None)
        >
            <button
                class="absolute top-4 right-4 w-10 h-10 rounded-md bg-bg-elevated text-fg border border-border hover:bg-fg hover:text-bg text-2xl leading-none transition"
                on:click=move |ev: web_sys::MouseEvent| {
                    ev.stop_propagation();
                    set_viewing_image.set(None);
                }
                title="Close"
            >
                "×"
            </button>
            <img
                src=move || viewing_image.get().unwrap_or_default()
                class="max-w-[92vw] max-h-[82vh] object-contain rounded-md shadow-2xl border border-border/60"
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            />
            <div
                class="text-sm text-fg-muted text-center max-w-[80vw] truncate"
                inner_html=move || render_inline_md(
                    &current.get().map(|note| note.display_title()).unwrap_or_default()
                )
            ></div>
            <div class="absolute bottom-4 left-1/2 -translate-x-1/2 text-xs text-fg-muted/70">
                "Click anywhere to close"
            </div>
        </div>
    }
}
