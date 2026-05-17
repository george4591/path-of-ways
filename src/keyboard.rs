use leptos::prelude::{on_cleanup, Set, Update};
use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::app_state::{create_blank_note, AppState, Page};

/// Attach a document-level `keydown` listener that calls `close` whenever
/// the user presses Escape. The listener is removed when the calling
/// component's reactive scope drops (i.e. when the modal unmounts).
pub fn use_escape_key<F>(close: F)
where
    F: Fn() + Copy + 'static,
{
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    let handler = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            close();
        }
    }) as Box<dyn FnMut(_)>);

    let handler_fn: js_sys::Function = handler.as_ref().unchecked_ref::<js_sys::Function>().clone();
    let _ = document.add_event_listener_with_callback("keydown", &handler_fn);

    let document_for_cleanup = document.clone();
    let handler_for_cleanup = handler_fn.clone();
    on_cleanup(move || {
        let _ = document_for_cleanup
            .remove_event_listener_with_callback("keydown", &handler_for_cleanup);
    });

    handler.forget();
}

/// Attach a document-level `keydown` listener that calls `confirm` when the
/// user presses Enter — typically used by modal shells so Enter saves an edit.
///
/// Behavior:
/// - Plain `Enter` inside a `<textarea>` is left alone (newline as expected).
/// - `Ctrl+Enter` works *anywhere*, including textareas, so multi-line forms
///   can still be saved without the mouse.
/// - Plain `Enter` anywhere else (inputs, focused buttons, no focus) → confirm.
/// - Combinations with `Alt`, `Shift`, or `Meta` are ignored to avoid clashing
///   with the editor / OS shortcuts.
///
/// The listener is cleaned up when the calling reactive scope drops.
pub fn use_enter_key<F>(confirm: F)
where
    F: Fn() + Copy + 'static,
{
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    let handler = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
        if ev.key() != "Enter" {
            return;
        }
        // Don't compete with editor shortcuts.
        if ev.alt_key() || ev.shift_key() || ev.meta_key() {
            return;
        }
        // Plain Enter inside a textarea must remain a newline — only Ctrl+Enter
        // is a save shortcut there.
        if active_is_textarea() && !ev.ctrl_key() {
            return;
        }
        ev.prevent_default();
        confirm();
    }) as Box<dyn FnMut(_)>);

    let handler_fn: js_sys::Function = handler.as_ref().unchecked_ref::<js_sys::Function>().clone();
    let _ = document.add_event_listener_with_callback("keydown", &handler_fn);

    let document_for_cleanup = document.clone();
    let handler_for_cleanup = handler_fn.clone();
    on_cleanup(move || {
        let _ = document_for_cleanup
            .remove_event_listener_with_callback("keydown", &handler_for_cleanup);
    });

    handler.forget();
}

fn active_is_textarea() -> bool {
    let Some(window) = web_sys::window() else { return false };
    let Some(doc) = window.document() else { return false };
    let Some(active) = doc.active_element() else { return false };
    active.tag_name().eq_ignore_ascii_case("TEXTAREA")
}

/// Attach a global keydown listener to the document for app-wide shortcuts.
/// The listener is leaked intentionally — App is the root component and
/// never unmounts, so cleanup isn't needed.
pub fn install_global_shortcuts(state: AppState) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    let handler = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
        // Ctrl/Cmd shortcuts work regardless of focus.
        if (ev.ctrl_key() || ev.meta_key()) && !ev.alt_key() && !ev.shift_key() {
            let key = ev.key().to_lowercase();
            if key == "n" {
                ev.prevent_default();
                create_blank_note(state);
                return;
            }
            if key == "k" {
                ev.prevent_default();
                state
                    .set_show_quick_switcher
                    .update(|open| *open = !*open);
                return;
            }
        }

        // Single-key shortcuts only fire when the user isn't typing.
        if is_typing_target() {
            return;
        }
        if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }

        match ev.key().as_str() {
            "1" => {
                ev.prevent_default();
                state.set_page.set(Page::Notes);
            }
            "2" => {
                ev.prevent_default();
                state.set_page.set(Page::Campaign);
            }
            "3" => {
                ev.prevent_default();
                state.set_page.set(Page::Recipes);
            }
            "4" => {
                ev.prevent_default();
                state.set_page.set(Page::Links);
            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    let _ = document.add_event_listener_with_callback(
        "keydown",
        handler.as_ref().unchecked_ref(),
    );
    handler.forget();
}

fn is_typing_target() -> bool {
    let Some(window) = web_sys::window() else { return false };
    let Some(doc) = window.document() else { return false };
    let Some(active) = doc.active_element() else { return false };
    let tag = active.tag_name().to_uppercase();
    tag == "INPUT" || tag == "TEXTAREA" || tag == "SELECT"
}
