use leptos::web_sys;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Notes,
    Campaign,
    Bosses,
    Recipes,
    Links,
}

impl Page {
    pub fn as_str(self) -> &'static str {
        match self {
            Page::Notes => "notes",
            Page::Campaign => "campaign",
            Page::Bosses => "bosses",
            Page::Recipes => "recipes",
            Page::Links => "links",
        }
    }

    /// URL path for the page (e.g. `/notes`).
    pub fn route(self) -> &'static str {
        match self {
            Page::Notes => "/notes",
            Page::Campaign => "/campaign",
            Page::Bosses => "/bosses",
            Page::Recipes => "/recipes",
            Page::Links => "/links",
        }
    }

    /// Map a URL path to a Page. `/` and unknown paths fall back to Notes.
    pub fn from_route(pathname: &str) -> Self {
        let trimmed = pathname.trim_start_matches('/');
        match trimmed {
            "campaign" => Page::Campaign,
            "bosses" => Page::Bosses,
            "recipes" => Page::Recipes,
            "links" => Page::Links,
            _ => Page::Notes,
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "notes" => Page::Notes,
            "campaign" => Page::Campaign,
            "bosses" => Page::Bosses,
            "recipes" => Page::Recipes,
            "links" => Page::Links,
            _ => return None,
        })
    }
}

// ─── localStorage persistence ────────────────────────────────────────────

const LS_PAGE: &str = "last_page";
const LS_SELECTED_NOTE: &str = "last_selected_note_id";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub(super) fn read_last_page() -> Option<Page> {
    let stored = local_storage()?.get_item(LS_PAGE).ok().flatten()?;
    Page::from_str(&stored)
}

pub(super) fn write_last_page(page: Page) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(LS_PAGE, page.as_str());
    }
}

pub(super) fn read_last_selected_id() -> Option<String> {
    local_storage()?.get_item(LS_SELECTED_NOTE).ok().flatten()
}

pub(super) fn write_last_selected_id(id: Option<&str>) {
    if let Some(storage) = local_storage() {
        match id {
            Some(id) if !id.is_empty() => {
                let _ = storage.set_item(LS_SELECTED_NOTE, id);
            }
            _ => {
                let _ = storage.remove_item(LS_SELECTED_NOTE);
            }
        }
    }
}
