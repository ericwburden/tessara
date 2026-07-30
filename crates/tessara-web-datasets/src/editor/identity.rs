//! Dataset editor identity fields.

#[cfg(feature = "hydrate")]
use crate::api;
use icons::{Plus, X};
use leptos::prelude::*;
use tessara_module_ui::{Combobox, ComboboxOption};

#[component]
pub(crate) fn DatasetIdentitySection(
    dataset_id: Option<String>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    tags: RwSignal<Vec<String>>,
    known_tags: RwSignal<Vec<String>>,
    tag_input: RwSignal<String>,
    available_tags: Signal<Vec<String>>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <h3>"Dataset Definition"</h3>
            <div class="form-grid">
                <label class="form-field">
                    <span>"Name"</span>
                    <input
                        required
                        prop:value=move || name.get()
                        on:change=move |event| {
                            commit_name(name, slug, event_target_value(&event));
                        }
                        on:blur=move |event| {
                            commit_name(name, slug, event_target_value(&event));
                        }
                    />
                </label>
                <label class="form-field">
                    <span>"Slug"</span>
                    <input
                        required
                        prop:value=move || slug.get()
                        on:change=move |event| slug.set(event_target_value(&event))
                        on:blur=move |event| slug.set(event_target_value(&event))
                    />
                </label>
                <DatasetTagsControl
                    dataset_id=dataset_id
                    tags=tags
                    known_tags=known_tags
                    tag_input=tag_input
                    available_tags=available_tags
                    save_error=save_error
                    save_message=save_message
                />
            </div>
        </section>
    }
}

