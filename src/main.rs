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
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::hooks::{use_location, use_navigate};
use leptos_router::path;
use links::Links;
use notes::Notes;
use recipes::Recipes;
use search::QuickSwitcher;
use theme::{provide_theme_context, use_theme};
use titlebar::TitleBar;

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

    view! {
        <Router>
            <TitleBar/>
            <main class="p-3">
                <RouteSync/>
                <NavBar/>
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
            </main>
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

#[component]
fn NavBar() -> impl IntoView {
    let app = use_app_state();
    view! {
        <div class="flex items-center justify-between mb-3 gap-3 pb-3 border-b border-border">
            <PageTabs/>
            <div class="flex items-center gap-2">
                <button
                    class="inline-flex items-center justify-center w-9 h-9 rounded-md bg-bg-elevated text-fg border border-border hover:bg-fg hover:text-bg transition text-sm font-semibold"
                    on:click=move |_| app.set_show_help.set(true)
                    title="Help & shortcuts"
                >
                    "?"
                </button>
                <ThemeToggle/>
            </div>
        </div>
    }
}

#[component]
fn PageTabs() -> impl IntoView {
    view! {
        <nav class="flex gap-2">
            <NavLink target=Page::Notes label="Notes" title="Notes (1)"/>
            <NavLink target=Page::Campaign label="Campaign" title="Campaign (2)"/>
            <NavLink target=Page::Recipes label="Recipes" title="Recipes (3)"/>
            <NavLink target=Page::Links label="Links" title="Links (4)"/>
        </nav>
    }
}

#[component]
fn NavLink(
    target: Page,
    #[prop(into)] label: String,
    #[prop(into)] title: String,
) -> impl IntoView {
    let app = use_app_state();
    let class = move || {
        let base = "inline-flex items-center h-9 px-3 rounded-md border text-sm transition no-underline";
        if app.page.get() == target {
            format!("{} bg-accent text-accent-fg border-accent", base)
        } else {
            format!(
                "{} bg-transparent text-fg border-border hover:bg-fg hover:text-bg",
                base
            )
        }
    };
    view! {
        <A href=target.route() attr:class=class attr:title=title>
            {label}
        </A>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let ctx = use_theme();
    view! {
        <button
            class="inline-flex items-center h-9 px-3 rounded-md bg-bg-elevated text-fg border border-border hover:bg-fg hover:text-bg transition text-sm"
            on:click=move |_| ctx.cycle()
            title="Cycle theme"
        >
            {move || format!("Theme: {}", ctx.theme.get().label())}
        </button>
    }
}
