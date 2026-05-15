use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn PrimaryButton<F>(
    on_click: F,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + Copy + Send + Sync + 'static,
{
    let computed = format!(
        "inline-flex items-center justify-center gap-1.5 h-9 px-3 rounded-md bg-accent text-accent-fg hover:opacity-90 transition text-sm {}",
        class
    );
    view! {
        <button class=computed on:click=on_click title=title>
            {children()}
        </button>
    }
}

#[component]
pub fn SecondaryButton<F>(
    on_click: F,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + Copy + Send + Sync + 'static,
{
    let computed = format!(
        "inline-flex items-center justify-center gap-1.5 h-9 px-3 rounded-md border border-border bg-transparent text-fg hover:bg-fg hover:text-bg transition text-sm {}",
        class
    );
    view! {
        <button class=computed on:click=on_click title=title>
            {children()}
        </button>
    }
}

#[component]
pub fn DangerButton<F>(
    on_click: F,
    #[prop(optional, into)] title: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + Copy + Send + Sync + 'static,
{
    let computed = format!(
        "inline-flex items-center justify-center gap-1.5 h-9 px-3 rounded-md border border-red-700 bg-red-700 text-white hover:bg-red-800 hover:border-red-800 transition text-sm {}",
        class
    );
    view! {
        <button class=computed on:click=on_click title=title>
            {children()}
        </button>
    }
}
