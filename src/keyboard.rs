use leptos::ev;
use leptos::prelude::{window_event_listener, Set, Update};
use leptos::web_sys;

use crate::app_state::{create_blank_note, AppState, Page};

pub fn use_escape_key<F>(close: F)
where
    F: Fn() + Copy + 'static,
{
    window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            close();
        }
    });
}

pub fn use_enter_key<F>(confirm: F)
where
    F: Fn() + Copy + 'static,
{
    window_event_listener(ev::keydown, move |ev| {
        if ev.key() != "Enter" {
            return;
        }

        if ev.alt_key() || ev.shift_key() || ev.meta_key() {
            return;
        }

        if active_is_textarea() && !ev.ctrl_key() {
            return;
        }

        ev.prevent_default();
        confirm();
    });
}

fn active_is_textarea() -> bool {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.active_element())
        .map(|active| active.tag_name().eq_ignore_ascii_case("TEXTAREA"))
        .unwrap_or(false)
}

pub fn install_global_shortcuts(state: AppState) {
    // Leptos handles attaching this to the window cleanly.
    // Since this is called once at root initialization, it naturally stays alive
    // for the entire life of the app without needing manual leak methods.
    window_event_listener(ev::keydown, move |ev| {
        if (ev.ctrl_key() || ev.meta_key()) && !ev.alt_key() && !ev.shift_key() {
            let key = ev.key().to_lowercase();
            match key.as_str() {
                "n" => {
                    ev.prevent_default();
                    create_blank_note(state);
                    return;
                }
                "k" => {
                    ev.prevent_default();
                    state.set_show_quick_switcher.update(|open| *open = !*open);
                    return;
                }
                _ => {}
            }
        }

        if is_typing_target() || ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }

        if let Some(target_page) = match ev.key().as_str() {
            "1" => Some(Page::Notes),
            "2" => Some(Page::Campaign),
            "3" => Some(Page::Recipes),
            "4" => Some(Page::Links),
            _ => None,
        } {
            ev.prevent_default();
            state.set_page.set(target_page);
        }
    });
}

fn is_typing_target() -> bool {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.active_element())
        .map(|active| {
            let tag = active.tag_name().to_lowercase();
            matches!(tag.as_str(), "input" | "textarea" | "select")
        })
        .unwrap_or(false)
}
