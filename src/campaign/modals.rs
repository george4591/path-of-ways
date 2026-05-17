use leptos::prelude::*;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

#[derive(Clone, Default)]
pub struct ZoneDraft {
    pub id: Option<String>,
    pub act: String,
    pub name: String,
    pub has_waypoint: bool,
    pub has_side_area: bool,
    pub quest_reward: String,
    pub boss: String,
}

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
    let update_bool = move |mutate: fn(&mut ZoneDraft, bool), value: bool| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                mutate(current, value);
            }
        });
    };

    view! {
        <ModalShell cancel=cancel confirm=confirm panel_class="max-w-md">
            <h3 class="text-lg font-semibold text-fg mb-4">
                {move || if is_edit() { "Edit zone" } else { "Add zone" }}
            </h3>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Act"</span>
                    <input
                        type="text"
                        placeholder="e.g. Act 1: The Riverbank"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|d| d.act).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.act = v, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Zone name"</span>
                    <input
                        type="text"
                        placeholder="e.g. Clearfell"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|d| d.name).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.name = v, event_target_value(&ev))
                    />
                </label>
                <div class="grid grid-cols-2 gap-2">
                    <label class="flex items-center gap-2 cursor-pointer select-none text-sm text-fg">
                        <input
                            type="checkbox"
                            class="w-4 h-4 accent-accent cursor-pointer"
                            prop:checked=move || draft.get().map(|d| d.has_waypoint).unwrap_or(false)
                            on:change=move |ev| update_bool(|d, v| d.has_waypoint = v, event_target_checked(&ev))
                        />
                        "Has waypoint"
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer select-none text-sm text-fg">
                        <input
                            type="checkbox"
                            class="w-4 h-4 accent-accent cursor-pointer"
                            prop:checked=move || draft.get().map(|d| d.has_side_area).unwrap_or(false)
                            on:change=move |ev| update_bool(|d, v| d.has_side_area = v, event_target_checked(&ev))
                        />
                        "Has side area"
                    </label>
                </div>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Quest reward (optional)"</span>
                    <input
                        type="text"
                        placeholder="e.g. Skill gem"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|d| d.quest_reward).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.quest_reward = v, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Boss (optional)"</span>
                    <input
                        type="text"
                        placeholder="e.g. Count Geonor"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|d| d.boss).unwrap_or_default()
                        on:input=move |ev| update_text(|d, v| d.boss = v, event_target_value(&ev))
                    />
                </label>
            </div>
            <div class="flex justify-end gap-2 mt-4">
                <SecondaryButton on_click=move |_| cancel()>"Cancel"</SecondaryButton>
                <PrimaryButton on_click=move |_| confirm()>"Save"</PrimaryButton>
            </div>
        </ModalShell>
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
