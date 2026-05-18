use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::app_state::{create_blank_note, use_app_state, Page};
use crate::notes::{export_json, now_ms, trigger_download};
use crate::theme::use_theme;

fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|win| js_sys::Reflect::has(&win, &"__TAURI__".into()).ok())
        .unwrap_or(false)
}

fn call_window_method(method: &str) {
    if !is_tauri() {
        return;
    }

    let _ = try_call_window_method(method);
}

fn get_prop(obj: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
}

fn get_fn(obj: &JsValue, key: &str) -> Result<js_sys::Function, JsValue> {
    get_prop(obj, key)?.dyn_into::<js_sys::Function>()
}

// Calls `window.__TAURI__.window.getCurrentWindow().<method>()`
fn try_call_window_method(method: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let window_mod = JsValue::from(window);
    let tauri = get_prop(&window_mod, "__TAURI__")?;
    let tauri_window = get_prop(&tauri, "window")?;
    let current = get_fn(&tauri_window, "getCurrentWindow")?.call0(&JsValue::NULL)?;

    get_fn(&current, method)?.call0(&current)?;

    Ok(())
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
            class="flex items-stretch h-9 bg-bg border-b-2 border-border select-none"
            data-tauri-drag-region
        >
            <div
                class="flex items-center px-3 gap-2 text-sm text-fg-muted font-medium"
                data-tauri-drag-region
            >
                <AppIcon/>
                <span class="font-bold" data-tauri-drag-region>"Path of Ways"</span>
                <span
                    class="text-xs text-fg-muted"
                    data-tauri-drag-region
                    title="App version"
                >
                    {format!("[v{}]", env!("CARGO_PKG_VERSION"))}
                </span>
            </div>

            // Thin vertical divider between the app-identity block (icon +
            // title + version) and the action menus. Without this, the
            // title text and "File" / "View" blur into a single label
            // stripe.
            <div
                class="self-center w-px h-5 bg-border mx-1"
                data-tauri-drag-region
            ></div>

            <FileMenu/>
            <ViewMenu/>

            <div class="flex-1" data-tauri-drag-region></div>

            <PageTabs/>

            <div class="flex-1" data-tauri-drag-region></div>

            {if in_tauri {
                view! { <WindowControls /> }.into_any()
            } else {
                view! { <div class="w-[132px]" data-tauri-drag-region></div> }.into_any()
            }}
        </div>
    }
}

/// Small inline rendition of the forking-paths app icon — same silhouette as
/// `src-tauri/icons/source.svg` but stripped to a clean line drawing so it
/// scales cleanly at title-bar size and inherits the current theme's accent
/// color via `currentColor`.
#[component]
fn AppIcon() -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 24 24"
            class="w-5 h-5 text-accent shrink-0"
            attr:data-tauri-drag-region=""
        >
            <g
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M12 19 L12 6"/>
                <path d="M12 19 C 12 14 8 9 4 6"/>
                <path d="M12 19 C 12 14 16 9 20 6"/>
            </g>
            <circle cx="12" cy="19" r="2" fill="currentColor"/>
            <circle cx="12" cy="5"  r="1.5" fill="currentColor"/>
            <circle cx="4"  cy="5"  r="1.5" fill="currentColor"/>
            <circle cx="20" cy="5"  r="1.5" fill="currentColor"/>
        </svg>
    }
}

#[component]
fn PageTabs() -> impl IntoView {
    view! {
        <nav class="flex items-stretch">
            <NavLink target=Page::Notes    label="Notes"    title="Notes (1)"/>
            <NavLink target=Page::Campaign label="Campaign" title="Campaign (2)"/>
            <NavLink target=Page::Recipes  label="Recipes"  title="Recipes (3)"/>
            <NavLink target=Page::Links    label="Links"    title="Links (4)"/>
        </nav>
    }
}

/// Inactive tabs blend with the chrome (muted text + hover fill). The
/// active tab uses the theme accent for both its text and a 2px underline
/// pinned to the bottom edge of the title bar — the classic browser-tab
/// "you are here" marker. `relative` on the anchor lets the underline
/// position absolutely to the tab's own bounds.
#[component]
fn NavLink(
    target: Page,
    #[prop(into)] label: String,
    #[prop(into)] title: String,
) -> impl IntoView {
    let app = use_app_state();
    let is_active_for_class = move || app.page.get() == target;
    let is_active_for_show = move || app.page.get() == target;
    let class = move || {
        let base = "relative px-3 inline-flex items-center text-sm transition no-underline";
        if is_active_for_class() {
            format!("{} text-accent font-medium", base)
        } else {
            format!("{} text-fg-muted hover:bg-fg/10 hover:text-fg", base)
        }
    };
    view! {
        <A href=target.route() attr:class=class attr:title=title>
            {label}
            <Show when=is_active_for_show>
                <span class="absolute left-2 right-2 -bottom-px h-0.5 bg-accent pointer-events-none"/>
            </Show>
        </A>
    }
}

#[component]
fn WindowControls() -> impl IntoView {
    let base_btn_class = "w-11 inline-flex items-center justify-center text-fg-muted transition";

    view! {
        <div class="flex items-stretch">
            <button
                class=format!("{base_btn_class} hover:bg-fg/10 hover:text-fg")
                on:click=move |_| minimize()
                title="Minimize"
            >
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                    <path d="M0 5 H10" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>

            <button
                class=format!("{base_btn_class} hover:bg-fg/10 hover:text-fg")
                on:click=move |_| toggle_maximize()
                title="Maximize"
            >
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                    <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>

            <button
                class=format!("{base_btn_class} hover:bg-red-600 hover:text-white")
                on:click=move |_| close()
                title="Close"
            >
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                    <path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
        </div>
    }
}

#[component]
fn MenuButton(#[prop(into)] label: String, children: ChildrenFn) -> impl IntoView {
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
                // Use `bg-bg` (darker) instead of `bg-bg-elevated` so the
                // dropdown contrasts with the now-elevated main content area
                // — visually it reads as the title bar extending downward.
                // `shadow-2xl` gives a proper "floating above content" depth
                // cue so the panel doesn't blur into whatever's below it.
                <div
                    class="absolute top-full left-0 z-50 min-w-[12rem] rounded-md border border-border bg-bg shadow-2xl py-1 mt-px"
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

    let do_export = move || {
        let list = app.notes.get_untracked();
        let json = export_json(&list);
        let date = (now_ms() / 1000.0) as u64;
        trigger_download(&format!("path-of-ways-notes-{}.json", date), &json);
    };

    let do_import = move || {
        // Increments the counter App's Effect watches; click is fired there.
        app.set_trigger_import_picker
            .update(|n| *n = n.wrapping_add(1));
    };

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
                label="Export notes…"
                on_select=do_export
            />
            <MenuItem
                label="Import notes…"
                on_select=do_import
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
    let theme = use_theme();

    view! {
        <MenuButton label="View">
            <MenuItem label="Notes"    shortcut="1" on_select=move || app.set_page.set(Page::Notes)/>
            <MenuItem label="Campaign" shortcut="2" on_select=move || app.set_page.set(Page::Campaign)/>
            <MenuItem label="Recipes"  shortcut="3" on_select=move || app.set_page.set(Page::Recipes)/>
            <MenuItem label="Links"    shortcut="4" on_select=move || app.set_page.set(Page::Links)/>
            <MenuSeparator/>
            <MenuItem
                label="Cycle Theme"
                on_select=move || theme.cycle()
            />
            <MenuItem
                label="Help"
                on_select=move || app.set_show_help.set(true)
            />
        </MenuButton>
    }
}
