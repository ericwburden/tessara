# Tessara UI Guidance Allium Specification

This document formalizes the canonical UI behavior in [ui-guidance.md](./ui-guidance.md) as an Allium specification.

It is a behavioral companion, not a replacement. The prose guidance remains the human-first design authority. This spec captures the observable UI contract that should remain stable across implementation changes.

Scope:

- shared authenticated shell
- bare sign-in behavior
- one-time administrator enrollment behavior
- dynamically composed, permission-gated Core and module navigation
- semantic destination and module-state behavior
- responsive shell state
- operational home behavior
- organization explorer behavior
- form-builder authoring posture
- feedback and state-separation behavior

Excludes:

- exact CSS implementation
- asset-file details
- icon-library internals
- route-by-route legacy compatibility behavior except where the guidance already treats it as a constraint
- sprint-specific fixed destination keys, default slots, and capability names; the active sprint plan and human-first `ui-guidance.md` remain authoritative for those concrete composition rules

```allium
-- allium: 3
-- ui-guidance-spec.allium
-- Scope: Canonical UI shell and screen-family behaviour derived from docs/ui-guidance.md.
-- Includes: shared authenticated shell, sign-in, administrator enrollment, dynamically composed Core/module navigation,
-- semantic destinations, module state, responsive shell states, home, organization explorer,
-- form builder posture, and feedback/state behaviour.
-- Excludes: exact CSS, asset implementation details, icon-library internals, and legacy route ownership details.

------------------------------------------------------------
-- Enumerations
------------------------------------------------------------

enum ThemeMode { system | light | dark }
enum ViewportClass { mobile | tablet | desktop }
enum SidebarState { expanded | collapsed | overlay }
enum NotificationStyle { bell_icon | labeled_control }
enum MetricPresentation { compact_text | summary_cards }
enum ExplorerPattern { explorer_detail | tree_sheet | flat_cards }
enum ToastPlacement { top_right | elsewhere }
enum MajorSuccessPattern { banner | toast_only }
enum DestinationOwnerKind { core_installation | module_instance }
enum NavigationDisplayState { hidden | available | disabled | unconfigured | unavailable | incompatible }
enum DestinationResolutionOutcome { resolved | disabled | unconfigured | unavailable | incompatible | unauthorized | unknown }
enum AdministratorEnrollmentClaimKind { initial | recovery }
enum AdministratorEnrollmentClaimState { issued | reserved | consumed | expired | revoked }
enum SurfaceKind {
    sign_in
    administrator_enrollment
    home
    organization
    forms
    workflows
    responses
    components
    dashboards
    datasets
    administration
    migration
    module_product
    module_administration
    module_management
}

------------------------------------------------------------
-- Entities and Variants
------------------------------------------------------------

entity UserSession {
    signed_in: Boolean
    authorization_grant_summaries: Set<String>
    delegated_user_label: String?
    scope_root_labels: Set<String>
    scope_labels_used_as_authority: Boolean

    invariant DisplayScopeLabelsAreNotAuthorization {
        not scope_labels_used_as_authority
    }
}

entity NavigationDestination {
    owner_kind: DestinationOwnerKind
    owner_id: String
    route_name: String
    required_capabilities_any_of: Set<String>
    actor_has_any_required_display_capability: Boolean
    core_admin_all_implied_for_display: Boolean
    display_capability_eligible: Boolean
    administrative: Boolean
    administrator_displayed: Boolean
    module_installed: Boolean
    module_enabled: Boolean
    requires_enabled_module: Boolean
    user_authorized: Boolean
    display_state: NavigationDisplayState
    deployment_url_persisted: Boolean
    display_choice_changes_authorization: Boolean

    invariant DestinationIsSemanticRatherThanDeploymentSpecific {
        not deployment_url_persisted
    }

    invariant DisplayPolicyDoesNotChangeAuthorization {
        not display_choice_changes_authorization
    }

    invariant DisplayCapabilityEligibilityUsesAnyOf {
        display_capability_eligible = (
            required_capabilities_any_of.count = 0
            or actor_has_any_required_display_capability
            or core_admin_all_implied_for_display
        )
    }

    invariant HiddenDestinationsAreNotEligibleForDisplay {
        display_state = hidden implies (
            not administrator_displayed
            or not display_capability_eligible
            or not user_authorized
            or (
                owner_kind = module_instance
                and (
                    not module_installed
                    or (requires_enabled_module and not module_enabled)
                )
            )
        )
    }

    invariant VisibleDestinationStatesAreEligible {
        display_state != hidden implies (
            administrator_displayed
            and display_capability_eligible
            and user_authorized
            and (
                owner_kind = core_installation
                or (
                    module_installed
                    and (not requires_enabled_module or module_enabled)
                )
            )
        )
    }

    invariant AdministrativeDestinationsRemainRecoverable {
        owner_kind = module_instance and administrative implies not requires_enabled_module
    }

    invariant ProductDestinationsRequireEnabledModule {
        owner_kind = module_instance and not administrative implies requires_enabled_module
    }
}

entity DestinationResolution {
    requested_owner_kind: DestinationOwnerKind
    requested_owner_id: String
    requested_route_name: String
    user_authorized: Boolean
    destination_existence_disclosure_authorized: Boolean
    outcome: DestinationResolutionOutcome

    invariant UnauthorizedResolutionIsExplicit {
        outcome = unauthorized implies not user_authorized
    }

    invariant UnauthorizedUsersDoNotLearnDestinationExistence {
        not user_authorized implies outcome = unauthorized
    }

    invariant UnknownDestinationRequiresAuthorization {
        outcome = unknown implies (
            user_authorized
            and destination_existence_disclosure_authorized
        )
    }

    invariant NoDisclosureAuthorizationUsesRestrictedOutcome {
        not destination_existence_disclosure_authorized implies outcome = unauthorized
    }

    invariant DestinationActionAuthorizationIncludesExistenceDisclosure {
        user_authorized implies destination_existence_disclosure_authorized
    }

    invariant ResolvedDestinationIsAuthorized {
        outcome = resolved implies user_authorized
    }
}

entity ShellExperience {
    session: UserSession
    active_surface: SurfaceKind
    viewport: ViewportClass
    theme_mode: ThemeMode
    sidebar_state: SidebarState
    top_bar_height_px: Integer
    sidebar_expanded_width_px: Integer
    sidebar_collapsed_width_px: Integer
    top_bar_search_visible: Boolean
    top_bar_notifications_visible: Boolean
    top_bar_help_visible: Boolean
    top_bar_mobile_nav_visible: Boolean
    top_bar_account_visible: Boolean
    top_bar_session_visible: Boolean
    notifications_style: NotificationStyle
    sidebar_footer_account_visible: Boolean
    sidebar_footer_delegation_visible: Boolean
    sidebar_footer_scope_visible: Boolean
    sidebar_footer_theme_selector_visible: Boolean
    admin_group_visible: Boolean
    navigation_destinations: Set<NavigationDestination>
    destination_resolutions: Set<DestinationResolution>
    navigation_composed_from_core_and_modules: Boolean
    administration_destinations_permission_gated: Boolean
    navigation_visibility_separate_from_authorization: Boolean
    module_state_dimensions_distinct: Boolean
    semantic_destination_resolution_used: Boolean
    destination_resolution_outcomes_distinct: Boolean
    shell_context_versioned: Boolean
    module_documents_server_render_shell: Boolean
    core_fallback_preserves_context: Boolean
    remote_fragment_wrapping_used: Boolean
    global_search_provider_contracts_used: Boolean
    global_search_results_owner_qualified: Boolean
    global_search_provider_failures_isolated: Boolean
    reports_visible_in_default_sidebar: Boolean
    shell_horizontal_scroll_required: Boolean

    is_authenticated: session.signed_in

    invariant AuthenticatedShellNeverShowsSignIn {
        is_authenticated implies (
            active_surface != sign_in
            and active_surface != administrator_enrollment
        )
    }

    invariant EnrollmentAndSignInStayOutsideAuthenticatedShell {
        (
            active_surface = sign_in
            or active_surface = administrator_enrollment
        ) implies not is_authenticated
    }

    invariant TopBarOnlyOwnsQuietUtilities {
        top_bar_search_visible
        and top_bar_notifications_visible
        and top_bar_help_visible
        and not top_bar_account_visible
        and not top_bar_session_visible
    }

    invariant NotificationsUseBellStyle {
        notifications_style = bell_icon
    }

    invariant FooterOwnsAccountThemeAndContext {
        sidebar_footer_account_visible
        and sidebar_footer_theme_selector_visible
        and sidebar_footer_delegation_visible = (session.delegated_user_label != null)
        and sidebar_footer_scope_visible = (session.scope_root_labels.count > 0)
    }

    invariant NavigationCompositionUsesCoreAndModuleContributions {
        navigation_composed_from_core_and_modules
        and administration_destinations_permission_gated
        and navigation_visibility_separate_from_authorization
        and module_state_dimensions_distinct
        and semantic_destination_resolution_used
        and destination_resolution_outcomes_distinct
    }

    invariant AdministrationGroupReflectsVisibleContributions {
        admin_group_visible = navigation_destinations.any(destination =>
            destination.administrative and destination.display_state != hidden
        )
    }

    invariant ModuleDocumentsUseShellContext {
        shell_context_versioned
        and module_documents_server_render_shell
        and core_fallback_preserves_context
        and not remote_fragment_wrapping_used
    }

    invariant GlobalSearchUsesBoundedProviders {
        global_search_provider_contracts_used
        and global_search_results_owner_qualified
        and global_search_provider_failures_isolated
    }

    invariant ReportsStayOutOfDefaultSidebar {
        not reports_visible_in_default_sidebar
    }

    invariant ResponsiveSidebarBehaviour {
        (viewport = desktop implies sidebar_state = expanded)
        and (viewport = tablet implies sidebar_state = collapsed)
        and (viewport = mobile implies sidebar_state = overlay)
    }

    invariant CanonicalShellDimensions {
        top_bar_height_px = app_bar_height_px
        and sidebar_expanded_width_px = desktop_sidebar_expanded_width_px
        and sidebar_collapsed_width_px = tablet_sidebar_collapsed_width_px
    }

    invariant NoShellLevelHorizontalScroll {
        not shell_horizontal_scroll_required
    }
}

entity HomeSurface {
    shell: ShellExperience
    work_discovery_contribution_available: Boolean
    related_work_contribution_available: Boolean
    installation_context_primary_when_no_work: Boolean
    queue_primary: Boolean
    hierarchy_secondary: Boolean
    selected_node_related_work_visible: Boolean
    metrics_presentation: MetricPresentation
    destination_launcher_cards_present: Boolean

    invariant HomeUsesAvailableContributions {
        queue_primary = work_discovery_contribution_available
        and hierarchy_secondary
        and selected_node_related_work_visible = related_work_contribution_available
        and (not work_discovery_contribution_available implies installation_context_primary_when_no_work)
        and metrics_presentation = compact_text
        and not destination_launcher_cards_present
    }
}

entity OrganizationSurface {
    shell: ShellExperience
    scope_title_visible: Boolean
    generic_organization_title: Boolean
    desktop_pattern: ExplorerPattern
    tablet_pattern: ExplorerPattern
    mobile_pattern: ExplorerPattern
    node_cards_used_in_explorer: Boolean
    selected_node_related_work_primary: Boolean

    invariant OrganizationUsesScopeAwareExplorer {
        scope_title_visible
        and not generic_organization_title
        and desktop_pattern = explorer_detail
        and tablet_pattern = explorer_detail
        and mobile_pattern = tree_sheet
        and not node_cards_used_in_explorer
        and selected_node_related_work_primary
    }
}

entity BuilderSurface {
    shell: ShellExperience
    section_panels_stacked: Boolean
    section_settings_visible_in_canvas: Boolean
    insert_affordance_adjacent_to_canvas: Boolean
    properties_panel_selection_driven: Boolean
    page_level_lifecycle_actions_separate: Boolean

    invariant BuilderKeepsCanvasPrimary {
        section_panels_stacked
        and section_settings_visible_in_canvas
        and insert_affordance_adjacent_to_canvas
        and properties_panel_selection_driven
        and page_level_lifecycle_actions_separate
    }
}

entity SignInSurface {
    session: UserSession
    shell_visible: Boolean
    non_auth_content_visible: Boolean
    sign_in_action_visible: Boolean

    invariant SignInStaysBare {
        not session.signed_in
        and sign_in_action_visible
        and not shell_visible
        and not non_auth_content_visible
    }
}

entity AdministratorEnrollmentSurface {
    session: UserSession
    claim_kind: AdministratorEnrollmentClaimKind
    claim_state: AdministratorEnrollmentClaimState
    reserved_attempt_resumable: Boolean
    designated_role_covers_capability_floor: Boolean
    viable_core_administrator_exists: Boolean
    enrollment_action_visible: Boolean
    normal_sign_in_fields_visible: Boolean
    claim_secret_redisplayed: Boolean
    claim_failure_reason_disclosed: Boolean

    invariant EnrollmentIsOneTimeBareAndSeparate {
        not session.signed_in
        and enrollment_action_visible = (
            designated_role_covers_capability_floor
            and not viable_core_administrator_exists
            and (
                claim_state = issued
                or (claim_state = reserved and reserved_attempt_resumable)
            )
        )
        and not normal_sign_in_fields_visible
        and not claim_secret_redisplayed
        and not claim_failure_reason_disclosed
    }
}

entity FeedbackPresentation {
    shell: ShellExperience
    distinguishes_empty_loading_no_results_error: Boolean
    distinguishes_read_only_restricted_unavailable: Boolean
    toast_placement: ToastPlacement
    major_success_pattern: MajorSuccessPattern

    invariant FeedbackStatesStayDistinct {
        distinguishes_empty_loading_no_results_error
        and distinguishes_read_only_restricted_unavailable
        and toast_placement = top_right
        and major_success_pattern = banner
    }
}

------------------------------------------------------------
-- Config
------------------------------------------------------------

config {
    app_bar_height_px: Integer = 56
    desktop_sidebar_expanded_width_px: Integer = 256
    tablet_sidebar_collapsed_width_px: Integer = 72
}

------------------------------------------------------------
-- Rules
------------------------------------------------------------

rule SuccessfulSignInReturnsToHome {
    when: SignIn(visitor, sign_in)
    requires: sign_in.session = visitor
    requires: not visitor.signed_in
    ensures: visitor.signed_in = true
    for shell in ShellExperiences where shell.session = visitor:
        ensures: shell.active_surface = home
}

rule SessionEndReturnsToSignIn {
    when: session: UserSession.signed_in transitions_to false
    for shell in ShellExperiences where shell.session = session:
        ensures: shell.active_surface = sign_in
}

rule UserChoosesThemeMode {
    when: ChooseThemeMode(user, shell, mode)
    requires: shell.session = user
    requires: user.signed_in
    ensures: shell.theme_mode = mode
}

rule DesktopViewportUsesExpandedSidebar {
    when: shell: ShellExperience.viewport becomes desktop
    ensures: shell.sidebar_state = expanded
}

rule TabletViewportUsesCollapsedSidebar {
    when: shell: ShellExperience.viewport becomes tablet
    ensures: shell.sidebar_state = collapsed
}

rule MobileViewportUsesOverlaySidebar {
    when: shell: ShellExperience.viewport becomes mobile
    ensures: shell.sidebar_state = overlay
}

------------------------------------------------------------
-- Actor Declarations
------------------------------------------------------------

actor AuthenticatedUser {
    identified_by: UserSession where signed_in = true
}

actor AnonymousVisitor {
    identified_by: UserSession where signed_in = false
}

------------------------------------------------------------
-- Surfaces
------------------------------------------------------------

surface SignInExperience {
    facing visitor: AnonymousVisitor

    context sign_in: SignInSurface where sign_in.session = visitor

    exposes:
        sign_in.sign_in_action_visible

    provides:
        SignIn(visitor, sign_in)

    @guarantee SignInRemainsBare
        -- Sign-in does not preview the post-auth shell
        -- or unrelated product content.
}

surface AdministratorEnrollmentExperience {
    facing visitor: AnonymousVisitor

    context enrollment: AdministratorEnrollmentSurface where enrollment.session = visitor

    exposes:
        enrollment.enrollment_action_visible
        enrollment.claim_kind

    @guarantee EnrollmentRemainsBareAndDistinct
        -- Initial or recovery administrator enrollment is a one-time surface,
        -- not normal sign-in and never a place that redisplays the claim secret
        -- or distinguishes invalid claim lifecycle reasons.
}

surface SharedApplicationShell {
    facing user: AuthenticatedUser

    context shell: ShellExperience
        where shell.session = user
            and shell.active_surface != sign_in
            and shell.active_surface != administrator_enrollment

    exposes:
        shell.active_surface
        shell.theme_mode
        shell.sidebar_state
        shell.admin_group_visible
        shell.shell_context_versioned
        shell.module_documents_server_render_shell
        shell.core_fallback_preserves_context
        shell.global_search_provider_contracts_used
        shell.global_search_results_owner_qualified
        shell.global_search_provider_failures_isolated
        user.delegated_user_label
        user.scope_root_labels
        for destination in shell.navigation_destinations:
            destination.owner_id
            destination.route_name
            destination.required_capabilities_any_of
            destination.display_capability_eligible
            destination.administrative
            destination.display_state
        for resolution in shell.destination_resolutions:
            resolution.requested_owner_kind
            resolution.requested_owner_id
            resolution.requested_route_name
            resolution.outcome

    provides:
        OpenDestination(user, shell, destination)
        SearchGlobally(user, shell, query)
        ChooseThemeMode(user, shell, mode)
        ToggleSidebar(user, shell)

    @guarantee SharedShellNavigationIsPermissionGated
        -- The shell composes permanent Core destinations with installed modules'
        -- advertised contributions and filters them by administrator display policy,
        -- module enablement and the current user's applicable scope-bound grants.

    @guarantee NavigationVisibilityIsNotAuthorization
        -- Hiding or ordering a destination never grants, revokes or substitutes
        -- for authorization enforced by the owning Core or module application.

    @guarantee ModuleDestinationStatesRemainDistinct
        -- Eligible destinations keep disabled, unconfigured, unavailable and
        -- incompatible outcomes distinct instead of collapsing them into empty state.

    @guarantee DirectDestinationResolutionRemainsExplicit
        -- Unauthorized direct requests use one restricted outcome whether the destination
        -- exists; only disclosure-authorized resolution may return unknown separately from
        -- disabled, unconfigured, unavailable and incompatible providers.

    @guarantee ModulesRenderAgainstVersionedShellContext
        -- Route-owning modules server-render complete documents against the authenticated
        -- versioned Shell Context; the gateway fallback preserves context without remote fragments.

    @guarantee GlobalSearchUsesBoundedModuleProviders
        -- Global search invokes versioned provider contracts, returns owner-qualified
        -- semantic destinations, and isolates timeout or failure in one module provider.

    @guarantee TopBarRemainsGlobalOnly
        -- The top bar owns search, quiet utilities and mobile navigation only.
}

surface OperationalHome {
    facing user: AuthenticatedUser

    context home: HomeSurface
        where home.shell.session = user and home.shell.active_surface = home

    exposes:
        home.queue_primary
        home.hierarchy_secondary
        home.metrics_presentation
        home.selected_node_related_work_visible

    provides:
        OpenFullQueue(user, home)
        SelectHierarchyNode(user, home, node)

    @guarantee HomeStaysOperational
        -- Home is an installation-neutral workspace rather than a launcher-card index.
        -- Work queues and related work appear only through eligible module contributions.
}

surface OrganizationExplorer {
    facing user: AuthenticatedUser

    context explorer: OrganizationSurface
        where explorer.shell.session = user and explorer.shell.active_surface = organization

    exposes:
        explorer.desktop_pattern
        explorer.tablet_pattern
        explorer.mobile_pattern
        explorer.selected_node_related_work_primary

    provides:
        SelectNode(user, explorer, node)
        ExpandHierarchyBranch(user, explorer, branch)
        CollapseHierarchyBranch(user, explorer, branch)

    @guarantee OrganizationStaysScopeAware
        -- Organization browsing uses a scope-aware explorer pattern
        -- rather than a generic card-list directory.
}

surface FormBuilderAuthoring {
    facing user: AuthenticatedUser

    context builder: BuilderSurface
        where builder.shell.session = user and builder.shell.active_surface = forms

    exposes:
        builder.section_panels_stacked
        builder.section_settings_visible_in_canvas
        builder.properties_panel_selection_driven

    provides:
        AddSection(user, builder)
        SelectSection(user, builder, section)
        SelectField(user, builder, field)
        SaveDraft(user, builder)
        PublishDraft(user, builder)

    @guarantee BuilderKeepsTheCanvasPrimary
        -- The canvas remains the dominant authoring surface,
        -- with insertion near the canvas and deeper settings in selection-driven properties.
}

surface FeedbackMessages {
    facing user: AuthenticatedUser

    context feedback: FeedbackPresentation
        where feedback.shell.session = user and feedback.shell.active_surface != sign_in

    exposes:
        feedback.distinguishes_empty_loading_no_results_error
        feedback.distinguishes_read_only_restricted_unavailable
        feedback.toast_placement
        feedback.major_success_pattern

    provides:
        DismissToast(user, feedback)

    @guarantee FeedbackStatesRemainExplicit
        -- Empty, loading, no-results, error, read-only, restricted,
        -- and unavailable states stay distinct at the boundary.
}

------------------------------------------------------------
-- Open Questions
------------------------------------------------------------

open question "What global number-formatting pattern should apply beyond tabular numerals and local surface consistency?"
-- Decision: Core owns shell policy, Shell Context, navigation policy and semantic destination resolution.
-- Each separately deployed full-stack module uses the shared UI SDK and authenticated Shell Context
-- to server-render the complete coherent document for its own product and administration routes.
-- The gateway supplies a Core-owned fallback document when the route owner cannot render.
```
