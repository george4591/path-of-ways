use leptos::web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = hljs, js_name = highlightElement, catch)]
    fn hljs_highlight_element(element: &web_sys::Element) -> Result<(), JsValue>;
}

pub fn highlight_within(container: &web_sys::Element) {
    let Ok(codes) = container.query_selector_all("pre code") else {
        return;
    };
    for i in 0..codes.length() {
        let Some(node) = codes.get(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let _ = hljs_highlight_element(&el);
    }
}
