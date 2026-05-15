use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use crate::app_state::{create_blank_note, use_app_state, Page};
use crate::theme::use_theme;

/// True if the app is running inside Tauri (i.e. the `__TAURI__` global is present).
/// In a plain browser dev session (`trunk serve`) this returns false and the title
/// bar quietly skips its native-only bits.
fn is_tauri() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    js_sys::Reflect::get(&window, &"__TAURI__".into())
        .map(|value| !value.is_undefined() && !value.is_null())
        .unwrap_or(false)
}

/// Calls `window.__TAURI__.window.getCurrentWindow().<method>()` (which returns
/// a Promise). Errors are swallowed — there's no good UX for "minimize failed."
fn call_window_method(method: &str) {
    if !is_tauri() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) else {
        return;
    };
    let Ok(window_mod) = js_sys::Reflect::get(&tauri, &"window".into()) else {
        return;
    };
    let Ok(get_current) = js_sys::Reflect::get(&window_mod, &"getCurrentWindow".into()) else {
        return;
    };
    let Ok(get_current_fn) = get_current.dyn_into::<js_sys::Function>() else {
        return;
    };
    let Ok(current) = get_current_fn.call0(&wasm_bindgen::JsValue::NULL) else {
        return;
    };
    let Ok(method_value) = js_sys::Reflect::get(&current, &method.into()) else {
        return;
    };
    let Ok(method_fn) = method_value.dyn_into::<js_sys::Function>() else {
        return;
    };
    let _ = method_fn.call0(&current);
}

fn minimize() {
    call_window_method("minimize");
}

fn toggle_maximize() {
    call_window_method("toggleMaximize");
}

fn close() {
    call_window_method("close");
}

#[component]
pub fn TitleBar() -> impl IntoView {
    let in_tauri = is_tauri();

    view! {
        <div
            class="flex items-stretch h-9 bg-bg-elevated border-b border-border select-none"
            data-tauri-drag-region=""
        >
            <div
                class="flex items-center px-3 gap-2 text-sm text-fg-muted font-medium"
                data-tauri-drag-region=""
            >
                <span data-tauri-drag-region="">"Path of Ways"</span>
            </div>

            <FileMenu/>
            <ViewMenu/>

            <div class="flex-1" data-tauri-drag-region=""></div>

            {in_tauri.then(|| view! {
                <div class="flex items-stretch">
                    <button
                        class="w-11 inline-flex items-center justify-center text-fg-muted hover:bg-fg/10 hover:text-fg transition"
                        on:click=move |_| minimize()
                        title="Minimize"
                    >
                        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                            <path d="M0 5 H10" stroke="currentColor" stroke-width="1"/>
                        </svg>
                    </button>
                    <button
                        class="w-11 inline-flex items-center justify-center text-fg-muted hover:bg-fg/10 hover:text-fg transition"
                        on:click=move |_| toggle_maximize()
                        title="Maximize"
                    >
                        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                            <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1"/>
                        </svg>
                    </button>
                    <button
                        class="w-11 inline-flex items-center justify-center text-fg-muted hover:bg-red-600 hover:text-white transition"
                        on:click=move |_| close()
                        title="Close"
                    >
                        <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                            <path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" stroke-width="1"/>
                        </svg>
                    </button>
                </div>
            })}
        </div>
    }
}

/// Shared dropdown menu shell used by `FileMenu` and `ViewMenu`.
#[component]
fn MenuButton(
    #[prop(into)] label: String,
    children: ChildrenFn,
) -> impl IntoView {
    let (open, set_open) = signal(false);

    // Click-outside: when open, listen for any pointerdown that isn't inside
    // this menu and close it. We give the wrapper a unique class so the handler
    // can identify "outside."
    let close_menu = move || set_open.set(false);

    view! {
        <div class="relative flex items-stretch">
            <button
                class=move || {
                    let base = "px-3 inline-flex items-center text-sm text-fg-muted hover:bg-fg/10 hover:text-fg transition";
                    if open.get() { format!("{} bg-fg/10 text-fg", base) } else { base.to_string() }
                }
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_open.update(|v| *v = !*v);
                }
            >
                {label}
            </button>
            <Show when=move || open.get()>
                <div
                    class="absolute top-full left-0 z-50 min-w-[12rem] rounded-md border border-border bg-bg-elevated shadow-lg py-1 mt-px"
                    on:click=move |_| close_menu()
                >
                    {children()}
                </div>
                <div
                    class="fixed inset-0 z-40"
                    on:click=move |_| close_menu()
                ></div>
            </Show>
        </div>
    }
}

#[component]
fn MenuItem<F>(
    #[prop(into)] label: String,
    #[prop(into, optional)] shortcut: String,
    on_select: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button
            class="w-full flex items-center justify-between gap-6 px-3 py-1.5 text-sm text-fg hover:bg-accent hover:text-accent-fg transition text-left"
            on:click=move |_| on_select()
        >
            <span>{label}</span>
            {(!shortcut.is_empty()).then(|| view! {
                <span class="text-xs text-fg-muted">{shortcut}</span>
            })}
        </button>
    }
}

#[component]
fn MenuSeparator() -> impl IntoView {
    view! { <div class="my-1 border-t border-border"></div> }
}

#[component]
fn FileMenu() -> impl IntoView {
    let app = use_app_state();
    view! {
        <MenuButton label="File">
            <MenuItem
                label="New Note"
                shortcut="Ctrl+N"
                on_select=move || create_blank_note(app)
            />
            <MenuItem
                label="Quick Switcher"
                shortcut="Ctrl+K"
                on_select=move || app.set_show_quick_switcher.set(true)
            />
            <MenuSeparator/>
            <MenuItem
                label="Quit"
                on_select=move || close()
            />
        </MenuButton>
    }
}

#[component]
fn ViewMenu() -> impl IntoView {
    let app = use_app_state();
    let theme_ctx = use_theme();
    view! {
        <MenuButton label="View">
            <MenuItem label="Notes"    shortcut="1" on_select=move || app.set_page.set(Page::Notes)/>
            <MenuItem label="Campaign" shortcut="2" on_select=move || app.set_page.set(Page::Campaign)/>
            <MenuItem label="Bosses"   shortcut="3" on_select=move || app.set_page.set(Page::Bosses)/>
            <MenuItem label="Recipes"  shortcut="4" on_select=move || app.set_page.set(Page::Recipes)/>
            <MenuItem label="Links"    shortcut="5" on_select=move || app.set_page.set(Page::Links)/>
            <MenuSeparator/>
            <MenuItem
                label="Cycle Theme"
                on_select=move || theme_ctx.cycle()
            />
            <MenuItem
                label="Help & Shortcuts"
                shortcut="?"
                on_select=move || app.set_show_help.set(true)
            />
        </MenuButton>
    }
}
