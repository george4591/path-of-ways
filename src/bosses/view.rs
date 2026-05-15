use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::app_state::{open_note_by_title, use_app_state};
use crate::buttons::PrimaryButton;
use crate::icons::{PencilIcon, TrashIcon};
use crate::notes::render_inline_md;

use super::modals::{BossDraft, DeleteModal, EditModal};
use super::model::Boss;
use super::storage::{delete_boss, load_bosses, save_boss};

#[component]
pub fn Bosses() -> impl IntoView {
    let app = use_app_state();
    let (bosses, set_bosses) = signal(Vec::<Boss>::new());
    let (editing, set_editing) = signal::<Option<BossDraft>>(None);
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        set_bosses.set(load_bosses().await);
        set_loaded.set(true);
    });

    let grouped = move || -> BTreeMap<String, Vec<Boss>> {
        let mut map: BTreeMap<String, Vec<Boss>> = BTreeMap::new();
        let mut list = bosses.get();
        list.sort_by(|left, right| {
            left.created_at
                .partial_cmp(&right.created_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for boss in list {
            map.entry(boss.category.clone()).or_default().push(boss);
        }
        map
    };

    let open_add = move |_| set_editing.set(Some(BossDraft::default()));

    let open_edit = move |boss: Boss| {
        set_editing.set(Some(BossDraft {
            id: Some(boss.id),
            name: boss.name,
            category: boss.category,
            zone: boss.zone,
            description: boss.description,
        }));
    };

    let cancel_edit = move || set_editing.set(None);

    let save_draft = move || {
        let Some(draft) = editing.get_untracked() else {
            return;
        };
        let name = draft.name.trim().to_string();
        if name.is_empty() {
            return;
        }
        let category = {
            let trimmed = draft.category.trim();
            if trimmed.is_empty() {
                "Misc".to_string()
            } else {
                trimmed.to_string()
            }
        };
        let zone = draft.zone.trim().to_string();
        let description = draft.description.trim().to_string();

        match draft.id {
            Some(id) => {
                let id_match = id.clone();
                let mut updated: Option<Boss> = None;
                set_bosses.update(|list| {
                    if let Some(boss) = list.iter_mut().find(|candidate| candidate.id == id_match) {
                        boss.name = name.clone();
                        boss.category = category.clone();
                        boss.zone = zone.clone();
                        boss.description = description.clone();
                        updated = Some(boss.clone());
                    }
                });
                if let Some(boss) = updated {
                    spawn_local(async move {
                        save_boss(boss).await;
                    });
                }
            }
            None => {
                let boss = Boss::new(name, category, zone, description);
                let to_save = boss.clone();
                set_bosses.update(|list| list.push(boss));
                spawn_local(async move {
                    save_boss(to_save).await;
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
        set_bosses.update(|list| list.retain(|boss| boss.id != id));
        spawn_local(async move {
            delete_boss(id_for_delete).await;
        });
        set_pending_delete.set(None);
    };

    let open_boss_note = move |name: String| {
        open_note_by_title(app, name);
    };

    view! {
        <section class="rounded-xl border border-border bg-bg-elevated p-6 h-[calc(100vh-7.75rem)] min-h-[28rem] overflow-auto">
            <div class="flex items-start justify-between gap-3 mb-6">
                <div>
                    <h2 class="text-2xl font-semibold text-fg m-0 mb-1">"Bosses"</h2>
                    <p class="text-sm text-fg-muted m-0">"Add campaign and pinnacle bosses. Click 'Notes ↗' to jump to a strategy note (created on first click)."</p>
                </div>
                <PrimaryButton on_click=open_add>"+ Add boss"</PrimaryButton>
            </div>

            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let groups = grouped();
                if groups.is_empty() {
                    view! {
                        <div class="text-center text-fg-muted py-16">
                            <p class="m-0 mb-3">"No bosses yet."</p>
                            <PrimaryButton on_click=open_add>"+ Add your first boss"</PrimaryButton>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-6">
                            {groups.into_iter().map(|(category, list_in_category)| {
                                view! {
                                    <div>
                                        <h3 class="text-lg font-medium text-fg-muted m-0 mb-2">{category}</h3>
                                        <ul class="flex flex-col gap-2">
                                            {list_in_category.into_iter().map(|boss| {
                                                view! {
                                                    <BossCard
                                                        boss=boss
                                                        on_edit=open_edit
                                                        on_delete=request_delete
                                                        on_open_notes=open_boss_note
                                                    />
                                                }
                                            }).collect_view()}
                                        </ul>
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

#[component]
fn BossCard<E, D, N>(
    boss: Boss,
    on_edit: E,
    on_delete: D,
    on_open_notes: N,
) -> impl IntoView
where
    E: Fn(Boss) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
    N: Fn(String) + Copy + Send + Sync + 'static,
{
    let boss_for_edit = boss.clone();
    let id_for_delete = boss.id.clone();
    let name_for_notes = boss.name.clone();
    let name_html = render_inline_md(&boss.name);
    let zone_present = !boss.zone.is_empty();
    let zone_html = render_inline_md(&boss.zone);
    let description_present = !boss.description.is_empty();
    let description_html = render_inline_md(&boss.description);

    view! {
        <li class="group flex items-start justify-between gap-3 rounded-md border border-border bg-bg p-3">
            <div class="min-w-0 flex-1">
                <div class="font-medium text-fg truncate" inner_html=name_html></div>
                {zone_present.then(|| view! {
                    <div class="text-xs text-fg-muted truncate" inner_html=zone_html></div>
                })}
                {description_present.then(|| view! {
                    <div class="text-sm text-fg-muted mt-1" inner_html=description_html></div>
                })}
            </div>
            <div class="shrink-0 flex items-center gap-1">
                <button
                    class="hidden group-hover:inline-flex items-center justify-center w-7 h-7 rounded bg-bg-elevated border border-border text-fg-muted hover:text-fg"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        on_edit(boss_for_edit.clone());
                    }
                    title="Edit boss"
                >
                    <PencilIcon class="w-3.5 h-3.5"/>
                </button>
                <button
                    class="hidden group-hover:inline-flex items-center justify-center w-7 h-7 rounded bg-red-700 text-white border border-red-700 hover:bg-red-800 hover:border-red-800"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        on_delete(id_for_delete.clone());
                    }
                    title="Delete boss"
                >
                    <TrashIcon class="w-3.5 h-3.5"/>
                </button>
                <button
                    class="inline-flex items-center h-8 px-3 rounded-md border border-accent text-accent hover:bg-accent hover:text-accent-fg text-xs transition"
                    on:click=move |_| on_open_notes(name_for_notes.clone())
                    title="Open / create a strategy note for this boss"
                >
                    "Notes ↗"
                </button>
            </div>
        </li>
    }
}
