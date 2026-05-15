use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::app_state::{open_note_by_title, use_app_state};
use crate::buttons::{PrimaryButton, SecondaryButton};
use crate::icons::{CheckIcon, PencilIcon};
use crate::images::{resolve_image_urls, save_image};

use super::context::use_notes_context;
use super::highlight::highlight_within;
use super::markdown::{render_inline_md, render_markdown};
use super::model::{format_relative, now_ms, parse_tags, Note};
use super::storage::save_one;
use super::templates::Template;

#[component]
pub fn Editor() -> impl IntoView {
    let app = use_app_state();
    let ctx = use_notes_context();
    let notes = app.notes;
    let set_notes = app.set_notes;
    let selected_id = app.selected_note_id;
    let set_selected_id = app.set_selected_note_id;
    let edit_mode = app.edit_mode;
    let set_edit_mode = app.set_edit_mode;

    let current = Memo::new(move |_| {
        let id = selected_id.get()?;
        notes.get().into_iter().find(|note| note.id == id)
    });

    let show_editor = Memo::new(move |_| edit_mode.get() && current.get().is_some());

    let persist_note = move |note: Note| {
        spawn_local(async move {
            save_one(note).await;
        });
    };

    let create_blank = move |_| {
        let mut note = Note::new_blank();
        Template::Blank.apply(&mut note);
        let id = note.id.clone();
        let to_save = note.clone();
        set_notes.update(|list| list.push(note));
        set_selected_id.set(Some(id));
        set_edit_mode.set(true);
        persist_note(to_save);
    };

    let on_title_input = move |ev: web_sys::Event| {
        let Some(id) = selected_id.get_untracked() else {
            return;
        };
        let value = event_target_value(&ev);
        update_note(set_notes, &id, persist_note, |note| {
            note.title = value;
        });
    };

    let on_tags_input = move |ev: web_sys::Event| {
        let Some(id) = selected_id.get_untracked() else {
            return;
        };
        let parsed = parse_tags(&event_target_value(&ev));
        update_note(set_notes, &id, persist_note, |note| {
            note.tags = parsed;
        });
    };

    let on_body_input = move |ev: web_sys::Event| {
        let Some(id) = selected_id.get_untracked() else {
            return;
        };
        let value = event_target_value(&ev);
        update_note(set_notes, &id, persist_note, |note| {
            note.body = value;
        });
    };

    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let preview_ref = NodeRef::<leptos::html::Div>::new();

    // After every preview re-render, re-apply syntax highlighting and resolve
    // any inline `image:ID` URLs back to live blob URLs.
    Effect::new(move |_| {
        let _ = current.get();
        if let Some(div) = preview_ref.get() {
            let element: &web_sys::Element = div.as_ref();
            highlight_within(element);
            resolve_image_urls(element);
        }
    });

    let insert_at_cursor = move |insert: String| {
        let Some(textarea) = textarea_ref.get() else {
            return;
        };
        let Some(id) = selected_id.get_untracked() else {
            return;
        };
        let start = textarea.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = textarea
            .selection_end()
            .ok()
            .flatten()
            .unwrap_or(start as u32) as usize;
        let value = textarea.value();
        let safe_start = start.min(value.len());
        let safe_end = end.min(value.len()).max(safe_start);
        let mut new_value = String::with_capacity(value.len() + insert.len());
        new_value.push_str(&value[..safe_start]);
        new_value.push_str(&insert);
        new_value.push_str(&value[safe_end..]);
        let new_cursor = (safe_start + insert.len()) as u32;

        update_note(set_notes, &id, persist_note, |note| {
            note.body = new_value;
        });

        // Restore cursor position after Leptos updates the DOM.
        let restore = Closure::once_into_js(move || {
            if let Some(textarea) = textarea_ref.get() {
                let _ = textarea.set_selection_start(Some(new_cursor));
                let _ = textarea.set_selection_end(Some(new_cursor));
            }
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback(restore.as_ref().unchecked_ref());
        }
    };

    let on_paste = move |ev: web_sys::ClipboardEvent| {
        let Some(data) = ev.clipboard_data() else {
            return;
        };
        let items = data.items();
        for i in 0..items.length() {
            let Some(item) = items.get(i) else { continue };
            if item.kind() != "file" {
                continue;
            }
            if !item.type_().starts_with("image/") {
                continue;
            }
            let Ok(Some(file)) = item.get_as_file() else {
                continue;
            };
            let blob: web_sys::Blob = file.into();
            ev.prevent_default();
            let inserter = insert_at_cursor;
            spawn_local(async move {
                if let Some(id) = save_image(&blob).await {
                    inserter(format!("\n![image](image:{})\n", id));
                }
            });
            return;
        }
    };

    let on_preview_click = move |ev: web_sys::MouseEvent| {
        let Some(target) = ev.target() else { return };
        let Ok(start) = target.dyn_into::<web_sys::Element>() else {
            return;
        };

        // Image click → open lightbox via context.
        if start.tag_name().eq_ignore_ascii_case("IMG") {
            if let Ok(img) = start.clone().dyn_into::<web_sys::HtmlImageElement>() {
                let src = img.src();
                if !src.is_empty() && !src.starts_with("image:") {
                    ev.prevent_default();
                    ctx.set_viewing_image.set(Some(src));
                    return;
                }
            }
        }

        // Walk up for a `note:` anchor.
        let mut current_el = Some(start);
        while let Some(el) = current_el {
            if el.tag_name().eq_ignore_ascii_case("A") {
                let href = el.get_attribute("href").unwrap_or_default();
                if let Some(encoded) = href.strip_prefix("note:") {
                    ev.prevent_default();
                    let title = js_sys::decode_uri_component(encoded)
                        .ok()
                        .and_then(|value| value.as_string())
                        .unwrap_or_else(|| encoded.to_string());
                    open_note_by_title(app, title);
                }
                return;
            }
            current_el = el.parent_element();
        }
    };

    view! {
        <div class="flex-1 flex flex-col gap-2 min-w-0">
            <Show when=move || edit_mode.get() && current.get().is_some()>
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        class="flex-1 rounded-lg border border-border bg-bg px-3 py-2 text-fg text-lg font-medium placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent"
                        placeholder="Title"
                        prop:value=move || current.get().map(|note| note.title).unwrap_or_default()
                        on:input=on_title_input
                    />
                    <PrimaryButton
                        on_click=move |_| set_edit_mode.set(false)
                        title="Done editing"
                        class="shrink-0"
                    >
                        <CheckIcon class="w-3.5 h-3.5"/>
                        "Done"
                    </PrimaryButton>
                </div>
                <input
                    type="text"
                    class="w-full rounded-md border border-border bg-bg px-3 py-1.5 text-fg text-sm placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent"
                    placeholder="Tags (comma-separated)"
                    prop:value=move || current.get().map(|note| note.tags.join(", ")).unwrap_or_default()
                    on:input=on_tags_input
                />
            </Show>
            <Show when=move || !edit_mode.get() && current.get().is_some()>
                <div class="border-b border-border pb-3 mb-1">
                    <div class="flex items-start justify-between gap-3">
                        <h2
                            class="text-2xl font-semibold text-fg m-0 flex-1 min-w-0"
                            inner_html=move || render_inline_md(
                                &current.get().map(|note| note.display_title()).unwrap_or_default()
                            )
                        ></h2>
                        <SecondaryButton
                            on_click=move |_| set_edit_mode.set(true)
                            title="Edit note"
                            class="shrink-0"
                        >
                            <PencilIcon class="w-3.5 h-3.5"/>
                            "Edit"
                        </SecondaryButton>
                    </div>
                    <div class="flex items-center gap-2 mt-1">
                        <div class="text-xs text-fg-muted">
                            "Updated "
                            {move || current.get().map(|note| format_relative(note.updated_at)).unwrap_or_default()}
                        </div>
                        <Show when=move || current.get().map(|note| !note.tags.is_empty()).unwrap_or(false)>
                            <div class="flex flex-wrap gap-1">
                                {move || current.get().map(|note| note.tags.clone()).unwrap_or_default().into_iter().map(|tag| {
                                    view! {
                                        <span class="px-2 py-0.5 rounded-full text-xs bg-bg text-fg-muted border border-border">
                                            "#"{tag}
                                        </span>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                    </div>
                </div>
            </Show>
            <Show when=move || current.get().is_none()>
                <div class="flex-1 flex items-center justify-center">
                    <div class="text-center">
                        <p class="text-fg-muted m-0 mb-4">"Select a note from the sidebar, or create a new one."</p>
                        <PrimaryButton on_click=create_blank>"+ New note"</PrimaryButton>
                    </div>
                </div>
            </Show>
            <Show when=move || current.get().is_some()>
                <div
                    class="flex-1 grid gap-3 min-h-0"
                    class:grid-cols-2=move || show_editor.get()
                    class:grid-cols-1=move || !show_editor.get()
                >
                    <Show when=move || show_editor.get()>
                        <textarea
                            node_ref=textarea_ref
                            class="w-full h-full min-h-[16rem] resize-none rounded-lg border border-border bg-bg p-3 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent font-mono text-sm"
                            placeholder="# Note title — Markdown supported. Paste images directly!"
                            prop:value=move || current.get().map(|note| note.body).unwrap_or_default()
                            on:input=on_body_input
                            on:paste=on_paste
                        />
                    </Show>
                    <div
                        class=move || {
                            if edit_mode.get() {
                                "rounded-lg border border-border bg-bg p-3 overflow-auto min-h-[16rem]"
                            } else {
                                "overflow-auto"
                            }
                        }
                    >
                        <Show when=move || edit_mode.get() && current.get().is_some()>
                            <h2
                                class="text-2xl font-semibold text-fg m-0 mb-3 pb-3 border-b border-border"
                                inner_html=move || render_inline_md(
                                    &current.get().map(|note| note.display_title()).unwrap_or_default()
                                )
                            ></h2>
                        </Show>
                        <div
                            node_ref=preview_ref
                            class="markdown-preview"
                            inner_html=move || render_markdown(&current.get().map(|note| note.body).unwrap_or_default())
                            on:click=on_preview_click
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Mutate the note with the matching id and persist it.
fn update_note<F, P>(
    set_notes: WriteSignal<Vec<Note>>,
    id: &str,
    persist: P,
    mutate: F,
)
where
    F: FnOnce(&mut Note),
    P: Fn(Note),
{
    let mut updated: Option<Note> = None;
    set_notes.update(|list| {
        if let Some(note) = list.iter_mut().find(|candidate| candidate.id == id) {
            mutate(note);
            note.updated_at = now_ms();
            updated = Some(note.clone());
        }
    });
    if let Some(note) = updated {
        persist(note);
    }
}
