use leptos::prelude::*;
use leptos::web_sys;

use crate::keyboard::{use_enter_key, use_escape_key};

/// Shared modal chrome: full-screen backdrop that dismisses on outside click /
/// Escape, plus a centered panel that swallows its own clicks. `panel_class`
/// supplies sizing or scroll behavior (e.g. `"max-w-md"`,
/// `"max-w-3xl max-h-[85vh] overflow-auto"`).
///
/// Both `cancel` (Escape) and `confirm` (Enter / Ctrl+Enter) are wired up
/// for keyboard parity — every modal in the app accepts the same two
/// callbacks, so users can keyboard through them consistently.
#[component]
pub fn ModalShell<C, K>(
    cancel: C,
    confirm: K,
    #[prop(optional, into)] panel_class: String,
    children: Children,
) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    use_escape_key(cancel);
    use_enter_key(confirm);

    let panel = format!(
        "rounded-xl border border-border bg-bg-elevated p-6 w-full mx-4 shadow-2xl {}",
        if panel_class.is_empty() { "max-w-sm" } else { panel_class.as_str() }
    );

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
            on:click=move |_| cancel()
        >
            <div
                class=panel
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                {children()}
            </div>
        </div>
    }
}
