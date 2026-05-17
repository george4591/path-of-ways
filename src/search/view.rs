use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::app_state::{open_note, open_note_for_zone, use_app_state, Page};
use crate::campaign::{load_zones, Zone};
use crate::keyboard::use_escape_key;
use crate::notes::{render_inline_md, Template};
use crate::recipes::{load_recipes, Recipe};

use super::links_loader::{load_links, open_url, LinkRow};
use super::matching::compute_groups;
use super::model::{render_labels, Result};

#[component]
pub fn QuickSwitcher() -> impl IntoView {
    let app = use_app_state();
    let (query, set_query) = signal(String::new());
    let (links, set_links) = signal(Vec::<LinkRow>::new());
    let (recipes, set_recipes) = signal(Vec::<Recipe>::new());
    let (zones, set_zones) = signal(Vec::<Zone>::new());

    spawn_local(async move {
        set_links.set(load_links().await);
    });
    spawn_local(async move {
        set_recipes.set(load_recipes().await);
    });
    spawn_local(async move {
        set_zones.set(load_zones().await);
    });

    let input_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    });

    let close = move || {
        app.set_show_quick_switcher.set(false);
        set_query.set(String::new());
    };

    use_escape_key(close);

    let groups = move || {
        compute_groups(
            app.notes.get(),
            zones.get(),
            recipes.get(),
            links.get(),
            &query.get(),
        )
    };

    let on_pick = move |result: Result| {
        match result {
            Result::Note { id, .. } => open_note(app, id),
            Result::Zone {
                zone_id, name, ..
            } => {
                open_note_for_zone(app, zone_id, name, Template::Blank);
            }
            Result::Recipe { .. } => {
                app.set_page.set(Page::Recipes);
            }
            Result::Link { url, .. } => open_url(&url),
        }
        close();
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-sm"
            on:click=move |_| close()
        >
            <div
                class="w-full max-w-2xl mx-4 rounded-xl border border-border bg-bg-elevated shadow-2xl overflow-hidden flex flex-col max-h-[70vh]"
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                <div class="p-3 border-b border-border">
                    <input
                        node_ref=input_ref
                        type="text"
                        placeholder="Search notes, zones, recipes, links…"
                        class="w-full rounded-md bg-bg border border-border px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Escape" {
                                close();
                            }
                        }
                    />
                </div>
                <div class="overflow-auto flex-1">
                    {move || {
                        let computed = groups();
                        if computed.is_empty() {
                            view! {
                                <div class="text-sm text-fg-muted p-6 text-center">"No results."</div>
                            }.into_any()
                        } else {
                            computed.into_iter()
                                .map(|(label, results)| view! {
                                    <ResultGroup label=label results=results on_pick=on_pick/>
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
                <div class="px-3 py-2 border-t border-border text-xs text-fg-muted">
                    "Ctrl+K to toggle · Esc to close"
                </div>
            </div>
        </div>
    }
}

#[component]
fn ResultGroup<F>(
    label: &'static str,
    results: Vec<Result>,
    on_pick: F,
) -> impl IntoView
where
    F: Fn(Result) + Copy + Send + Sync + 'static,
{
    view! {
        <div>
            <div class="px-3 pt-3 pb-1 text-xs font-medium uppercase tracking-wider text-fg-muted">
                {label}
            </div>
            <ul class="flex flex-col">
                {results.into_iter().map(|result| view! {
                    <ResultRow result=result on_pick=on_pick/>
                }).collect_view()}
            </ul>
        </div>
    }
}

#[component]
fn ResultRow<F>(result: Result, on_pick: F) -> impl IntoView
where
    F: Fn(Result) + Copy + Send + Sync + 'static,
{
    let (primary, secondary) = render_labels(&result);
    let primary_html = render_inline_md(&primary);
    let result_for_click = result.clone();
    view! {
        <li>
            <button
                class="w-full text-left px-3 py-2 hover:bg-bg transition flex items-baseline gap-2"
                on:click=move |_| on_pick(result_for_click.clone())
            >
                <span class="text-sm text-fg truncate flex-1" inner_html=primary_html></span>
                <span class="text-xs text-fg-muted truncate shrink-0">{secondary}</span>
            </button>
        </li>
    }
}
