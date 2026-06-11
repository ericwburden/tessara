//! Owns the features::administration::pages module behavior.

use crate::ui::{AppShell, PageHeader};

use leptos::prelude::*;

#[component]
/// Renders the administration page view.
pub fn AdministrationPage() -> impl IntoView {
    view! {
        <AppShell active_route="administration" title="Administration">
            <section class="route-panel administration-page">
                <PageHeader
                    title="Administration"
                    description="Internal configuration routes are registered natively while their detailed management screens are restored."
                />

                <div class="organization-detail-content administration-landing">
                    <div class="organization-detail-content__grid">
                        <section class="organization-detail-card">
                            <h3>"User Management"</h3>
                            <p>"Manage local users, passwords, role memberships, and active status."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/users">"Open Users"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Roles And Access"</h3>
                            <p>"Review reusable capability bundles and the access assignments that control application visibility."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/roles">"Open Roles"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Organization Schema"</h3>
                            <p>"Manage node type labels and hierarchy rules for the organization model."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/node-types">"Open Node Types"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Datasets"</h3>
                            <p>"Review imported dataset catalogs and supporting reference data."</p>
                            <div class="form-actions">
                                <a class="button" href="/datasets">"Open Datasets"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Components"</h3>
                            <p>"Review dataset-backed components and dashboard composition assets."</p>
                            <div class="form-actions">
                                <a class="button button--secondary" href="/components">"Open Components"</a>
                            </div>
                        </section>
                    </div>
                </div>
            </section>
        </AppShell>
    }
}
