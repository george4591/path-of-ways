//! Surfaces panics and uncaught JS errors as in-app toasts.
//!
//! In a Tauri release build the devtools inspector is reachable (`Ctrl+Shift+I`),
//! but for everyday use we want errors to be visible *without* opening a debugger.
//! This module:
//!   1. Installs a panic hook that funnels every wasm panic into a shared signal
//!      (and still forwards to `console.error` via `console_error_panic_hook`).
//!   2. Attaches `window.onerror` / `window.onunhandledrejection` listeners that
//!      forward uncaught JS-side errors into the same signal.
//!   3. Provides an `<ErrorBanner/>` component that renders pending entries as
//!      a stack of dismissable toasts pinned to the bottom-right.
//!
//! The signal lives in a `RwSignal` stored in a `OnceCell` so the panic hook
//! (a `'static` closure with no Leptos context) can still push to it.

use std::cell::RefCell;
use std::panic::PanicHookInfo;
use std::sync::OnceLock;

use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
pub struct ErrorEntry {
    pub id: u64,
    pub kind: &'static str,
    pub message: String,
}

#[derive(Clone, Copy)]
struct ErrorLog {
    entries: RwSignal<Vec<ErrorEntry>>,
}

static LOG: OnceLock<ErrorLog> = OnceLock::new();

thread_local! {
    static NEXT_ID: RefCell<u64> = const { RefCell::new(1) };
}

fn next_id() -> u64 {
    NEXT_ID.with(|cell| {
        let mut id = cell.borrow_mut();
        let value = *id;
        *id = id.wrapping_add(1);
        value
    })
}

fn push_entry(kind: &'static str, message: String) {
    // Always mirror to console for the devtools inspector.
    web_sys::console::error_1(&format!("[{}] {}", kind, message).into());
    if let Some(log) = LOG.get() {
        log.entries.update(|list| {
            list.push(ErrorEntry {
                id: next_id(),
                kind,
                message,
            });
            // Cap retained errors so a runaway loop doesn't blow up memory.
            if list.len() > 32 {
                let drop = list.len() - 32;
                list.drain(0..drop);
            }
        });
    }
}

/// Install the panic hook + window error listeners. Call once, before mounting
/// the Leptos app — but *after* signals are usable (Leptos's runtime is created
/// at first `signal()` call inside a reactive owner, but `RwSignal::new` works
/// at module load too). We delay storing the signal until the App component
/// runs, via `install_log_state`.
pub fn install_panic_hook() {
    // Forward panics to the in-app banner AND to console.error.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info: &PanicHookInfo| {
        push_entry("panic", panic_message(info));
        default_hook(info);
    }));
    // Also forward via console_error_panic_hook so stack traces show up nicely.
    console_error_panic_hook::set_once();

    install_window_listeners();
}

fn panic_message(info: &PanicHookInfo) -> String {
    let payload = info.payload();
    let body = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    };
    match info.location() {
        Some(loc) => format!("{} at {}:{}:{}", body, loc.file(), loc.line(), loc.column()),
        None => body,
    }
}

fn install_window_listeners() {
    let Some(window) = web_sys::window() else {
        return;
    };

    // window.onerror — runtime errors from any script
    let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(|ev: web_sys::Event| {
        // ErrorEvent has a `.message` getter; fall back to event type if missing.
        let msg = ev
            .dyn_ref::<web_sys::ErrorEvent>()
            .map(|e| e.message())
            .unwrap_or_else(|| ev.type_());
        push_entry("error", msg);
    });
    let _ = window.add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref());
    on_error.forget();

    // window.onunhandledrejection — rejected promises with no catch
    let on_rej = Closure::<dyn FnMut(web_sys::Event)>::new(|ev: web_sys::Event| {
        let msg = ev
            .dyn_ref::<web_sys::PromiseRejectionEvent>()
            .map(|e| {
                e.reason()
                    .as_string()
                    .unwrap_or_else(|| format!("{:?}", e.reason()))
            })
            .unwrap_or_else(|| "unhandled rejection".to_string());
        push_entry("rejection", msg);
    });
    let _ = window
        .add_event_listener_with_callback("unhandledrejection", on_rej.as_ref().unchecked_ref());
    on_rej.forget();
}

/// Initialize the shared signal. Must be called inside a Leptos reactive owner
/// (i.e. from the root `App` component) so the signal lives for the app's
/// lifetime.
pub fn install_log_state() {
    let entries = RwSignal::new(Vec::<ErrorEntry>::new());
    let _ = LOG.set(ErrorLog { entries });
}

#[component]
pub fn ErrorBanner() -> impl IntoView {
    // If install_log_state hasn't run yet (it should have), render nothing.
    let Some(log) = LOG.get().copied() else {
        return view! { <div/> }.into_any();
    };

    let entries = log.entries;

    let dismiss = move |id: u64| {
        entries.update(|list| list.retain(|entry| entry.id != id));
    };
    let clear_all = move || entries.set(Vec::new());

    view! {
        <Show when=move || !entries.with(|list| list.is_empty())>
            <div class="fixed bottom-3 right-3 z-[1000] flex flex-col gap-2 max-w-[28rem] pointer-events-none">
                <For
                    each=move || entries.get()
                    key=|entry| entry.id
                    let:entry
                >
                    <div class="pointer-events-auto rounded-md border border-red-700 bg-red-950/95 text-red-50 shadow-lg p-3 text-sm">
                        <div class="flex items-start justify-between gap-3">
                            <div class="font-medium uppercase tracking-wide text-xs text-red-300">
                                {entry.kind}
                            </div>
                            <button
                                class="text-red-300 hover:text-white text-xs leading-none"
                                on:click={
                                    let id = entry.id;
                                    move |_| dismiss(id)
                                }
                                title="Dismiss"
                            >
                                "✕"
                            </button>
                        </div>
                        <pre class="whitespace-pre-wrap break-words mt-1 m-0 text-xs font-mono">
                            {entry.message}
                        </pre>
                    </div>
                </For>
                <Show when=move || entries.with(|list| list.len() > 1)>
                    <button
                        class="pointer-events-auto self-end text-xs text-red-300 hover:text-white underline"
                        on:click=move |_| clear_all()
                    >
                        "Dismiss all"
                    </button>
                </Show>
            </div>
        </Show>
    }
    .into_any()
}
