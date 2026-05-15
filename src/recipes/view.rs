use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::buttons::PrimaryButton;
use crate::icons::{PencilIcon, TrashIcon};
use crate::notes::render_inline_md;

use super::modals::{DeleteModal, EditModal, RecipeDraft};
use super::model::Recipe;
use super::storage::{delete_recipe, load_recipes, save_recipe};

#[component]
pub fn Recipes() -> impl IntoView {
    let (recipes, set_recipes) = signal(Vec::<Recipe>::new());
    let (editing, set_editing) = signal::<Option<RecipeDraft>>(None);
    let (pending_delete, set_pending_delete) = signal::<Option<String>>(None);
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        let list = load_recipes().await;
        set_recipes.set(list);
        set_loaded.set(true);
    });

    let grouped = move || {
        let mut map: BTreeMap<String, Vec<Recipe>> = BTreeMap::new();
        for recipe in recipes.get() {
            map.entry(recipe.category.clone())
                .or_default()
                .push(recipe);
        }
        map
    };

    let open_add = move |_| set_editing.set(Some(RecipeDraft::default()));

    let open_edit = move |recipe: Recipe| {
        set_editing.set(Some(RecipeDraft {
            id: Some(recipe.id),
            name: recipe.name,
            category: recipe.category,
            ingredients: recipe.ingredients.join("\n"),
            result: recipe.result,
            note: recipe.note.unwrap_or_default(),
        }));
    };

    let cancel_edit = move || set_editing.set(None);

    let save_draft = move || {
        let Some(draft) = editing.get_untracked() else {
            return;
        };
        let name = draft.name.trim().to_string();
        let result = draft.result.trim().to_string();
        if name.is_empty() || result.is_empty() {
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
        let ingredients: Vec<String> = draft
            .ingredients
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        let note = {
            let trimmed = draft.note.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };

        match draft.id {
            Some(id) => {
                let id_match = id.clone();
                let mut updated: Option<Recipe> = None;
                set_recipes.update(|list| {
                    if let Some(recipe) = list.iter_mut().find(|candidate| candidate.id == id_match) {
                        recipe.name = name.clone();
                        recipe.category = category.clone();
                        recipe.ingredients = ingredients.clone();
                        recipe.result = result.clone();
                        recipe.note = note.clone();
                        updated = Some(recipe.clone());
                    }
                });
                if let Some(recipe) = updated {
                    spawn_local(async move {
                        save_recipe(recipe).await;
                    });
                }
            }
            None => {
                let recipe = Recipe::new(name, category, ingredients, result, note);
                let to_save = recipe.clone();
                set_recipes.update(|list| list.push(recipe));
                spawn_local(async move {
                    save_recipe(to_save).await;
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
        set_recipes.update(|list| list.retain(|recipe| recipe.id != id));
        spawn_local(async move {
            delete_recipe(id_for_delete).await;
        });
        set_pending_delete.set(None);
    };

    view! {
        <section class="rounded-xl border border-border bg-bg-elevated p-6 h-[calc(100vh-7.75rem)] min-h-[28rem] overflow-auto">
            <div class="flex items-start justify-between gap-3 mb-6">
                <div>
                    <h2 class="text-2xl font-semibold text-fg m-0 mb-1">"Vendor recipes"</h2>
                    <p class="text-sm text-fg-muted m-0">"Add your own — recipes are saved locally."</p>
                </div>
                <PrimaryButton on_click=open_add>"+ Add recipe"</PrimaryButton>
            </div>

            {move || {
                if !loaded.get() {
                    return view! { <div/> }.into_any();
                }
                let groups = grouped();
                if groups.is_empty() {
                    view! {
                        <div class="text-center text-fg-muted py-16">
                            <p class="m-0 mb-3">"No recipes yet."</p>
                            <PrimaryButton on_click=open_add>"+ Add your first recipe"</PrimaryButton>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-8">
                            {groups.into_iter().map(|(category, recipes_in_category)| {
                                view! {
                                    <div>
                                        <h3 class="text-lg font-medium text-fg-muted m-0 mb-3 pb-1 border-b border-border">
                                            {category}
                                        </h3>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                            {recipes_in_category.into_iter().map(|recipe| {
                                                view! {
                                                    <RecipeCard
                                                        recipe=recipe
                                                        on_edit=open_edit
                                                        on_delete=request_delete
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

#[component]
fn RecipeCard<E, D>(recipe: Recipe, on_edit: E, on_delete: D) -> impl IntoView
where
    E: Fn(Recipe) + Copy + Send + Sync + 'static,
    D: Fn(String) + Copy + Send + Sync + 'static,
{
    let recipe_for_edit = recipe.clone();
    let id_for_delete = recipe.id.clone();
    let name_html = render_inline_md(&recipe.name);
    let result_html = render_inline_md(&recipe.result);
    let ingredients_html: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|ingredient| render_inline_md(ingredient))
        .collect();
    let note_html = recipe.note.as_ref().map(|text| render_inline_md(text));

    view! {
        <div class="relative group rounded-lg border border-border bg-bg p-3">
            <div class="font-medium text-fg mb-2 pr-12" inner_html=name_html></div>
            <ul class="text-sm text-fg-muted m-0 mb-2 list-disc pl-5">
                {ingredients_html.into_iter().map(|html| view! {
                    <li inner_html=html></li>
                }).collect_view()}
            </ul>
            <div class="text-sm flex items-center gap-2">
                <span class="text-fg-muted">"→"</span>
                <span class="text-accent font-medium" inner_html=result_html></span>
            </div>
            {note_html.map(|html| view! {
                <div class="text-xs text-fg-muted mt-2 italic" inner_html=html></div>
            })}
            <div class="absolute top-2 right-2 hidden group-hover:flex gap-0.5">
                <button
                    class="inline-flex items-center justify-center w-6 h-6 rounded bg-bg-elevated border border-border text-fg-muted hover:text-fg"
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        on_edit(recipe_for_edit.clone());
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
