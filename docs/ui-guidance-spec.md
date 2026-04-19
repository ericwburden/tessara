# Tessara UI Guidance Allium Specification

This document formalizes the canonical UI behavior in [ui-guidance.md](./ui-guidance.md) as an Allium specification.

It is a behavioral companion, not a replacement. The prose guidance remains the human-first design authority. This spec captures the observable UI contract that should remain stable across implementation changes.

Scope:

- shared authenticated shell
- bare sign-in behavior
- permission-gated navigation
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

```allium
-- allium: 3
-- ui-guidance-spec.allium
-- Scope: Canonical UI shell and screen-family behaviour derived from docs/ui-guidance.md.
-- Includes: shared authenticated shell, sign-in, permission-gated navigation, responsive shell states,
-- home, organization explorer, form builder posture, and feedback/state behaviour.
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
enum SurfaceKind {
    sign_in
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
}

------------------------------------------------------------
-- Entities and Variants
------------------------------------------------------------

entity UserSession {
    signed_in: Boolean
    permissions: Set<String>
    delegated_user_label: String?
    scope_root_labels: Set<String>
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
    reports_visible_in_default_sidebar: Boolean
    shell_horizontal_scroll_required: Boolean

    is_authenticated: session.signed_in

    invariant AuthenticatedShellNeverShowsSignIn {
        is_authenticated implies active_surface != sign_in
    }

    invariant SignInStateStaysOutsideAuthenticatedShell {
        active_surface = sign_in implies not is_authenticated
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

    invariant AdminGroupMatchesPermission {
        admin_group_visible = session.permissions.any(p => p = "admin:all")
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
    queue_primary: Boolean
    hierarchy_secondary: Boolean
    selected_node_related_work_visible: Boolean
    metrics_presentation: MetricPresentation
    destination_launcher_cards_present: Boolean

    invariant HomePrioritisesQueueAndHierarchy {
        queue_primary
        and hierarchy_secondary
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

surface SharedApplicationShell {
    facing user: AuthenticatedUser

    context shell: ShellExperience
        where shell.session = user and shell.active_surface != sign_in

    exposes:
        shell.active_surface
        shell.theme_mode
        shell.sidebar_state
        shell.admin_group_visible
        user.delegated_user_label
        user.scope_root_labels

    provides:
        OpenDestination(user, shell, destination)
        SearchGlobally(user, shell, query)
        ChooseThemeMode(user, shell, mode)
        ToggleSidebar(user, shell)

    @guarantee SharedShellNavigationIsPermissionGated
        -- The shell keeps one shared navigation model and hides destinations
        -- that the current user's permission set does not allow.

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
        -- Home is a shared operational workspace rather than a launcher-card index.
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
open question "Should future UI Allium work split shell behaviour and builder behaviour into separate modules once the migration stabilises?"
```
