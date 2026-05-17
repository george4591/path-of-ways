use leptos::prelude::*;

use crate::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use crate::icons::TrashIcon;
use crate::modal::ModalShell;

#[component]
pub fn DeleteModal<C, K>(cancel: C, confirm: K) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <ModalShell cancel=cancel confirm=confirm>
            <h3 class="text-lg font-semibold text-fg mb-2">"Delete this note?"</h3>
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

#[component]
pub fn ImportModal<C, K>(summary: Signal<(usize, usize)>, cancel: C, confirm: K) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
    K: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <ModalShell cancel=cancel confirm=confirm>
            <h3 class="text-lg font-semibold text-fg mb-2">"Merge imported notes?"</h3>
            <p class="text-sm text-fg-muted mb-4">
                {move || {
                    let (new, updated) = summary.get();
                    format!(
                        "{} new note(s) will be added. {} existing note(s) will be overwritten by the file. Other notes will not be touched.",
                        new, updated
                    )
                }}
            </p>
            <div class="flex justify-end gap-2">
                <SecondaryButton on_click=move |_| cancel()>"Cancel"</SecondaryButton>
                <PrimaryButton on_click=move |_| confirm()>"Merge"</PrimaryButton>
            </div>
        </ModalShell>
    }
}
