use std::collections::{BTreeMap, HashMap};

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::app_state::{open_note_for_zone, use_app_state};
use crate::buttons::PrimaryButton;
use crate::icons::{PencilIcon, TrashIcon};
use crate::notes::{render_inline_md, Template};

use super::modals::{DeleteModal, EditModal, ZoneDraft};
use super::model::{Zone, ZoneProgress};
use super::storage::{
    delete_zone, load_progress, load_zones, save_zone, save_zone_progress,
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

    let toggle_progress = move |zone_id: String, field: ProgressField| {
        let mut updated: Option<ZoneProgress> = None;
        set_progress_map.update(|map| {
            let progress = map
                .entry(zone_id.clone())
                .or_insert_with(|| ZoneProgress::for_zone(&zone_id));
            match field {
                ProgressField::Waypoint => progress.waypoint_done = !progress.waypoint_done,
                ProgressField::SideArea => progress.side_area_done = !progress.side_area_done,
                ProgressField::QuestReward => {
                    progress.quest_reward_done = !progress.quest_reward_done;
                }
                ProgressField::Boss => progress.boss_done = !progress.boss_done,
            }
            updated = Some(progress.clone());
        });
        if let Some(progress) = updated {
            spawn_local(async move {
                save_zone_progress(progress).await;
            });
        }
    };

    let grouped = move || -> BTreeMap<String, Vec<Zone>> {
        let mut map: BTreeMap<String, Vec<Zone>> = BTreeMap::new();
        let mut list = zones.get();
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

    let act_summary = move |act_name: String| -> (usize, usize) {
        let map = progress_map.get();
        let zones_list = zones.get();
        let mut total = 0usize;
        let mut done = 0usize;
        for zone in zones_list.iter().filter(|zone| zone.act == act_name) {
            if zone.has_waypoint {
                total += 1;
            }
            if zone.has_side_area {
                total += 1;
            }
            if zone.quest_reward.is_some() {
                total += 1;
            }
            if zone.boss.is_some() {
                total += 1;
            }
            if let Some(progress) = map.get(&zone.id) {
                if zone.has_waypoint && progress.waypoint_done {
                    done += 1;
                }
                if zone.has_side_area && progress.side_area_done {
                    done += 1;
                }
                if zone.quest_reward.is_some() && progress.quest_reward_done {
                    done += 1;
                }
                if zone.boss.is_some() && progress.boss_done {
                    done += 1;
                }
            }
        }
        (done, total)
    };

    let open_add = move |_| set_editing.set(Some(ZoneDraft::default()));

    let open_edit = move |zone: Zone| {
        set_editing.set(Some(ZoneDraft {
            id: Some(zone.id),
            act: zone.act,
            name: zone.name,
            has_waypoint: zone.has_waypoint,
            has_side_area: zone.has_side_area,
            quest_reward: zone.quest_reward.unwrap_or_default(),
            boss: zone.boss.unwrap_or_default(),
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
        let quest_reward = {
            let trimmed = draft.quest_reward.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };
        let boss = {
            let trimmed = draft.boss.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };

        match draft.id {
            Some(id) => {
                let id_match = id.clone();
                let mut updated: Option<Zone> = None;
                set_zones.update(|list| {
                    if let Some(zone) = list.iter_mut().find(|candidate| candidate.id == id_match) {
                        zone.act = act.clone();
                        zone.name = name.clone();
                        zone.has_waypoint = draft.has_waypoint;
                        zone.has_side_area = draft.has_side_area;
                        zone.quest_reward = quest_reward.clone();
                        zone.boss = boss.clone();
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
                let zone = Zone::new(
                    act,
                    name,
                    draft.has_waypoint,
                    draft.has_side_area,
                    quest_reward,
                    boss,
                );
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

    view! {
        <section class="rounded-xl border border-border bg-bg-elevated p-6 h-[calc(100vh-7.75rem)] min-h-[28rem] overflow-auto">
            <div class="flex items-start justify-between gap-3 mb-6">
                <div>
                    <h2 class="text-2xl font-semibold text-fg m-0 mb-1">"Campaign"</h2>
                    <p class="text-sm text-fg-muted m-0">"Zone progress per act. Add your own zones — they're saved locally."</p>
                </div>
                <PrimaryButton on_click=open_add>"+ Add zone"</PrimaryButton>
            </div>

            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let groups = grouped();
                if groups.is_empty() {
                    view! {
                        <div class="text-center text-fg-muted py-16">
                            <p class="m-0 mb-3">"No zones yet."</p>
                            <PrimaryButton on_click=open_add>"+ Add your first zone"</PrimaryButton>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-8">
                            {groups.into_iter().map(|(act_name, zones_in_act)| {
                                let act_for_summary = act_name.clone();
                                let act_for_toggle = act_name.clone();
                                let act_for_class_check = act_name.clone();
                                let act_for_label_check = act_name.clone();
                                let is_managing_class = move || managing_act.get().as_deref() == Some(act_for_class_check.as_str());
                                let is_managing_label = move || managing_act.get().as_deref() == Some(act_for_label_check.as_str());
                                view! {
                                    <div>
                                        <div class="flex items-baseline justify-between gap-3 border-b border-border pb-2 mb-3">
                                            <h3 class="text-2xl font-semibold text-fg m-0">{act_name}</h3>
                                            <div class="flex items-center gap-3">
                                                <span class="text-xs text-fg-muted">
                                                    {move || {
                                                        let (done, total) = act_summary(act_for_summary.clone());
                                                        format!("{} / {}", done, total)
                                                    }}
                                                </span>
                                                <button
                                                    class=move || {
                                                        let base = "inline-flex items-center gap-1 h-7 px-2 rounded-md border text-xs transition";
                                                        if is_managing_class() {
                                                            format!("{} bg-accent text-accent-fg border-accent", base)
                                                        } else {
                                                            format!("{} bg-transparent text-fg-muted border-border hover:text-fg hover:border-fg", base)
                                                        }
                                                    }
                                                    on:click={
                                                        let act = act_for_toggle.clone();
                                                        move |_| {
                                                            let target = if managing_act.get_untracked().as_deref() == Some(act.as_str()) {
                                                                None
                                                            } else {
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
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                            {zones_in_act.into_iter().map(|zone| {
                                                view! {
                                                    <ZoneCard
                                                        zone=zone
                                                        progress_map=progress_map
                                                        managing_act=managing_act
                                                        toggle=toggle_progress
                                                        on_edit=open_edit
                                                        on_delete=request_delete
                                                        on_open_notes=move |(zid, zname): (String, String)| {
                                                            open_note_for_zone(app, zid, zname, Template::Blank);
                                                        }
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
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
        </section>
    }
}

#[derive(Clone, Copy)]
enum ProgressField {
    Waypoint,
    SideArea,
    QuestReward,
    Boss,
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
    T: Fn(String, ProgressField) + Copy + Send + Sync + 'static,
    N: Fn((String, String)) + Copy + Send + Sync + 'static,
    E: Fn(Zone) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
{
    let zone_id = zone.id.clone();
    let zone_name = zone.name.clone();
    let zone_name_html = render_inline_md(&zone.name);
    let zone_act = zone.act.clone();
    let zone_for_edit = zone.clone();

    let zone_act_for_edit = zone_act.clone();
    let zone_act_for_delete = zone_act.clone();
    let edit_managing = move || managing_act.get().as_deref() == Some(zone_act_for_edit.as_str());
    let delete_managing = move || managing_act.get().as_deref() == Some(zone_act_for_delete.as_str());

    let id_for_delete = zone_id.clone();
    let id_for_open_notes = zone_id.clone();
    let name_for_open_notes = zone_name.clone();
    let id_for_wp_check = zone_id.clone();
    let id_for_wp_toggle = zone_id.clone();
    let id_for_sa_check = zone_id.clone();
    let id_for_sa_toggle = zone_id.clone();
    let id_for_qr_check = zone_id.clone();
    let id_for_qr_toggle = zone_id.clone();
    let id_for_boss_check = zone_id.clone();
    let id_for_boss_toggle = zone_id.clone();

    let has_waypoint = zone.has_waypoint;
    let has_side_area = zone.has_side_area;
    let quest_reward_for_show = zone.quest_reward.clone();
    let quest_reward_label = zone.quest_reward.clone();
    let boss_for_show = zone.boss.clone();
    let boss_label = zone.boss.clone();

    let lookup_bool = move |id: String, getter: fn(&ZoneProgress) -> bool| -> Signal<bool> {
        Signal::derive(move || {
            progress_map
                .get()
                .get(&id)
                .map(getter)
                .unwrap_or(false)
        })
    };

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
        <div class="rounded-lg border border-border bg-bg p-3">
            <div class="flex items-baseline justify-between gap-2 mb-2">
                <h3 class="text-base font-medium text-fg m-0 truncate" inner_html=zone_name_html></h3>
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
            <div class="flex flex-col gap-1 text-sm">
                <Show when=move || has_waypoint>
                    <ChecklistItem
                        label="Waypoint".to_string()
                        checked=lookup_bool(id_for_wp_check.clone(), |p| p.waypoint_done)
                        on_toggle={
                            let id = id_for_wp_toggle.clone();
                            move |_| toggle(id.clone(), ProgressField::Waypoint)
                        }
                    />
                </Show>
                <Show when=move || has_side_area>
                    <ChecklistItem
                        label="Side area".to_string()
                        checked=lookup_bool(id_for_sa_check.clone(), |p| p.side_area_done)
                        on_toggle={
                            let id = id_for_sa_toggle.clone();
                            move |_| toggle(id.clone(), ProgressField::SideArea)
                        }
                    />
                </Show>
                <Show when=move || quest_reward_for_show.is_some()>
                    <ChecklistItem
                        label=format!("Quest reward: {}", quest_reward_label.clone().unwrap_or_default())
                        checked=lookup_bool(id_for_qr_check.clone(), |p| p.quest_reward_done)
                        on_toggle={
                            let id = id_for_qr_toggle.clone();
                            move |_| toggle(id.clone(), ProgressField::QuestReward)
                        }
                    />
                </Show>
                <Show when=move || boss_for_show.is_some()>
                    <ChecklistItem
                        label=format!("Boss: {}", boss_label.clone().unwrap_or_default())
                        checked=lookup_bool(id_for_boss_check.clone(), |p| p.boss_done)
                        on_toggle={
                            let id = id_for_boss_toggle.clone();
                            move |_| toggle(id.clone(), ProgressField::Boss)
                        }
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn ChecklistItem<F>(label: String, checked: Signal<bool>, on_toggle: F) -> impl IntoView
where
    F: Fn(()) + Send + Sync + 'static,
{
    let label_html = render_inline_md(&label);
    view! {
        <label class="flex items-center gap-2 cursor-pointer select-none">
            <input
                type="checkbox"
                class="w-4 h-4 accent-accent cursor-pointer"
                prop:checked=move || checked.get()
                on:change=move |_| on_toggle(())
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
