use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::buttons::PrimaryButton;
use crate::external::open_external;
use crate::icons::{PencilIcon, TrashIcon};
use crate::notes::render_inline_md;

use super::modals::{DeleteModal, EditModal, LinkDraft};
use super::model::Link;
use super::storage::{delete_link, load_links, save_link};

#[component]
fn LinkCard<O, E, D>(link: Link, on_open: O, on_edit: E, on_delete: D) -> impl IntoView
where
    O: Fn(String) + Copy + Send + Sync + 'static,
    E: Fn(Link) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
{
    let url = link.url.clone();
    let link_for_edit = link.clone();
    let id_for_delete = link.id.clone();
    let has_description = !link.description.is_empty();
    let title_html = render_inline_md(&link.title);
    let description_html = render_inline_md(&link.description);

    view! {
        <div
            class="relative group rounded-lg border border-border bg-bg p-3 hover:border-accent transition cursor-pointer"
            on:click=move |_| on_open(url.clone())
        >
            <div class="flex items-start gap-3">
                <img
                    src=link.favicon_url()
                    alt=""
                    class="w-8 h-8 rounded shrink-0 mt-0.5"
                    loading="lazy"
                />
                <div class="min-w-0 flex-1">
                    <div class="font-medium text-fg truncate" inner_html=title_html></div>
                    <div class="text-xs text-fg-muted truncate">{link.domain().to_string()}</div>
                </div>
            </div>
            {has_description.then(|| view! {
                <div class="text-sm text-fg-muted mt-2 line-clamp-2" inner_html=description_html></div>
            })}
            <div class="absolute top-2 right-2 hidden group-hover:flex gap-0.5">
                <button
                    class="inline-flex items-center justify-center w-6 h-6 rounded bg-bg-elevated border border-border text-fg-muted hover:text-fg"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        on_edit(link_for_edit.clone());
                    }
                    title="Edit"
                >
                    <PencilIcon class="w-3 h-3"/>
                </button>
                <button
                    class="inline-flex items-center justify-center w-6 h-6 rounded bg-red-700 text-white border border-red-700 hover:bg-red-800 hover:border-red-800"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        on_delete(id_for_delete.clone());
                    }
                    title="Delete"
                >
                    <TrashIcon class="w-3 h-3"/>
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn Links() -> impl IntoView {
    let (links, set_links) = signal(Vec::<Link>::new());
    let (editing, set_editing) = signal::<Option<LinkDraft>>(None);
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        let list = load_links().await;
        set_links.set(list);
        set_loaded.set(true);
    });

    let sorted = move || {
        let mut list = links.get();
        list.sort_by(|a, b| {
            b.created_at
                .partial_cmp(&a.created_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        list
    };

    let open_add = move |_| set_editing.set(Some(LinkDraft::default()));
    let open_edit = move |link: Link| {
        set_editing.set(Some(LinkDraft {
            id: Some(link.id),
            url: link.url,
            title: link.title,
            description: link.description,
        }));
    };
    let cancel_edit = move || set_editing.set(None);

    let save_draft = move || {
        let Some(draft) = editing.get_untracked() else {
            return;
        };
        let url = draft.url.trim().to_string();
        let title_input = draft.title.trim().to_string();
        if url.is_empty() {
            return;
        }
        let url_with_scheme = if url.starts_with("http://") || url.starts_with("https://") {
            url
        } else {
            format!("https://{}", url)
        };
        match draft.id.as_ref() {
            Some(id) => {
                let id_match = id.clone();
                let mut updated: Option<Link> = None;
                set_links.update(|list| {
                    if let Some(l) = list.iter_mut().find(|l| l.id == id_match) {
                        l.url = url_with_scheme.clone();
                        l.title = if title_input.is_empty() {
                            l.domain().to_string()
                        } else {
                            title_input.clone()
                        };
                        l.description = draft.description.clone();
                        updated = Some(l.clone());
                    }
                });
                if let Some(l) = updated {
                    spawn_local(async move {
                        save_link(l).await;
                    });
                }
            }
            None => {
                let title = if title_input.is_empty() {
                    let tmp = Link::new(url_with_scheme.clone(), String::new(), String::new());
                    tmp.domain().to_string()
                } else {
                    title_input
                };
                let link = Link::new(url_with_scheme, title, draft.description);
                let to_save = link.clone();
                set_links.update(|list| list.push(link));
                spawn_local(async move {
                    save_link(to_save).await;
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
        set_links.update(|list| list.retain(|l| l.id != id));
        spawn_local(async move {
            delete_link(&id_for_delete).await;
        });
        set_pending_delete.set(None);
    };

    let open_link = move |url: String| {
        open_external(&url);
    };

    view! {
        <section class="p-6 h-[calc(100vh-2.25rem)] min-h-[28rem] overflow-auto">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h2 class="text-2xl font-semibold text-fg m-0 mb-1">"Links"</h2>
                    <p class="text-sm text-fg-muted m-0">"Useful sites for builds, trade, references."</p>
                </div>
                <PrimaryButton on_click=open_add>"+ Add link"</PrimaryButton>
            </div>

            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let list = sorted();
                if list.is_empty() {
                    view! {
                        <div class="text-center text-fg-muted py-16">
                            <p class="m-0 mb-3">"No links yet."</p>
                            <PrimaryButton on_click=open_add>"+ Add your first link"</PrimaryButton>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                            {list.into_iter().map(|link| view! {
                                <LinkCard
                                    link=link
                                    on_open=open_link
                                    on_edit=open_edit
                                    on_delete=request_delete
                                />
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
