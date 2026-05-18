use leptos::prelude::*;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

/// In-flight form values for adding or editing a recipe. `id` is `Some` when
/// editing, `None` for a new recipe.
#[derive(Clone, Default)]
pub struct RecipeDraft {
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    /// Newline-separated ingredient list (one per line).
    pub ingredients: String,
    pub result: String,
    pub note: String,
}

#[component]
pub fn EditModal<C, K>(
    draft: ReadSignal<Option<RecipeDraft>>,
    set_draft: WriteSignal<Option<RecipeDraft>>,
    cancel: C,
    confirm: K,
) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    let is_edit = move || draft.get().and_then(|current| current.id).is_some();

    let update_field = move |mutate: fn(&mut RecipeDraft, String), value: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                mutate(current, value);
            }
        });
    };

    view! {
        <ModalShell cancel=cancel confirm=confirm panel_class="max-w-md">
            <h3 class="text-lg font-semibold text-fg mb-4">
                {move || if is_edit() { "Edit recipe" } else { "Add recipe" }}
            </h3>
            <div class="flex flex-col gap-3">
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Name"</span>
                    <input
                        type="text"
                        placeholder="e.g. Transmutation Orb"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|current| current.name).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.name = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Category"</span>
                    <input
                        type="text"
                        placeholder="Currency · Equipment · Maps · Misc · …"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|current| current.category).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.category = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Ingredients (one per line)"</span>
                    <textarea
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm resize-none font-mono"
                        rows="4"
                        placeholder="20 Transmutation Shards"
                        prop:value=move || draft.get().map(|current| current.ingredients).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.ingredients = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Result"</span>
                    <input
                        type="text"
                        placeholder="1 Transmutation Orb"
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm"
                        prop:value=move || draft.get().map(|current| current.result).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.result = value, event_target_value(&ev))
                    />
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-sm text-fg-muted">"Note (optional)"</span>
                    <textarea
                        class="rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm resize-none"
                        rows="2"
                        placeholder="Anything worth remembering"
                        prop:value=move || draft.get().map(|current| current.note).unwrap_or_default()
                        on:input=move |ev| update_field(|draft, value| draft.note = value, event_target_value(&ev))
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
            <h3 class="text-lg font-semibold text-fg mb-2">"Delete this recipe?"</h3>
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