#[component]
fn DatasetTagsControl(
    dataset_id: Option<String>,
    tags: RwSignal<Vec<String>>,
    known_tags: RwSignal<Vec<String>>,
    tag_input: RwSignal<String>,
    available_tags: Signal<Vec<String>>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) -> impl IntoView {
    let combobox_options = Signal::derive(move || {
        let selected = tags.get();
        available_tags
            .get()
            .into_iter()
            .filter(|tag| !contains_tag(&selected, tag))
            .map(|tag| ComboboxOption {
                value: tag.clone(),
                label: tag,
            })
            .collect::<Vec<_>>()
    });
    let selected_label = Signal::derive(move || {
        let count = tags.get().len();
        if count == 0 {
            "Choose existing tag".into()
        } else {
            format!("Add tag ({count} selected)")
        }
    });
    let persist_tag_change = Callback::new(move |()| {
        persist_dataset_tags(dataset_id.clone(), tags, save_error, save_message)
    });
    let add_selected_tag = Callback::new(move |tag: String| {
        add_known_tag(known_tags, &tag);
        add_tag(tags, &tag);
        persist_tag_change.run(());
    });

    view! {
        <fieldset class="form-field form-field--wide dataset-tags-editor">
            <legend>"Tags"</legend>
            <div class="dataset-tags-editor__controls">
                <div class="dataset-tags-editor__combobox">
                    <Combobox
                        options=combobox_options
                        selected_label=selected_label
                        on_select=add_selected_tag
                        placeholder="Choose existing tag"
                        search_placeholder="Search tags..."
                        empty_label="No matching tags."
                        aria_label="Choose dataset tag"
                    />
                    <div class="dataset-tags-editor__chips dataset-tags-editor__chips--inside" aria-label="Selected dataset tags">
                        {move || {
                            let selected_tags = tags.get();
                            if selected_tags.is_empty() {
                                view! { <span class="dataset-tags-editor__placeholder">"No tags selected"</span> }.into_any()
                            } else {
                                view! {
                                    <>
                                        {selected_tags.into_iter().map(|tag| {
                                            let tag_for_remove = tag.clone();
                                            let tag_label = tag.clone();
                                            view! {
                                                <button
                                                    class="dataset-tags-editor__chip"
                                                    type="button"
                                                    aria-label=format!("Remove tag {tag}")
                                                    on:click=move |_| {
                                                        remove_tag(tags, &tag_for_remove);
                                                        persist_tag_change.run(());
                                                    }
                                                >
                                                    <span>{tag_label}</span>
                                                    <X/>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
                <label class="dataset-tags-editor__custom">
                    <span class="sr-only">"Custom tag"</span>
                    <input
                        type="text"
                        placeholder="Add custom tag"
                        prop:value=move || tag_input.get()
                        on:input=move |event| tag_input.set(event_target_value(&event))
                    />
                    <button
                        class="icon-button dataset-tags-editor__add-button"
                        type="button"
                        title="Add custom tag"
                        aria-label="Add custom tag"
                        disabled=move || normalize_tag(&tag_input.get()).is_empty()
                        on:click=move |_| {
                            let value = tag_input.get_untracked();
                            add_known_tag(known_tags, &value);
                            add_tag(tags, &value);
                            tag_input.set(String::new());
                            persist_tag_change.run(());
                        }
                    >
                        <Plus class="icon-button__icon"/>
                    </button>
                </label>
            </div>
        </fieldset>
    }
}

fn commit_name(name: RwSignal<String>, slug: RwSignal<String>, value: String) {
    let derived_slug = snake_case_slug(&value);
    name.set(value);

    if slug.get_untracked().trim().is_empty() && !derived_slug.is_empty() {
        slug.set(derived_slug);
    }
}

fn snake_case_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = true;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_end_matches('_').to_string()
}

fn add_tag(tags: RwSignal<Vec<String>>, value: &str) {
    let normalized = normalize_tag(value);
    if normalized.is_empty() {
        return;
    }
    tags.update(|tags| {
        if !contains_tag(tags, &normalized) {
            tags.push(normalized);
            tags.sort_by_key(|tag| tag.to_ascii_lowercase());
        }
    });
}

fn add_known_tag(known_tags: RwSignal<Vec<String>>, value: &str) {
    let normalized = normalize_tag(value);
    if normalized.is_empty() {
        return;
    }
    known_tags.update(|tags| {
        if !contains_tag(tags, &normalized) {
            tags.push(normalized);
            tags.sort_by_key(|tag| tag.to_ascii_lowercase());
        }
    });
}

fn remove_tag(tags: RwSignal<Vec<String>>, value: &str) {
    tags.update(|tags| tags.retain(|tag| !tag.eq_ignore_ascii_case(value)));
}

fn contains_tag(tags: &[String], value: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(value))
}

fn normalize_tag(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(feature = "hydrate")]
fn persist_dataset_tags(
    dataset_id: Option<String>,
    tags: RwSignal<Vec<String>>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) {
    let Some(dataset_id) = dataset_id else {
        return;
    };
    let tags = tags.get_untracked();
    leptos::task::spawn_local(async move {
        save_error.set(None);
        match api::update_dataset_tags(&dataset_id, tags).await {
            Ok(_) => save_message.set(Some("Catalog tags saved.".into())),
            Err(message) => save_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn persist_dataset_tags(
    _: Option<String>,
    _: RwSignal<Vec<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(test)]
mod tests {
    use super::{add_known_tag, contains_tag, normalize_tag, snake_case_slug};
    use leptos::prelude::{GetUntracked, RwSignal};

    #[test]
    fn snake_case_slug_normalizes_dataset_names() {
        assert_eq!(snake_case_slug("UAT Dataset"), "uat_dataset");
        assert_eq!(
            snake_case_slug(" Demo Partner: Snapshot 2026 "),
            "demo_partner_snapshot_2026"
        );
        assert_eq!(snake_case_slug("Already_snake"), "already_snake");
    }

    #[test]
    fn tag_helpers_trim_and_match_case_insensitively() {
        assert_eq!(normalize_tag(" demo "), "demo");
        assert!(contains_tag(&["Demo".into()], "demo"));
    }

    #[test]
    fn known_tags_keep_custom_values_available_once_removed() {
        let known_tags = RwSignal::new(Vec::<String>::new());

        add_known_tag(known_tags, " bears ");
        add_known_tag(known_tags, "Bears");

        assert_eq!(known_tags.get_untracked(), vec!["bears".to_string()]);
    }
}
