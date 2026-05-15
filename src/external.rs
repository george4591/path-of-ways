//! Opens external URLs in the user's OS-default browser.
//!
//! Tauri's webview won't honour `window.open` for `http(s)://` URLs the way a
//! real browser does, so we route external URLs through `tauri-plugin-opener`
//! (`window.__TAURI__.core.invoke('plugin:opener|open_url', { url })`). When
//! running outside Tauri (e.g. `trunk serve` for development) we fall back to
//! `window.open(url, "_blank")`.
//!
//! Also installs a document-level click interceptor (`install_link_interceptor`)
//! that catches clicks on any rendered `<a href="http(s)://...">` anchor and
//! routes them through `open_external`. Without it, clicking a link inside a
//! markdown preview would do nothing (or worse — navigate the webview away).

use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

/// True if running inside Tauri (i.e. `window.__TAURI__` exists).
fn is_tauri() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    js_sys::Reflect::get(&window, &"__TAURI__".into())
        .map(|value| !value.is_undefined() && !value.is_null())
        .unwrap_or(false)
}

/// Open `url` in the OS default browser (or new browser tab if running outside
/// Tauri). Errors are swallowed — there's nothing useful we can do if it fails.
pub fn open_external(url: &str) {
    if !is_tauri() {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
        return;
    }

    // window.__TAURI__.core.invoke('plugin:opener|open_url', { url })
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) else {
        return;
    };
    let Ok(core) = js_sys::Reflect::get(&tauri, &"core".into()) else {
        return;
    };
    let Ok(invoke) = js_sys::Reflect::get(&core, &"invoke".into()) else {
        return;
    };
    let Ok(invoke_fn) = invoke.dyn_into::<js_sys::Function>() else {
        return;
    };
    let args = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&args, &"url".into(), &JsValue::from_str(url));
    let _ = invoke_fn.call2(
        &JsValue::NULL,
        &JsValue::from_str("plugin:opener|open_url"),
        &args.into(),
    );
}

/// Install a single document-level click handler that intercepts clicks on
/// anchors whose href starts with `http://` or `https://`, calls
/// `prevent_default`, and routes the URL through `open_external`. Internal
/// anchors (relative paths, `note:` schemes, `mailto:`, etc.) are left alone.
///
/// Call once at app startup, before mounting.
pub fn install_link_interceptor() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    let on_click = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|ev: web_sys::MouseEvent| {
        // Don't hijack ctrl/shift/middle-click — let the user explicitly choose.
        // (In a Tauri app these still won't navigate, but leaving them is harmless.)
        if ev.ctrl_key() || ev.shift_key() || ev.meta_key() || ev.button() != 0 {
            return;
        }
        let Some(target) = ev.target() else { return };
        let Ok(start) = target.dyn_into::<web_sys::Element>() else {
            return;
        };

        // Walk up to find the nearest <a> ancestor.
        let mut current = Some(start);
        while let Some(el) = current {
            if el.tag_name().eq_ignore_ascii_case("A") {
                let href = el.get_attribute("href").unwrap_or_default();
                if href.starts_with("http://") || href.starts_with("https://") {
                    ev.prevent_default();
                    open_external(&href);
                }
                return;
            }
            current = el.parent_element();
        }
    });

    let _ = document.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref());
    on_click.forget();
}
