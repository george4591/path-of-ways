use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::app_state::{open_note_for_zone, use_app_state};
use crate::buttons::PrimaryButton;
use crate::icons::{PencilIcon, TrashIcon};
use crate::notes::{render_inline_md, Template};

use super::modals::{DeleteModal, EditModal, ResetProgressModal, ZoneDraft};
use super::model::{tag_color_classes, Zone, ZoneProgress, COMMON_TAGS};
use super::storage::{
    clear_all_progress, delete_zone, load_progress, load_zones, save_zone,
    save_zone_progress,
};

#[component]
pub fn CampaignTracker() -> impl IntoView {
    let app = use_app_state();
    let (zones, set_zones) = signal(Vec::<Zone>::new());
    let (progress_map, set_progress_map) = signal(HashMap::<String, ZoneProgress>::new());
    let (editing, set_editing) = signal::<Option<ZoneDraft>>(None);
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (managing_act, set_managing_act) = signal::<Option<String>>(None);
    let (loaded, set_loaded) = signal(false);
    // Tags currently selected as filters. Empty set = show everything.
    // Persisted to localStorage so a user's filter choice survives reloads.
    let (selected_tags, set_selected_tags) = signal(read_string_set(LS_SELECTED_TAGS));
    // Names of acts currently expanded. Empty = everything collapsed (default).
    // Also persisted so the user's chosen "open" state isn't lost on reload.
    let (expanded_acts, set_expanded_acts) = signal(read_string_set(LS_EXPANDED_ACTS));
    // Toggles the "Reset all progress" confirmation modal.
    let (pending_reset, set_pending_reset) = signal(false);

    // Write-through effects. The `prev.is_some()` guard skips the first run
    // so we don't pointlessly rewrite the same value we just read on init.
    Effect::new(move |prev: Option<HashSet<String>>| {
        let current = selected_tags.get();
        if prev.is_some() {
            write_string_set(LS_SELECTED_TAGS, &current);
        }
        current
    });
    Effect::new(move |prev: Option<HashSet<String>>| {
        let current = expanded_acts.get();
        if prev.is_some() {
            write_string_set(LS_EXPANDED_ACTS, &current);
        }
        current
    });

    let toggle_act_expanded = move |act: String| {
        set_expanded_acts.update(|set| {
            if !set.remove(&act) {
                set.insert(act);
            }
        });
    };

    spawn_local(async move {
        let zone_list = load_zones().await;
        set_zones.set(zone_list);
        set_loaded.set(true);
    });

    spawn_local(async move {
        let progress_list = load_progress().await;
        let mut map = HashMap::new();
        for progress in progress_list {
            map.insert(progress.zone_id.clone(), progress);
        }
        set_progress_map.set(map);
    });

    // Toggle a single checklist item's done state on the given zone.
    let toggle_item = move |zone_id: String, item_id: String| {
        let mut updated: Option<ZoneProgress> = None;
        set_progress_map.update(|map| {
            let progress = map
                .entry(zone_id.clone())
                .or_insert_with(|| ZoneProgress::for_zone(&zone_id));
            if progress.done_items.contains(&item_id) {
                progress.done_items.remove(&item_id);
            } else {
                progress.done_items.insert(item_id.clone());
            }
            updated = Some(progress.clone());
        });
        if let Some(progress) = updated {
            spawn_local(async move {
                save_zone_progress(progress).await;
            });
        }
    };

    // Filtering: a zone passes if no filters are active, or if it has at
    // least one of the selected tags (OR semantics — "anything offering
    // skill gem OR skill point", not "both").
    let zone_passes_filter = move |zone: &Zone| -> bool {
        let selected = selected_tags.get();
        if selected.is_empty() {
            return true;
        }
        zone.tags.iter().any(|tag| selected.contains(tag))
    };

    // Group filtered zones by act for display.
    let grouped = move || -> BTreeMap<String, Vec<Zone>> {
        let mut map: BTreeMap<String, Vec<Zone>> = BTreeMap::new();
        let mut list: Vec<Zone> = zones
            .get()
            .into_iter()
            .filter(|zone| zone_passes_filter(zone))
            .collect();
        list.sort_by(|left, right| {
            left.created_at
                .partial_cmp(&right.created_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for zone in list {
            map.entry(zone.act.clone()).or_default().push(zone);
        }
        map
    };

    // Tags shown in the filter bar, alphabetically. Pre-seeded with the
    // standard `COMMON_TAGS` so the bar always has presence — even on a
    // brand-new install with no tagged zones yet — and any custom tags
    // the user has added get appended on top of that baseline. Counts
    // use the full unfiltered zone list so numbers don't shift as the
    // user toggles other filters.
    let all_tags = move || -> Vec<(String, usize)> {
        let zones_list = zones.get();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for tag in COMMON_TAGS {
            counts.insert((*tag).to_string(), 0);
        }
        for zone in &zones_list {
            // Track each tag once per zone, so 5 zones with "Boss" = count 5.
            let unique: BTreeSet<&String> = zone.tags.iter().collect();
            for tag in unique {
                *counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        counts.into_iter().collect()
    };

    let toggle_tag_filter = move |tag: String| {
        set_selected_tags.update(|set| {
            if set.contains(&tag) {
                set.remove(&tag);
            } else {
                set.insert(tag);
            }
        });
    };

    let clear_filters = move |_| set_selected_tags.set(HashSet::new());

    // ─── Per-act totals over all (unfiltered) zones in that act ──────────
    let act_summary = move |act_name: String| -> (usize, usize) {
        let map = progress_map.get();
        let zones_list = zones.get();
        let mut total = 0usize;
        let mut done = 0usize;
        for zone in zones_list.iter().filter(|zone| zone.act == act_name) {
            total += zone.checklist.len();
            if let Some(progress) = map.get(&zone.id) {
                for item in &zone.checklist {
                    if progress.done_items.contains(&item.id) {
                        done += 1;
                    }
                }
            }
        }
        (done, total)
    };

    let open_add = move |_| set_editing.set(Some(ZoneDraft::default()));
    // Per-act add — prefills the act name so the user doesn't have to retype
    // it. Also auto-expands the act so the new zone (once added) is visible.
    let open_add_in_act = move |act_name: String| {
        set_expanded_acts.update(|set| {
            set.insert(act_name.clone());
        });
        set_editing.set(Some(ZoneDraft {
            act: act_name,
            ..ZoneDraft::default()
        }));
    };

    let open_edit = move |zone: Zone| {
        set_editing.set(Some(ZoneDraft {
            id: Some(zone.id),
            act: zone.act,
            name: zone.name,
            tags: zone.tags,
            tag_input: String::new(),
            checklist: zone.checklist,
        }));
    };

    let cancel_edit = move || set_editing.set(None);

    let save_draft = move || {
        let Some(draft) = editing.get_untracked() else {
            return;
        };
        let act = draft.act.trim().to_string();
        let name = draft.name.trim().to_string();
        if act.is_empty() || name.is_empty() {
            return;
        }
        // Drop checklist items with empty text — those are accidents from
        // hitting "+ Add" and not filling anything in.
        let checklist: Vec<_> = draft
            .checklist
            .into_iter()
            .filter_map(|mut item| {
                item.text = item.text.trim().to_string();
                if item.text.is_empty() {
                    None
                } else {
                    Some(item)
                }
            })
            .collect();
        // Commit a still-pending tag input (user typed something but didn't
        // press Enter before clicking Save). Drop empties + duplicates.
        let mut tags: Vec<String> = Vec::new();
        for tag in draft.tags.into_iter().chain(std::iter::once(draft.tag_input)) {
            let trimmed = tag.trim().to_string();
            if !trimmed.is_empty() && !tags.iter().any(|existing| existing == &trimmed) {
                tags.push(trimmed);
            }
        }

        match draft.id {
            Some(id) => {
                let id_match = id.clone();
                let mut updated: Option<Zone> = None;
                set_zones.update(|list| {
                    if let Some(zone) = list.iter_mut().find(|candidate| candidate.id == id_match) {
                        zone.act = act.clone();
                        zone.name = name.clone();
                        zone.tags = tags.clone();
                        zone.checklist = checklist.clone();
                        updated = Some(zone.clone());
                    }
                });
                if let Some(zone) = updated {
                    spawn_local(async move {
                        save_zone(zone).await;
                    });
                }
            }
            None => {
                let zone = Zone::new(act, name, tags, checklist);
                let to_save = zone.clone();
                set_zones.update(|list| list.push(zone));
                spawn_local(async move {
                    save_zone(to_save).await;
                });
            }
        }
        set_editing.set(None);
    };

    let request_delete = move |id: String| set_pending_delete.set(Some(id));
    let cancel_delete = move || set_pending_delete.set(None);
    let confirm_delete = move || {
        let Some(id) = pending_delete.get_untracked() else {
            return;
        };
        let id_for_delete = id.clone();
        set_zones.update(|list| list.retain(|zone| zone.id != id));
        spawn_local(async move {
            delete_zone(id_for_delete).await;
        });
        set_pending_delete.set(None);
    };

    let open_reset = move |_| set_pending_reset.set(true);
    let cancel_reset = move || set_pending_reset.set(false);
    let confirm_reset = move || {
        // Clear locally first so the UI updates instantly, then persist.
        set_progress_map.set(HashMap::new());
        spawn_local(async move {
            clear_all_progress().await;
        });
        set_pending_reset.set(false);
    };

    view! {
        <section class="p-6 h-[calc(100vh-2.25rem)] min-h-[28rem] overflow-auto">
            // ─── Top bar: filter pills (left) + Reset progress (right) ─
            //
            // The page title + description used to live here; they were
            // redundant with the active title-bar tab and visually weaker
            // than the act headers below. Reset progress moved into this
            // row so the section starts straight in the content.
            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let tags = all_tags();
                view! {
                    <div class="flex flex-wrap items-center gap-2 mb-5 pb-3 border-b border-border">
                        <span class="text-xs text-fg-muted mr-1">"Filter:"</span>
                        {tags.into_iter().map(|(tag, count)| {
                            let tag_for_class = tag.clone();
                            let tag_for_click = tag.clone();
                            let color_classes = tag_color_classes(&tag);
                            let is_selected = move || selected_tags.get().contains(&tag_for_class);
                            // Pills with zero matching zones still render so
                            // the filter bar has stable presence — they just
                            // fade so the user can tell at a glance which
                            // tags are currently unused.
                            let is_empty = count == 0;
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let base = "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border transition";
                                        if is_selected() {
                                            // Selected: solid accent for unambiguous "this is on"
                                            // signaling, regardless of the tag's category color.
                                            format!("{} bg-accent text-accent-fg border-accent", base)
                                        } else if is_empty {
                                            format!("{} {} opacity-50 hover:opacity-100", base, color_classes)
                                        } else {
                                            format!("{} {} hover:brightness-125", base, color_classes)
                                        }
                                    }
                                    on:click={
                                        let tag = tag_for_click.clone();
                                        move |_| toggle_tag_filter(tag.clone())
                                    }
                                >
                                    {tag.clone()}
                                    <span class="opacity-70">{format!("({})", count)}</span>
                                </button>
                            }
                        }).collect_view()}
                        <Show when=move || !selected_tags.with(|s| s.is_empty())>
                            <button
                                type="button"
                                class="ml-1 text-xs text-fg-muted hover:text-fg underline"
                                on:click=clear_filters
                            >
                                "Clear filters"
                            </button>
                        </Show>
                        // Push Reset progress to the far right of the row.
                        <div class="flex-1 min-w-0"></div>
                        <button
                            class="inline-flex items-center h-8 px-3 rounded-md border border-red-700/70 bg-red-950/20 text-red-300 hover:bg-red-700/25 hover:border-red-600 hover:text-red-200 text-sm transition"
                            on:click=open_reset
                            title="Uncheck every checklist item across every zone"
                        >
                            "Reset progress"
                        </button>
                    </div>
                }.into_any()
            }}

            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let groups = grouped();
                if groups.is_empty() {
                    let no_zones = zones.with(|list| list.is_empty());
                    if no_zones {
                        return view! {
                            <div class="text-center text-fg-muted py-16">
                                <p class="m-0 mb-3">"No zones yet."</p>
                                <PrimaryButton on_click=open_add>"+ Add your first zone"</PrimaryButton>
                            </div>
                        }.into_any();
                    }
                    // Zones exist but the active filter hides all of them.
                    return view! {
                        <div class="text-center text-fg-muted py-16">
                            <p class="m-0">"No zones match the current filter."</p>
                        </div>
                    }.into_any();
                }
                view! {
                    <div>
                        {groups.into_iter().map(|(act_name, zones_in_act)| {
                            let act_for_summary = act_name.clone();
                            let act_for_manage = act_name.clone();
                            let act_for_add = act_name.clone();
                            let act_for_class_check = act_name.clone();
                            let act_for_label_check = act_name.clone();
                            let act_for_expand_click = act_name.clone();
                            let act_for_expand_check = act_name.clone();
                            let act_for_rotation = act_name.clone();
                            // Interludes are the "filler" acts between the main four; they get
                            // a quieter visual treatment so the four main acts dominate the eye.
                            let is_interlude = act_name.starts_with("Interlude:");
                            let title_class = if is_interlude {
                                "text-2xl text-fg-muted group-hover:text-accent m-0 truncate transition-colors"
                            } else {
                                "text-3xl text-fg group-hover:text-accent m-0 truncate transition-colors"
                            };
                            let is_managing_class = move || managing_act.get().as_deref() == Some(act_for_class_check.as_str());
                            let is_managing_label = move || managing_act.get().as_deref() == Some(act_for_label_check.as_str());
                            let is_expanded = move || expanded_acts.with(|set| set.contains(&act_for_expand_check));
                            let chevron_class = move || {
                                let base = "w-3.5 h-3.5 text-fg-muted group-hover:text-accent transition-transform transition-colors";
                                if expanded_acts.with(|set| set.contains(&act_for_rotation)) {
                                    format!("{} rotate-0", base)
                                } else {
                                    format!("{} -rotate-90", base)
                                }
                            };
                            view! {
                                <div>
                                    <div
                                        class="group flex items-baseline justify-between gap-3 border-b border-border px-6 -mx-6 py-4 cursor-pointer hover:bg-black/20 transition"
                                        on:click={
                                            let act = act_for_expand_click.clone();
                                            let toggle = toggle_act_expanded;
                                            move |_| toggle(act.clone())
                                        }
                                    >
                                        <div class="flex items-baseline gap-3 min-w-0 flex-1">
                                            <svg
                                                class=chevron_class
                                                viewBox="0 0 14 14"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                            >
                                                <path d="M3 5 L7 9 L11 5"/>
                                            </svg>
                                            <h3 class=title_class>{act_name}</h3>
                                        </div>
                                        <div class="flex items-center gap-3 shrink-0">
                                            {move || {
                                                let (done, total) = act_summary(act_for_summary.clone());
                                                // Don't show "0 / 0" before any checklist items
                                                // exist — that's noise, not signal.
                                                (total > 0).then(|| view! {
                                                    <span class="text-xs text-fg-muted">
                                                        {format!("{} / {}", done, total)}
                                                    </span>
                                                })
                                            }}
                                            <button
                                                class="inline-flex items-center justify-center h-8 px-3 rounded-md border border-border bg-transparent text-fg-muted hover:text-accent hover:border-accent text-sm transition"
                                                on:click={
                                                    let act = act_for_add.clone();
                                                    move |ev: web_sys::MouseEvent| {
                                                        // Don't let the click bubble up and toggle the act's
                                                        // expand/collapse state.
                                                        ev.stop_propagation();
                                                        open_add_in_act(act.clone());
                                                    }
                                                }
                                                title="Add a zone to this act"
                                            >
                                                "+ Add zone"
                                            </button>
                                            <button
                                                class=move || {
                                                    let base = "inline-flex items-center gap-1 h-8 px-3 rounded-md border text-sm transition";
                                                    if is_managing_class() {
                                                        format!("{} bg-accent text-accent-fg border-accent", base)
                                                    } else {
                                                        format!("{} bg-transparent text-fg-muted border-border hover:text-fg hover:border-fg", base)
                                                    }
                                                }
                                                on:click={
                                                    let act = act_for_manage.clone();
                                                    move |ev: web_sys::MouseEvent| {
                                                        // Don't let the click bubble up and toggle the expand state.
                                                        ev.stop_propagation();
                                                        let target = if managing_act.get_untracked().as_deref() == Some(act.as_str()) {
                                                            None
                                                        } else {
                                                            // Expand the act when entering Manage mode so the
                                                            // edit/delete buttons it surfaces are actually visible.
                                                            set_expanded_acts.update(|set| { set.insert(act.clone()); });
                                                            Some(act.clone())
                                                        };
                                                        set_managing_act.set(target);
                                                    }
                                                }
                                                title="Edit zones in this act"
                                            >
                                                {move || if is_managing_label() {
                                                    view! { "Done" }.into_any()
                                                } else {
                                                    view! { <PencilIcon class="w-3 h-3"/> "Manage" }.into_any()
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                    <Show when=is_expanded>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3 mb-6">
                                            {zones_in_act.clone().into_iter().map(|zone| {
                                                view! {
                                                    <ZoneCard
                                                        zone=zone
                                                        progress_map=progress_map
                                                        managing_act=managing_act
                                                        toggle=toggle_item
                                                        on_edit=open_edit
                                                        on_delete=request_delete
                                                        on_open_notes=move |(zid, zname): (String, String)| {
                                                            open_note_for_zone(app, zid, zname, Template::Blank);
                                                        }
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                    </Show>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}

            <Show when=move || editing.get().is_some()>
                <EditModal
                    draft=editing
                    set_draft=set_editing
                    cancel=cancel_edit
                    confirm=save_draft
                />
            </Show>

            <Show when=move || pending_delete.get().is_some()>
                <DeleteModal cancel=cancel_delete confirm=confirm_delete/>
            </Show>

            <Show when=move || pending_reset.get()>
                <ResetProgressModal cancel=cancel_reset confirm=confirm_reset/>
            </Show>
        </section>
    }
}

#[component]
fn ZoneCard<T, N, E, D>(
    zone: Zone,
    progress_map: ReadSignal<HashMap<String, ZoneProgress>>,
    managing_act: ReadSignal<Option<String>>,
    toggle: T,
    on_edit: E,
    on_delete: D,
    on_open_notes: N,
) -> impl IntoView
where
    T: Fn(String, String) + Copy + Send + Sync + 'static,
    N: Fn((String, String)) + Copy + Send + Sync + 'static,
    E: Fn(Zone) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
{
    let zone_id = zone.id.clone();
    let zone_name = zone.name.clone();
    let zone_name_html = render_inline_md(&zone.name);
    let zone_act = zone.act.clone();
    let zone_for_edit = zone.clone();
    let checklist = zone.checklist.clone();
    let zone_tags = zone.tags.clone();

    let zone_act_for_edit = zone_act.clone();
    let zone_act_for_delete = zone_act.clone();
    let edit_managing = move || managing_act.get().as_deref() == Some(zone_act_for_edit.as_str());
    let delete_managing = move || managing_act.get().as_deref() == Some(zone_act_for_delete.as_str());

    let id_for_delete = zone_id.clone();
    let id_for_open_notes = zone_id.clone();
    let name_for_open_notes = zone_name.clone();

    let edit_btn_class = move || {
        let base = "items-center justify-center w-6 h-6 rounded bg-bg-elevated border border-border text-fg-muted hover:text-fg";
        if edit_managing() {
            format!("inline-flex {}", base)
        } else {
            format!("hidden {}", base)
        }
    };
    let delete_btn_class = move || {
        let base = "items-center justify-center w-6 h-6 rounded bg-red-700 text-white border border-red-700 hover:bg-red-800 hover:border-red-800";
        if delete_managing() {
            format!("inline-flex {}", base)
        } else {
            format!("hidden {}", base)
        }
    };

    view! {
        <div class="rounded-lg border border-border bg-bg p-3 flex flex-col h-full">
            <div class="flex items-baseline justify-between gap-2 mb-3 pb-2 border-b border-border/60">
                <h3 class="text-lg text-accent m-0 truncate" inner_html=zone_name_html></h3>
                <div class="shrink-0 flex items-center gap-1">
                    <button
                        class=edit_btn_class
                        on:click=move |ev: web_sys::MouseEvent| {
                            ev.stop_propagation();
                            on_edit(zone_for_edit.clone());
                        }
                        title="Edit zone"
                    >
                        <PencilIcon class="w-3 h-3"/>
                    </button>
                    <button
                        class=delete_btn_class
                        on:click=move |ev: web_sys::MouseEvent| {
                            ev.stop_propagation();
                            on_delete(id_for_delete.clone());
                        }
                        title="Delete zone"
                    >
                        <TrashIcon class="w-3 h-3"/>
                    </button>
                    <button
                        class="text-xs text-accent hover:underline ml-1"
                        on:click=move |_| on_open_notes((id_for_open_notes.clone(), name_for_open_notes.clone()))
                        title="Open notes for this zone"
                    >
                        "Notes ↗"
                    </button>
                </div>
            </div>
            <div class="flex flex-col gap-1 text-sm flex-1">
                {checklist.into_iter().map(|item| {
                    let item_id = item.id.clone();
                    let zone_id_for_check = zone_id.clone();
                    let zone_id_for_toggle = zone_id.clone();
                    let item_id_for_check = item_id.clone();
                    let item_id_for_toggle = item_id.clone();
                    let checked = Signal::derive(move || {
                        progress_map
                            .get()
                            .get(&zone_id_for_check)
                            .map(|progress| progress.done_items.contains(&item_id_for_check))
                            .unwrap_or(false)
                    });
                    view! {
                        <ChecklistRow
                            label=item.text
                            checked=checked
                            on_toggle=move || toggle(zone_id_for_toggle.clone(), item_id_for_toggle.clone())
                        />
                    }
                }).collect_view()}
            </div>
            {(!zone_tags.is_empty()).then(|| view! {
                <div class="flex flex-wrap gap-1 mt-3 pt-3 border-t border-border/60">
                    {zone_tags.into_iter().map(|tag| {
                        let classes = format!(
                            "text-[10px] uppercase tracking-wide rounded-full border px-1.5 py-0.5 {}",
                            tag_color_classes(&tag),
                        );
                        view! { <span class=classes>{tag}</span> }
                    }).collect_view()}
                </div>
            })}
        </div>
    }
}

#[component]
fn ChecklistRow<F>(label: String, checked: Signal<bool>, on_toggle: F) -> impl IntoView
where
    F: Fn() + Send + Sync + 'static,
{
    let label_html = render_inline_md(&label);
    view! {
        <label class="flex items-center gap-2 cursor-pointer select-none">
            <input
                type="checkbox"
                class="w-4 h-4 accent-accent cursor-pointer"
                prop:checked=move || checked.get()
                on:change=move |_| on_toggle()
            />
            <span
                class=move || {
                    if checked.get() {
                        "text-fg-muted line-through".to_string()
                    } else {
                        "text-fg".to_string()
                    }
                }
                inner_html=label_html
            ></span>
        </label>
    }
}

// ─── UI state persistence ────────────────────────────────────────────────
//
// Stores small, ephemeral UI bits (which acts are open, which tag filters
// are active) in localStorage so a reload doesn't reset them. The data
// isn't critical, isn't worth IDB ceremony, and never gets read by any
// other tab — localStorage is the right level.

const LS_EXPANDED_ACTS: &str = "campaign_expanded_acts";
const LS_SELECTED_TAGS: &str = "campaign_selected_tags";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn read_string_set(key: &str) -> HashSet<String> {
    let Some(storage) = local_storage() else {
        return HashSet::new();
    };
    let Ok(Some(raw)) = storage.get_item(key) else {
        return HashSet::new();
    };
    serde_json::from_str::<Vec<String>>(&raw)
        .map(|list| list.into_iter().collect())
        .unwrap_or_default()
}

fn write_string_set(key: &str, set: &HashSet<String>) {
    let Some(storage) = local_storage() else {
        return;
    };
    let list: Vec<&String> = set.iter().collect();
    if let Ok(json) = serde_json::to_string(&list) {
        let _ = storage.set_item(key, &json);
    }
}
