use leptos::prelude::*;
use leptos::web_sys;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

use super::model::{tag_color_classes, ChecklistItem, COMMON_TAGS};

#[derive(Clone, Default)]
pub struct ZoneDraft {
    pub id: Option<String>,
    pub act: String,
    pub name: String,
    pub tags: Vec<String>,
    /// Pending tag-in-progress; commits to `tags` on Enter/comma.
    pub tag_input: String,
    pub checklist: Vec<ChecklistItem>,
}

const FIELD_CLASS: &str = "rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-base";

#[component]
pub fn EditModal<C, K>(
    draft: ReadSignal<Option<ZoneDraft>>,
    set_draft: WriteSignal<Option<ZoneDraft>>,
    cancel: C,
    confirm: K,
) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    let is_edit = move || draft.get().and_then(|current| current.id).is_some();

    let update_text = move |mutate: fn(&mut ZoneDraft, String), value: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                mutate(current, value);
            }
        });
    };

    // ─── Tags handling ───────────────────────────────────────────────────

    let add_tag = move |tag: String| {
        let trimmed = tag.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        set_draft.update(|slot| {
            if let Some(current) = slot {
                if !current.tags.iter().any(|existing| existing == &trimmed) {
                    current.tags.push(trimmed);
                }
                current.tag_input.clear();
            }
        });
    };

    let remove_tag = move |tag: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                current.tags.retain(|existing| existing != &tag);
            }
        });
    };

    let commit_pending_tag = move || {
        let pending = draft
            .get_untracked()
            .map(|current| current.tag_input)
            .unwrap_or_default();
        add_tag(pending);
    };

    // ─── Checklist handling ──────────────────────────────────────────────

    let add_item = move |_| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                current.checklist.push(ChecklistItem::new(String::new()));
            }
        });
    };

    let remove_item = move |id: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                current.checklist.retain(|item| item.id != id);
            }
        });
    };

    let update_item_text = move |id: String, value: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                if let Some(item) = current.checklist.iter_mut().find(|item| item.id == id) {
                    item.text = value;
                }
            }
        });
    };

    // Materialize the checklist as a Memo so <For> sees a stable per-item key
    // (the ChecklistItem.id) and only re-renders the row whose text changes.
    let items = Memo::new(move |_| {
        draft
            .get()
            .map(|current| current.checklist)
            .unwrap_or_default()
    });

    let tags = Memo::new(move |_| {
        draft
            .get()
            .map(|current| current.tags)
            .unwrap_or_default()
    });

    view! {
        <ModalShell cancel=cancel confirm=confirm panel_class="max-w-md max-h-[85vh] flex flex-col">
            <h3 class="text-lg font-semibold text-fg mb-4 shrink-0">
                {move || if is_edit() { "Edit zone" } else { "Add zone" }}
            </h3>
            <div class="flex flex-col gap-3 flex-1 overflow-auto px-2 -mx-2">
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Act"</span>
                    <input
                        type="text"
                        placeholder="e.g. Act 1: The Riverbank"
                        class=FIELD_CLASS
                        prop:value=move || draft.get().map(|current| current.act).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.act = v, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Zone name"</span>
                    <input
                        type="text"
                        placeholder="e.g. Clearfell"
                        class=FIELD_CLASS
                        prop:value=move || draft.get().map(|current| current.name).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.name = v, event_target_value(&ev))
                    />
                </label>

                // ─── Tags ────────────────────────────────────────────────
                <div class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Tags"</span>
                    <Show when=move || !tags.with(|list| list.is_empty())>
                        <div class="flex flex-wrap gap-1">
                            <For
                                each=move || tags.get()
                                key=|tag| tag.clone()
                                let:tag
                            >
                                <span class=format!(
                                    "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs {}",
                                    tag_color_classes(&tag),
                                )>
                                    {tag.clone()}
                                    <button
                                        type="button"
                                        class="hover:brightness-125 leading-none"
                                        on:click={
                                            let tag = tag.clone();
                                            move |_| remove_tag(tag.clone())
                                        }
                                        title="Remove tag"
                                    >
                                        "×"
                                    </button>
                                </span>
                            </For>
                        </div>
                    </Show>
                    <input
                        type="text"
                        placeholder="Add a tag and press Enter"
                        class=FIELD_CLASS
                        prop:value=move || draft.get().map(|current| current.tag_input).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.tag_input = v, event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            let key = ev.key();
                            if key == "Enter" || key == "," {
                                let has_pending = draft
                                    .get_untracked()
                                    .map(|current| !current.tag_input.trim().is_empty())
                                    .unwrap_or(false);
                                if has_pending {
                                    // Don't let the modal-level Enter-to-save fire.
                                    ev.stop_propagation();
                                    ev.prevent_default();
                                    commit_pending_tag();
                                }
                            }
                        }
                    />
                    <div class="flex flex-wrap gap-1 mt-1">
                        {COMMON_TAGS.iter().map(|tag| {
                            let tag_string = tag.to_string();
                            let tag_for_view = tag.to_string();
                            // Preview the final tag color on the chip so the
                            // user knows what they'll get before clicking.
                            let color_classes = tag_color_classes(tag);
                            let already_has = Signal::derive(move || {
                                draft.get()
                                    .map(|current| current.tags.iter().any(|t| t == &tag_string))
                                    .unwrap_or(false)
                            });
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "text-xs px-2 py-0.5 rounded-full border transition";
                                        if already_has.get() {
                                            // Already applied: dim it and lock it out, regardless of category color.
                                            format!("{} bg-fg/10 text-fg-muted border-border opacity-50 cursor-default", base)
                                        } else {
                                            format!("{} {} hover:brightness-125", base, color_classes)
                                        }
                                    }
                                    disabled=move || already_has.get()
                                    on:click={
                                        let tag = tag_for_view.clone();
                                        move |_| add_tag(tag.clone())
                                    }
                                >
                                    "+ "{tag_for_view.clone()}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // ─── Checklist ───────────────────────────────────────────
                <div class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Checklist"</span>
                    <div class="flex flex-col gap-2">
                        <For
                            each=move || items.get()
                            key=|item| item.id.clone()
                            let:item
                        >
                            <ChecklistRow
                                id=item.id.clone()
                                text=item.text.clone()
                                on_text={
                                    let id = item.id.clone();
                                    move |value: String| update_item_text(id.clone(), value)
                                }
                                on_remove={
                                    let id = item.id.clone();
                                    move || remove_item(id.clone())
                                }
                            />
                        </For>
                        <button
                            type="button"
                            class="self-start text-xs text-accent hover:underline mt-1"
                            on:click=add_item
                        >
                            "+ Add checklist item"
                        </button>
                    </div>
                </div>
            </div>
            <div class="flex justify-end gap-2 mt-4 shrink-0">
                <SecondaryButton on_click=move |_| cancel()>"Cancel"</SecondaryButton>
                <PrimaryButton on_click=move |_| confirm()>"Save"</PrimaryButton>
            </div>
        </ModalShell>
    }
}

