use leptos::prelude::*;
use tessara_module_ui::{Button, ButtonSize, ButtonVariant, EmptyState, PageHeader};

use crate::{ComponentDefinitionBootstrap, ComponentRouteBootstrap};

pub(crate) fn component_bootstrap_view(bootstrap: &ComponentRouteBootstrap) -> AnyView {
    match bootstrap {
        ComponentRouteBootstrap::Directory {
            components,
            can_manage,
        } => {
            let components = components.clone();
            let can_manage = *can_manage;
            view! {
                <section class="route-panel components-page component-directory">
                    <PageHeader title="Components" description="Find, inspect, and reuse published Component definitions.">
                        {can_manage.then(|| view! { <Button href="/components/new">"Create Component"</Button> })}
                    </PageHeader>
                    <label class="table-search component-directory__search">
                        <span>"Search components by name"</span>
                        <input type="search" data-component-search placeholder="Search Components"/>
                    </label>
                    {if components.is_empty() {
                        view! { <EmptyState title="No visible components" message="No components are visible for the current account."/> }.into_any()
                    } else {
                        view! {
                            <div class="table-wrap component-directory__table">
                                <table class="data-table component-table">
                                    <thead><tr><th>"Component"</th><th>"Kind"</th><th>"Status"</th><th>"Actions"</th></tr></thead>
                                    <tbody>{components.into_iter().map(|item| {
                                        let view_href = format!("/components/{}/view", item.slug);
                                        let versions_href = format!("/components/{}/versions", item.slug);
                                        let edit_href = format!("/components/{}/edit", item.slug);
                                        view! { <tr data-component-row>
                                            <td><strong>{item.name}</strong><small class="data-table__secondary-text">{item.slug}</small></td>
                                            <td>{item.component_type}</td>
                                            <td><span class="status-badge">{item.publication_state}</span></td>
                                            <td class="data-table__actions"><div class="action-row">
                                                <Button href=view_href size=ButtonSize::Compact>"View"</Button>
                                                {item.manageable.then(|| view! { <Button href=versions_href size=ButtonSize::Compact variant=ButtonVariant::Secondary>"Versions"</Button> })}
                                                {item.manageable.then(|| view! { <Button href=edit_href size=ButtonSize::Compact variant=ButtonVariant::Secondary>"Edit"</Button> })}
                                            </div></td>
                                        </tr> }
                                    }).collect_view()}</tbody>
                                </table>
                            </div>
                        }.into_any()
                    }}
                </section>
            }.into_any()
        }
        ComponentRouteBootstrap::Create { dataset_error, .. } => {
            editor_view(None, dataset_error.clone())
        }
        ComponentRouteBootstrap::Edit {
            component,
            dataset_error,
            ..
        } => editor_view(Some(component.clone()), dataset_error.clone()),
        ComponentRouteBootstrap::Detail {
            component,
            manageable,
        } => detail_view(component.clone(), *manageable, false),
        ComponentRouteBootstrap::View {
            component,
            manageable,
        } => detail_view(component.clone(), *manageable, true),
        ComponentRouteBootstrap::Versions {
            component,
            manageable,
        } => versions_view(component.clone(), *manageable),
    }
}

fn editor_view(
    component: Option<ComponentDefinitionBootstrap>,
    dataset_error: Option<String>,
) -> AnyView {
    let editing = component.is_some();
    let name = component
        .as_ref()
        .map(|value| value.name.clone())
        .unwrap_or_default();
    let slug = component
        .as_ref()
        .map(|value| value.slug.clone())
        .unwrap_or_default();
    let description = component
        .as_ref()
        .and_then(|value| value.description.clone())
        .unwrap_or_default();
    view! {
        <section class="route-panel components-page component-editor">
            <PageHeader title=if editing { "Edit Component" } else { "Create Component" } description="Define reusable presentation backed by an authorized Dataset version."/>
            {dataset_error.map(|message| view! { <div class="empty-state" role="alert"><h2>"Dataset metadata unavailable"</h2><p>{message}</p></div> })}
            <form class="component-form" data-component-editor>
                <label class="field-label"><span>"Name"</span><input class="text-input" name="name" value=name/></label>
                <label class="field-label"><span>"Slug"</span><input class="text-input" name="slug" value=slug/></label>
                <label class="field-label"><span>"Description"</span><textarea class="text-input" name="description">{description}</textarea></label>
                <label class="field-label"><span>"Dataset version"</span><button class="button button--secondary" type="button" data-dataset-picker>"Choose Dataset version"</button></label>
                <label class="field-label"><span>"Component kind"</span><select class="text-input" name="component_type"><option value="table">"Table"</option><option value="stat_card">"Stat Card"</option><option value="bar">"Bar"</option><option value="line">"Line"</option><option value="pie">"Pie"</option><option value="donut">"Donut"</option></select></label>
                <section class="component-panel component-editor__configuration"><h2>"Presentation configuration"</h2><p>"Configure fields, filters, sorting, labels, and visual behavior for this Component kind."</p></section>
                <div class="action-row"><button class="button button--secondary" type="button">"Preview"</button><button class="button" type="submit">"Save draft"</button></div>
            </form>
        </section>
    }.into_any()
}

fn detail_view(component: ComponentDefinitionBootstrap, manageable: bool, viewer: bool) -> AnyView {
    let edit_href = format!("/components/{}/edit", component.slug);
    let versions_href = format!("/components/{}/versions", component.slug);
    view! {
        <section class="route-panel components-page component-detail">
            <PageHeader title=component.name.clone() description=component.description.clone().unwrap_or_default()>
                {manageable.then(|| view! { <Button href=edit_href variant=ButtonVariant::Secondary>"Edit"</Button> })}
                {manageable.then(|| view! { <Button href=versions_href variant=ButtonVariant::Secondary>"Versions"</Button> })}
            </PageHeader>
            <dl class="info-list component-meta"><div><dt>"Slug"</dt><dd>{component.slug.clone()}</dd></div><div><dt>"Versions"</dt><dd>{component.versions.len()}</dd></div></dl>
            {if viewer { view! { <section class="component-panel component-viewer" data-component-viewer><h2>"Component preview"</h2><p>"Loading the published Component presentation…"</p></section> }.into_any() } else { ().into_any() }}
        </section>
    }.into_any()
}

fn versions_view(component: ComponentDefinitionBootstrap, manageable: bool) -> AnyView {
    let edit_href = format!("/components/{}/edit", component.slug);
    view! {
        <section class="route-panel components-page component-versions">
            <PageHeader title=component.name.clone() description="Versions">
                {manageable.then(|| view! { <Button href=edit_href variant=ButtonVariant::Secondary>"Edit"</Button> })}
            </PageHeader>
            <div class="table-wrap"><table class="data-table component-table"><thead><tr><th>"Version"</th><th>"Publication"</th><th>"Lifecycle"</th><th>"Kind"</th><th>"Note"</th></tr></thead><tbody>
                {component.versions.into_iter().map(|version| view! { <tr><td>{version.version_label}</td><td>{version.publication_state}</td><td><span class="status-badge">{version.lifecycle_state}</span></td><td>{version.component_type}</td><td>{version.version_note}</td></tr> }).collect_view()}
            </tbody></table></div>
        </section>
    }.into_any()
}
