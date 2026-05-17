use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::app_state::use_app_state;
use crate::icons::{PinIcon, SearchIcon, TrashIcon};
use crate::keyboard::use_escape_key;

use super::context::use_notes_context;
use super::markdown::render_inline_md;
use super::model::{format_relative, now_ms, Note};
use super::storage::save_one;
use super::templates::Template;

#[component]
pub fn Sidebar() -> impl IntoView {
    let app = use_app_state();
    let ctx = use_notes_context();
    let notes = app.notes;
    let set_notes = app.set_notes;
    let selected_id = app.selected_note_id;
    let set_selected_id = app.set_selected_note_id;
    let set_edit_mode = app.set_edit_mode;

    let (search_query, set_search_query) = signal(String::new());
    let (active_tags, set_active_tags) = signal(Vec::<String>::new());
    let (show_new_menu, set_show_new_menu) = signal(false);

    // Esc closes the template dropdown when it's open.
    use_escape_key(move || {
        if show_new_menu.get_untracked() {
            set_show_new_menu.set(false);
        }
    });

    let persist_note = move |note: Note| {
        spawn_local(async move {
            save_one(note).await;
        });
    };

    let all_tags = move || {
        let mut tags: Vec<String> = notes
            .get()
            .iter()
            .flat_map(|note| note.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    };

    let filtered_sorted = move || {
        let query = search_query.get();
        let active = active_tags.get();
        let mut list: Vec<Note> = notes
            .get()
            .into_iter()
            .filter(|note| note.matches_query(&query) && note.has_any_tag(&active))
            .collect();
        list.sort_by(|left, right| {
            right.pinned.cmp(&left.pinned).then_with(|| {
                right
                    .updated_at
                    .partial_cmp(&left.updated_at)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        list
    };

    let create_with_template = move |template: Template| {
        let mut note = Note::new_blank();
        template.apply(&mut note);
        let id = note.id.clone();
        let to_save = note.clone();
        set_notes.update(|list| list.push(note));
        set_selected_id.set(Some(id));
        set_edit_mode.set(true);
        persist_note(to_save);
        set_show_new_menu.set(false);
    };

    let select_note = move |id: String| {
        set_selected_id.set(Some(id));
        set_edit_mode.set(false);
    };

    let toggle_pin_for = move |id: String| {
        let mut updated: Option<Note> = None;
        set_notes.update(|list| {
            if let Some(note) = list.iter_mut().find(|candidate| candidate.id == id) {
                note.pinned = !note.pinned;
                note.updated_at = now_ms();
                updated = Some(note.clone());
            }
        });
        if let Some(note) = updated {
            persist_note(note);
        }
    };

    let request_delete_for = move |id: String| {
        ctx.set_pending_delete.set(Some(id));
    };

    let toggle_tag_filter = move |tag: String| {
        set_active_tags.update(|tags| {
            if let Some(pos) = tags.iter().position(|existing| existing == &tag) {
                tags.remove(pos);
            } else {
                tags.push(tag);
            }
        });
    };

    view! {
        <aside class="w-72 shrink-0 flex flex-col gap-2 pr-4 border-r border-border">
            // ─── Search + New (single row) ─────────────────────────────
            // Search takes the full width of the row, with a compact
            // split-button on the right: `+` creates a blank note, `▾`
            // opens the template picker. Import / Export moved to the
            // global File menu in the title bar.
            <div class="flex gap-2">
                <div class="relative flex-1 min-w-0">
                    <input
                        type="text"
                        placeholder="Search notes…"
                        class="w-full rounded-md border border-border bg-bg pl-8 pr-2 py-1.5 text-sm text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent"
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <span class="absolute left-2 top-1/2 -translate-y-1/2 text-fg-muted pointer-events-none">
                        <SearchIcon class="w-4 h-4"/>
                    </span>
                </div>
                <div class="relative shrink-0">
                    <div class="flex">
                        <button
                            class="inline-flex items-center justify-center w-9 h-9 rounded-l-md bg-accent text-accent-fg hover:opacity-90 transition text-sm font-semibold"
                            on:click=move |_| create_with_template(Template::Blank)
                            title="New blank note (Ctrl+N)"
                        >
                            "+"
                        </button>
                        <button
                            class="inline-flex items-center justify-center w-6 h-9 rounded-r-md bg-accent text-accent-fg hover:opacity-90 transition border-l border-accent-fg/20 text-xs"
                            on:click=move |_| set_show_new_menu.update(|open| *open = !*open)
                            title="Choose a template"
                        >
                            "▾"
                        </button>
                    </div>
                    <Show when=move || show_new_menu.get()>
                        <div
                            class="fixed inset-0 z-40"
                            on:click=move |_| set_show_new_menu.set(false)
                        />
                        <div class="absolute z-50 top-full mt-1 right-0 w-56 rounded-lg border border-border bg-bg-elevated shadow-2xl overflow-hidden">
                            {[Template::Blank, Template::Build, Template::Boss].into_iter().map(|template| {
                                view! {
                                    <button
                                        class="w-full text-left px-3 py-2 hover:bg-bg transition border-b border-border last:border-b-0"
                                        on:click=move |_| create_with_template(template)
                                    >
                                        <div class="text-sm text-fg font-medium">{template.label()}</div>
                                        <div class="text-xs text-fg-muted">{template.description()}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </Show>
                </div>
            </div>
            <Show when=move || !all_tags().is_empty()>
                <div class="flex flex-wrap gap-1">
                    {move || {
                        let active = active_tags.get();
                        all_tags().into_iter().map(|tag| {
                            let is_active = active.contains(&tag);
                            let tag_for_click = tag.clone();
                            view! {
                                <button
                                    class=move || {
                                        let base = "px-2 py-0.5 rounded-full text-xs border transition";
                                        if is_active {
                                            format!("{} bg-accent text-accent-fg border-accent", base)
                                        } else {
                                            format!("{} bg-transparent text-fg-muted border-border hover:text-fg", base)
                                        }
                                    }
                                    on:click=move |_| toggle_tag_filter(tag_for_click.clone())
                                >
                                    "#"{tag.clone()}
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>
            </Show>
            <ul class="flex-1 overflow-y-auto flex flex-col gap-1 pr-1">
                {move || {
                    let list = filtered_sorted();
                    if list.is_empty() {
                        view! {
                            <li class="text-sm text-fg-muted p-2">"No notes match."</li>
                        }.into_any()
                    } else {
                        list.into_iter().map(|note| view! {
                            <NoteListItem
                                note=note
                                selected_id=selected_id
                                select_note=select_note
                                toggle_pin_for=toggle_pin_for
                                request_delete_for=request_delete_for
                            />
                        }).collect_view().into_any()
                    }
                }}
            </ul>
        </aside>
    }
}

#[component]
fn NoteListItem<S, P, D>(
    note: Note,
    selected_id: ReadSignal<Option<String>>,
    select_note: S,
    toggle_pin_for: P,
    request_delete_for: D,
) -> impl IntoView
where
    S: Fn(String) + Copy + Send + Sync + 'static,
    P: Fn(String) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
{
    let id = note.id.clone();
    let id_for_click = id.clone();
    let id_for_pin = id.clone();
    let id_for_delete = id.clone();
    let title_html = render_inline_md(&note.display_title());
    let snippet_html = render_inline_md(&note.body_preview());
    let updated_str = format_relative(note.updated_at);
    let pinned = note.pinned;
    let is_selected = move || selected_id.get().as_ref() == Some(&id);

    view! {
        <li class="relative group">
            <div
                class=move || {
                    let base = "w-full text-left px-3 py-2 rounded-md transition text-sm border cursor-pointer";
                    if is_selected() {
                        format!("{} bg-bg border-accent text-fg", base)
                    } else {
                        format!("{} bg-transparent border-transparent text-fg hover:bg-bg hover:border-border", base)
                    }
                }
                on:click=move |_| select_note(id_for_click.clone())
            >
                <div class="flex items-baseline gap-2">
                    <Show when=move || pinned>
                        <PinIcon class=Signal::derive(|| "w-3 h-3 shrink-0 text-accent".to_string())/>
                    </Show>
                    <div class="font-medium truncate flex-1" inner_html=title_html></div>
                    <div class="text-[0.65rem] text-fg-muted shrink-0 group-hover:invisible">{updated_str}</div>
                </div>
                <div class="text-[0.7rem] text-fg-muted truncate leading-snug mt-0.5" inner_html=snippet_html></div>
            </div>
            <div class="absolute top-2 right-2 hidden group-hover:flex gap-0.5">
                <button
                    class=move || {
                        let pin_color = if pinned { "text-accent" } else { "text-fg-muted" };
                        format!(
                            "inline-flex items-center justify-center w-6 h-6 rounded bg-bg-elevated border border-border hover:text-fg {}",
                            pin_color
                        )
                    }
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        toggle_pin_for(id_for_pin.clone());
                    }
                    title=if pinned { "Unpin" } else { "Pin to top" }
                >
                    <PinIcon class=Signal::derive(|| "w-3 h-3".to_string())/>
                </button>
                <button
                    class="inline-flex items-center justify-center w-6 h-6 rounded bg-red-700 text-white border border-red-700 hover:bg-red-800 hover:border-red-800"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        request_delete_for(id_for_delete.clone());
                    }
                    title="Delete"
                >
                    <TrashIcon class="w-3 h-3"/>
                </button>
            </div>
        </li>
    }
}