#[component]
fn ChecklistRow<T, R>(
    id: String,
    text: String,
    on_text: T,
    on_remove: R,
) -> impl IntoView
where
    T: Fn(String) + 'static,
    R: Fn() + 'static,
{
    // `id` is reserved for keying via <For>; we don't actually need it in the
    // row markup, but Leptos requires the binding to live for the closures.
    let _ = id;
    view! {
        <div class="flex items-center gap-2">
            <input
                type="text"
                placeholder="e.g. Kill The Bloated Miller"
                class=format!("flex-1 {}", FIELD_CLASS)
                prop:value=text
                on:input=move |ev| on_text(event_target_value(&ev))
            />
            <button
                type="button"
                class="inline-flex items-center justify-center w-8 h-8 rounded bg-bg-elevated border border-border text-fg-muted hover:text-red-400 hover:border-red-500"
                on:click=move |_| on_remove()
                title="Remove this item"
            >
                <TrashIcon class="w-3.5 h-3.5"/>
            </button>
        </div>
    }
}

#[component]
pub fn DeleteModal<C, K>(cancel: C, confirm: K) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <ModalShell cancel=cancel confirm=confirm>
            <h3 class="text-lg font-semibold text-fg mb-2">"Delete this zone?"</h3>
            <p class="text-sm text-fg-muted mb-4">
                "Existing notes linked to this zone keep their content but lose the link. This can't be undone."
            </p>
            <div class="flex justify-end gap-2">
                <SecondaryButton on_click=move |_| cancel()>"Cancel"</SecondaryButton>
                <DangerButton on_click=move |_| confirm()>
                    <TrashIcon class="w-4 h-4"/>
                    "Delete"
                </DangerButton>
            </div>
        </ModalShell>
    }
}

/// Confirmation dialog for resetting every checklist item's done-state across
/// every zone. Zones, checklist items, and tags themselves are left intact —
/// only the progress is wiped.
#[component]
pub fn ResetProgressModal<C, K>(cancel: C, confirm: K) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <ModalShell cancel=cancel confirm=confirm>
            <h3 class="text-lg font-semibold text-fg mb-2">"Reset all progress?"</h3>
            <p class="text-sm text-fg-muted mb-4">
                "Unchecks every checklist item in every zone. Your zones, checklist items, and tags stay exactly as they are — only the done-state is cleared. This can't be undone."
            </p>
            <div class="flex justify-end gap-2">
                <SecondaryButton on_click=move |_| cancel()>"Cancel"</SecondaryButton>
                <DangerButton on_click=move |_| confirm()>
                    <TrashIcon class="w-4 h-4"/>
                    "Reset progress"
                </DangerButton>
            </div>
        </ModalShell>
    }
}
