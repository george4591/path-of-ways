use leptos::task::spawn_local;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use super::storage::get_object_url;

/// Walk the rendered preview and replace every `<img src="image:ID">` with
/// the real blob URL by loading the blob from IndexedDB asynchronously.
pub fn resolve_image_urls(container: &web_sys::Element) {
    let Ok(imgs) = container.query_selector_all("img") else {
        return;
    };
    for i in 0..imgs.length() {
        let Some(node) = imgs.get(i) else { continue };
        let Ok(img) = node.dyn_into::<web_sys::HtmlImageElement>() else {
            continue;
        };
        let src = img.get_attribute("src").unwrap_or_default();
        let Some(id) = src.strip_prefix("image:") else { continue };
        let id = id.to_string();
        let img_clone = img.clone();
        spawn_local(async move {
            if let Some(url) = get_object_url(&id).await {
                img_clone.set_src(&url);
            }
        });
    }
}
