use leptos::prelude::*;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

#[derive(Clone, Default)]
pub struct BossDraft {
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    pub zone: String,
    pub description: String,
}

#[component]
pub fn EditModal<C, K>(
    draft: ReadSignal<Option<BossDraft>>,
    set_draft: WriteSignal<Option<BossDraft>>,
    cancel: C,
    confirm: K,
) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    let is_edit = move || draft.get().and_then(|current| current.id).is_some();

    let update_field = move |mutate: fn(&mut BossDraft, String), value: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                mutate(current, value);
            }
        });
    };

    view! {
        <ModalShell cancel=cancel confirm=confirm panel_class="max-w-md">
            <h3 class="text-lg font-semibold text-fg mb-4">
                {move || if is_edit() { "Edit boss" } else { "Add boss" }}
            </h3>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Name"</span>
                    <input
                        type="text"
                        placeholder="e.g. Count Geonor"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|draft| draft.name).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.name = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Category"</span>
                    <input
                        type="text"
                        placeholder="Act 1 · Cruel Act 2 · Pinnacle · Atlas · …"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|draft| draft.category).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.category = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Zone (optional)"</span>
                    <input
                        type="text"
                        placeholder="e.g. The Manor Ramparts"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|draft| draft.zone).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.zone = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-fg-muted">"Description (optional)"</span>
                    <textarea
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm resize-none"
                        rows="3"
                        placeholder="Mechanics, gear notes, anything quick to glance at"
                        prop:value=move || draft.get().map(|draft| draft.description).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.description = value, event_target_value(&ev))
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
            <h3 class="text-lg font-semibold text-fg mb-2">"Delete this boss?"</h3>
            <p class="text-sm text-fg-muted mb-4">"This can't be undone."</p>
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
