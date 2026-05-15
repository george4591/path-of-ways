use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    Poe2,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::Poe2 => "poe2",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::Poe2 => "PoE 2",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Poe2,
            Theme::Poe2 => Theme::Light,
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "dark" => Theme::Dark,
            "poe2" => Theme::Poe2,
            _ => Theme::Light,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: ReadSignal<Theme>,
    set_theme: WriteSignal<Theme>,
}

impl ThemeContext {
    pub fn cycle(&self) {
        self.set_theme.update(|theme| *theme = theme.next());
    }
}

pub fn provide_theme_context() {
    let initial = current_html_theme().unwrap_or(Theme::Light);
    let (theme, set_theme) = signal(initial);

    Effect::new(move |_| {
        let current_theme = theme.get().as_str();

        document_element().and_then(|html| html.set_attribute("data-theme", current_theme).ok());
        local_storage().and_then(|storage| storage.set_item("theme", current_theme).ok());
    });

    provide_context(ThemeContext { theme, set_theme });
}

pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>()
        .expect("ThemeContext not provided — call provide_theme_context() in your root component")
}

fn document_element() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

fn current_html_theme() -> Option<Theme> {
    Some(Theme::from_str(
        &document_element()?.get_attribute("data-theme")?,
    ))
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
