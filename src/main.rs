mod app_state;
mod buttons;
mod campaign;
mod db;
mod error_log;
mod external;
mod help;
mod icons;
mod images;
mod keyboard;
mod links;
mod modal;
mod notes;
mod recipes;
mod search;
mod theme;
mod titlebar;

use app_state::{provide_app_state, use_app_state, Page};
use campaign::CampaignTracker;
use error_log::{install_log_state, ErrorBanner};
use help::HelpModal;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::{use_location, use_navigate};
use leptos_router::path;
use links::Links;
use notes::{import_json, save_many, ImportModal, Notes};
use recipes::Recipes;
use search::QuickSwitcher;
use theme::provide_theme_context;
use titlebar::TitleBar;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

fn main() {
    error_log::install_panic_hook();
    external::install_link_interceptor();
    mount_to_body(|| view! { <App/> });
}

#[component]
fn App() -> impl IntoView {
    install_log_state();
    provide_theme_context();
    let app = provide_app_state();
    keyboard::install_global_shortcuts(app);

    // ─── Notes import flow (App-level) ───────────────────────────────────
    //
    // The hidden file input + onChange handler + ImportModal live here so
    // File > Import (in the title bar) works regardless of which tab is
    // currently active. The picker click is driven by `trigger_import_picker`
    // counter on AppState — File menu bumps it, this effect reacts.

    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |prev: Option<u32>| {
        let curr = app.trigger_import_picker.get();
        if prev.is_some() && prev != Some(curr) {
            if let Some(input) = file_input_ref.get_untracked() {
                input.click();
            }
        }
        curr
    });

    let on_file_change = move |ev: web_sys::Event| {
        let Some(target) = ev.target() else { return };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };
        // Clear the value so re-picking the same file fires `change` again.
        input.set_value("");
        let Ok(reader) = web_sys::FileReader::new() else {
            return;
        };
        let reader_clone = reader.clone();
        let set_pending_import = app.set_pending_import;
        let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let Ok(result) = reader_clone.result() else {
                return;
            };
            let Some(text) = result.as_string() else {
                return;
            };
            match import_json(&text) {
                Ok(list) => set_pending_import.set(Some(list)),
                Err(_) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.alert_with_message("Invalid notes JSON file.");
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        let _ = reader.read_as_text(&file);
        onload.forget();
    };

    let cancel_import = move || app.set_pending_import.set(None);

    let confirm_import = move || {
        let Some(incoming) = app.pending_import.get_untracked() else {
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
        app.set_pending_import.set(None);
    };

    let import_summary = move || {
        let Some(incoming) = app.pending_import.get() else {
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
        <Router>
            <TitleBar/>
            <main class="bg-bg-elevated">
                <RouteSync/>
                <Routes fallback=|| view! { <p class="text-fg-muted">"Page not found."</p> }>
                    <Route path=path!("/") view=Notes/>
                    <Route path=path!("/notes") view=Notes/>
                    <Route path=path!("/campaign") view=CampaignTracker/>
                    <Route path=path!("/recipes") view=Recipes/>
                    <Route path=path!("/links") view=Links/>
                </Routes>
                <Show when=move || app.show_quick_switcher.get()>
                    <QuickSwitcher/>
                </Show>
                <Show when=move || app.show_help.get()>
                    <HelpModal close=move || app.set_show_help.set(false)/>
                </Show>
                <Show when=move || app.pending_import.get().is_some()>
                    <ImportModal
                        summary=Signal::derive(import_summary)
                        cancel=cancel_import
                        confirm=confirm_import
                    />
                </Show>
            </main>
            <input
                node_ref=file_input_ref
                type="file"
                accept="application/json,.json"
                class="hidden"
                on:change=on_file_change
            />
            <ErrorBanner/>
        </Router>
    }
}

/// Keeps `AppState.page` in sync with the current URL in both directions:
/// route changes update the signal, signal changes drive navigation.
#[component]
fn RouteSync() -> impl IntoView {
    let app = use_app_state();
    let location = use_location();
    let navigate = use_navigate();

    // URL → signal
    Effect::new(move |_| {
        let path = location.pathname.get();
        let page = Page::from_route(&path);

        if app.page.get_untracked() != page {
            app.set_page.set(page);
        }
    });

    // signal → URL
    let navigate_owned = navigate.clone();
    Effect::new(move |_| {
        let page = app.page.get();
        let target = page.route();
        let current = location.pathname.get_untracked();

        if current != target {
            navigate_owned(target, Default::default());
        }
    });

    view! {}
}
