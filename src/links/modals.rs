use leptos::prelude::*;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

const FIELD_INPUT_CLASS: &str = "rounded-md border border-border bg-bg px-3 py-2 text-fg placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent text-sm";

/// In-flight form values for adding or editing a link. `id` is `Some` when
/// editing an existing link, `None` for a new one.
#[derive(Clone, Default)]
pub struct LinkDraft {
    pub id: Option<String>,
    pub url: String,
    pub title: String,
    pub description: String,
}

#[component]
fn LabeledField(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <label class="flex flex-col gap-1">
            <span class="text-xs text-fg-muted">{label}</span>
            {children()}
        </label>
    }
}

#[component]
pub fn EditModal<C, K>(
    draft: ReadSignal<Option<LinkDraft>>,
    set_draft: WriteSignal<Option<LinkDraft>>,
    cancel: C,
    confirm: K,
) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    let is_edit = move || draft.get().and_then(|current| current.id).is_some();

    let update_field = move |mutate: fn(&mut LinkDraft, String), value: String| {
        set_draft.update(|slot| {
            if let Some(current) = slot {
                mutate(current, value);
            }
        });
    };

    view! {
        <ModalShell cancel=cancel panel_class="max-w-md">
            <h3 class="text-lg font-semibold text-fg mb-4">
                {move || if is_edit() { "Edit link" } else { "Add link" }}
            </h3>
            <div class="flex flex-col gap-3">
                <LabeledField label="URL">
                    <input
                        type="text"
                        placeholder="https://example.com"
                        class=FIELD_INPUT_CLASS
                        prop:value=move || draft.get().map(|current| current.url).unwrap_or_default()
                        on:input=move |ev| update_field(|d, value| d.url = value, event_target_value(&ev))
                    />
                </LabeledField>
                <LabeledField label="Title">
                    <input
                        type="text"
                        placeholder="Defaults to the domain"
                        class=FIELD_INPUT_CLASS
                        prop:value=move || draft.get().map(|current| current.title).unwrap_or_default()
                        on:input=move |ev| update_field(|d, value| d.title = value, event_target_value(&ev))
                    />
                </LabeledField>
                <LabeledField label="Description (optional)">
                    <textarea
                        class=format!("{} resize-none", FIELD_INPUT_CLASS)
                        rows="3"
                        placeholder="A short note about this site"
                        prop:value=move || draft.get().map(|current| current.description).unwrap_or_default()
                        on:input=move |ev| update_field(|d, value| d.description = value, event_target_value(&ev))
                    />
                </LabeledField>
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
        <ModalShell cancel=cancel>
            <h3 class="text-lg font-semibold text-fg mb-2">"Delete this link?"</h3>
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
