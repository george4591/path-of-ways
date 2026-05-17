use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|win| js_sys::Reflect::has(&win, &"__TAURI__".into()).ok())
        .unwrap_or(false)
}

use wasm_bindgen::prelude::*;

pub fn open_external(url: &str) {
    if !is_tauri() {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
        return;
    }

    let run_tauri_open = || -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No Window"))?;

        let tauri = js_sys::Reflect::get(&window, &"__TAURI__".into())?;
        let core = js_sys::Reflect::get(&tauri, &"core".into())?;
        let invoke = js_sys::Reflect::get(&core, &"invoke".into())?;
        let invoke_fn = invoke.dyn_into::<js_sys::Function>()?;

        let args = js_sys::Object::new();
        js_sys::Reflect::set(&args, &"url".into(), &JsValue::from_str(url))?;

        invoke_fn.call2(
            &JsValue::NULL,
            &JsValue::from_str("plugin:opener|open_url"),
            &args.into(),
        )?;

        Ok(())
    };

    let _ = run_tauri_open();
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
