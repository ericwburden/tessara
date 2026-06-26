//! Shared draggable panel list behavior.

use icons::GripVertical;
use leptos::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::JsCast;

static NEXT_DRAGGABLE_PANEL_LIST_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraggablePanelListItem {
    pub(crate) id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DraggablePanelListAnchor {
    Start,
    After(String),
}

#[derive(Clone)]
pub(crate) struct DraggablePanelListDraggable {
    pub(crate) id: String,
    pub(crate) index: usize,
}

#[derive(Clone)]
pub(crate) struct DraggablePanelListDropZone {
    pub(crate) anchor: DraggablePanelListAnchor,
}

#[derive(Clone)]
pub(crate) struct DraggablePanelListMove {
    pub(crate) dragged_id: String,
    pub(crate) anchor: DraggablePanelListAnchor,
}

#[component]
pub(crate) fn DraggablePanelList(
    items: Signal<Vec<DraggablePanelListItem>>,
    render_draggable: Callback<DraggablePanelListDraggable, AnyView>,
    render_drop_zone: Callback<DraggablePanelListDropZone, AnyView>,
    on_move: Callback<DraggablePanelListMove>,
    #[prop(default = "")] list_id: &'static str,
    #[prop(default = "draggable-panel-list")] container_class: &'static str,
    #[prop(default = "draggable-panel-list__items")] list_class: &'static str,
    #[prop(default = "draggable-panel-list__draggable")] draggable_class: &'static str,
    #[prop(default = "draggable-panel-list__drop-zone")] drop_zone_class: &'static str,
    #[prop(default = "Drag to reorder")] drag_handle_title: &'static str,
    #[prop(default = "application/x-tessara-draggable-panel")] data_transfer_type: &'static str,
) -> impl IntoView {
    let list_id = if list_id.trim().is_empty() {
        generated_draggable_panel_list_id()
    } else {
        list_id.to_string()
    };
    let armed_item_id = RwSignal::new(None::<String>);
    let dragging_item_id = RwSignal::new(None::<String>);
    let drop_target_anchor = RwSignal::new(None::<DraggablePanelListAnchor>);

    view! {
        <div class=container_class>
            {draggable_panel_insert(
                list_id.clone(),
                DraggablePanelListAnchor::Start,
                render_drop_zone,
                on_move,
                armed_item_id,
                dragging_item_id,
                drop_target_anchor,
                drop_zone_class,
                data_transfer_type,
            )}
            <div class=list_class>
                <For
                    each=move || {
                        items.get().into_iter().enumerate().collect::<Vec<_>>()
                    }
                    key=|(_, item)| item.id.clone()
                    children=move |(index, item)| {
                        let item_id = item.id;
                        let list_id_for_item = list_id.clone();
                        let list_id_for_insert = list_id.clone();
                        view! {
                            {draggable_panel_item(
                                list_id_for_item,
                                item_id.clone(),
                                index,
                                items,
                                render_draggable,
                                on_move,
                                armed_item_id,
                                dragging_item_id,
                                drop_target_anchor,
                                draggable_class,
                                drag_handle_title,
                                data_transfer_type,
                            )}
                            {draggable_panel_insert(
                                list_id_for_insert,
                                DraggablePanelListAnchor::After(item_id),
                                render_drop_zone,
                                on_move,
                                armed_item_id,
                                dragging_item_id,
                                drop_target_anchor,
                                drop_zone_class,
                                data_transfer_type,
                            )}
                        }
                    }
                />
            </div>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
fn draggable_panel_item(
    list_id: String,
    item_id: String,
    index: usize,
    items: Signal<Vec<DraggablePanelListItem>>,
    render_draggable: Callback<DraggablePanelListDraggable, AnyView>,
    on_move: Callback<DraggablePanelListMove>,
    armed_item_id: RwSignal<Option<String>>,
    dragging_item_id: RwSignal<Option<String>>,
    drop_target_anchor: RwSignal<Option<DraggablePanelListAnchor>>,
    draggable_class: &'static str,
    drag_handle_title: &'static str,
    data_transfer_type: &'static str,
) -> impl IntoView {
    let item_id_for_pointer = item_id.clone();
    let item_id_for_mouse = item_id.clone();
    let item_id_for_dragstart = item_id.clone();
    let item_id_for_dragover = item_id.clone();
    let item_id_for_drop = item_id.clone();
    let item_id_for_class = item_id.clone();
    let draggable_class_name = format!("draggable-panel-list__draggable {draggable_class}");
    let list_id_for_pointer = list_id.clone();
    let list_id_for_mouse = list_id.clone();
    let list_id_for_dragstart = list_id.clone();
    let list_id_for_dragover = list_id.clone();
    let list_id_for_drop = list_id.clone();
    let list_id_for_draggable_attr = list_id.clone();
    let list_id_for_handle_attr = list_id.clone();

    view! {
        <article
            class=draggable_class_name
            class:is-dragging=move || dragging_item_id.get().as_deref() == Some(item_id_for_class.as_str())
            data-draggable-panel-list-id=list_id_for_draggable_attr
            draggable="true"
            on:pointerdown=move |event| {
                if is_panel_drag_handle_target(&list_id_for_pointer, event.current_target(), event.target()) {
                    armed_item_id.set(Some(item_id_for_pointer.clone()));
                } else {
                    armed_item_id.set(None);
                }
            }
            on:mousedown=move |event| {
                if is_panel_drag_handle_target(&list_id_for_mouse, event.current_target(), event.target()) {
                    armed_item_id.set(Some(item_id_for_mouse.clone()));
                } else {
                    armed_item_id.set(None);
                }
            }
            on:dragstart=move |event| {
                if !is_own_draggable_event(&list_id_for_dragstart, event.current_target(), event.target()) {
                    return;
                }
                if armed_item_id.get().as_deref() != Some(item_id_for_dragstart.as_str()) {
                    event.prevent_default();
                    clear_draggable_panel_state(armed_item_id, dragging_item_id, drop_target_anchor);
                    return;
                }

                if let Some(data_transfer) = event.data_transfer() {
                    data_transfer.set_effect_allowed("move");
                    let _ = data_transfer.set_data(data_transfer_type, &item_id_for_dragstart);
                    let _ = data_transfer
                        .set_data("text/plain", &format!("draggable-panel:{item_id_for_dragstart}"));
                    if let Some(target) = event.current_target()
                        && let Ok(element) = target.dyn_into::<web_sys::Element>()
                    {
                        data_transfer.set_drag_image(&element, 24, 24);
                    }
                }

                dragging_item_id.set(Some(item_id_for_dragstart.clone()));
                drop_target_anchor.set(None);
            }
            on:dragend=move |_| {
                clear_draggable_panel_state(armed_item_id, dragging_item_id, drop_target_anchor);
            }
            on:dragover=move |event| {
                if is_own_draggable_event(&list_id_for_dragover, event.current_target(), event.target())
                    && dragging_item_id.get().is_some()
                    && let Some(anchor) =
                        panel_anchor_from_drag_event(&event, &items.get(), &item_id_for_dragover)
                {
                    event.prevent_default();
                    if let Some(data_transfer) = event.data_transfer() {
                        data_transfer.set_drop_effect("move");
                    }
                    drop_target_anchor.set(Some(anchor));
                }
            }
            on:drop=move |event| {
                if is_own_draggable_event(&list_id_for_drop, event.current_target(), event.target())
                    && let Some(anchor) =
                    panel_anchor_from_drag_event(&event, &items.get(), &item_id_for_drop)
                {
                    event.prevent_default();
                    let dragged_id = draggable_panel_id_from_drag_event(&event, data_transfer_type)
                        .or_else(|| dragging_item_id.get());
                    if let Some(dragged_id) = dragged_id {
                        on_move.run(DraggablePanelListMove { dragged_id, anchor });
                    }
                }
                clear_draggable_panel_state(armed_item_id, dragging_item_id, drop_target_anchor);
            }
        >
            <div class="draggable-panel-list__draggable-body">
                <span
                    class="draggable-panel-list__handle"
                    data-draggable-panel-list-id=list_id_for_handle_attr
                    title=drag_handle_title
                    aria-hidden="true"
                >
                    <GripVertical class="icon-button__icon"/>
                </span>
                <div class="draggable-panel-list__contents">
                    {render_draggable.run(DraggablePanelListDraggable { id: item_id, index })}
                </div>
            </div>
        </article>
    }
}

#[allow(clippy::too_many_arguments)]
fn draggable_panel_insert(
    list_id: String,
    anchor: DraggablePanelListAnchor,
    render_drop_zone: Callback<DraggablePanelListDropZone, AnyView>,
    on_move: Callback<DraggablePanelListMove>,
    armed_item_id: RwSignal<Option<String>>,
    dragging_item_id: RwSignal<Option<String>>,
    drop_target_anchor: RwSignal<Option<DraggablePanelListAnchor>>,
    drop_zone_class: &'static str,
    data_transfer_type: &'static str,
) -> impl IntoView {
    let anchor_for_class = anchor.clone();
    let anchor_for_enter = anchor.clone();
    let anchor_for_over = anchor.clone();
    let anchor_for_drop = anchor.clone();
    let is_drop_target =
        Signal::derive(move || drop_target_anchor.get() == Some(anchor_for_class.clone()));

    view! {
        <div
            class=drop_zone_class
            class:is-drop-target=move || is_drop_target.get()
            data-draggable-panel-list-id=list_id
            on:dragenter=move |event| {
                event.prevent_default();
                drop_target_anchor.set(Some(anchor_for_enter.clone()));
            }
            on:dragover=move |event| {
                event.prevent_default();
                if let Some(data_transfer) = event.data_transfer() {
                    data_transfer.set_drop_effect("move");
                }
                drop_target_anchor.set(Some(anchor_for_over.clone()));
            }
            on:drop=move |event| {
                event.prevent_default();
                let dragged_id = draggable_panel_id_from_drag_event(&event, data_transfer_type)
                    .or_else(|| dragging_item_id.get());
                if let Some(dragged_id) = dragged_id {
                    on_move.run(DraggablePanelListMove {
                        dragged_id,
                        anchor: anchor_for_drop.clone(),
                    });
                }
                clear_draggable_panel_state(armed_item_id, dragging_item_id, drop_target_anchor);
            }
        >
            {render_drop_zone.run(DraggablePanelListDropZone { anchor })}
        </div>
    }
}

fn clear_draggable_panel_state(
    armed_item_id: RwSignal<Option<String>>,
    dragging_item_id: RwSignal<Option<String>>,
    drop_target_anchor: RwSignal<Option<DraggablePanelListAnchor>>,
) {
    armed_item_id.set(None);
    dragging_item_id.set(None);
    drop_target_anchor.set(None);
}

fn draggable_panel_id_from_drag_event(
    event: &web_sys::DragEvent,
    data_transfer_type: &'static str,
) -> Option<String> {
    let data_transfer = event.data_transfer()?;
    let custom = data_transfer
        .get_data(data_transfer_type)
        .ok()
        .filter(|value| !value.trim().is_empty());

    custom.or_else(|| {
        data_transfer
            .get_data("text/plain")
            .ok()
            .and_then(|value| value.strip_prefix("draggable-panel:").map(str::to_string))
    })
}

fn panel_anchor_from_drag_event(
    event: &web_sys::DragEvent,
    items: &[DraggablePanelListItem],
    item_id: &str,
) -> Option<DraggablePanelListAnchor> {
    let target = event.current_target()?;
    let element = target.dyn_into::<web_sys::Element>().ok()?;
    let bounds_fn = js_sys::Reflect::get(&element, &"getBoundingClientRect".into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let bounds = bounds_fn.call0(&element).ok()?;
    let top = js_sys::Reflect::get(&bounds, &"top".into())
        .ok()?
        .as_f64()?;
    let height = js_sys::Reflect::get(&bounds, &"height".into())
        .ok()?
        .as_f64()?;
    let halfway_y = top + height / 2.0;

    if f64::from(event.client_y()) < halfway_y {
        anchor_before_item(items, item_id)
    } else {
        Some(DraggablePanelListAnchor::After(item_id.to_string()))
    }
}

fn anchor_before_item(
    items: &[DraggablePanelListItem],
    item_id: &str,
) -> Option<DraggablePanelListAnchor> {
    let index = items.iter().position(|item| item.id == item_id)?;

    if index == 0 {
        Some(DraggablePanelListAnchor::Start)
    } else {
        items
            .get(index - 1)
            .map(|item| DraggablePanelListAnchor::After(item.id.clone()))
    }
}

fn is_panel_drag_handle_target(
    list_id: &str,
    current_target: Option<web_sys::EventTarget>,
    target: Option<web_sys::EventTarget>,
) -> bool {
    let Some(current_draggable) = current_target.and_then(event_target_element) else {
        return false;
    };
    if !element_belongs_to_list(&current_draggable, list_id) {
        return false;
    }
    let Some(handle_owner) = target
        .and_then(event_target_element)
        .and_then(|element| {
            element
                .closest(".draggable-panel-list__handle")
                .ok()
                .flatten()
        })
        .and_then(|handle| {
            handle
                .closest(".draggable-panel-list__draggable")
                .ok()
                .flatten()
        })
    else {
        return false;
    };

    element_belongs_to_list(&handle_owner, list_id)
        && same_element(&current_draggable, &handle_owner)
}

fn is_own_draggable_event(
    list_id: &str,
    current_target: Option<web_sys::EventTarget>,
    target: Option<web_sys::EventTarget>,
) -> bool {
    let Some(current_draggable) = current_target.and_then(event_target_element) else {
        return false;
    };
    if !element_belongs_to_list(&current_draggable, list_id) {
        return false;
    }
    let Some(event_owner) = target.and_then(event_target_element).and_then(|element| {
        element
            .closest(".draggable-panel-list__draggable")
            .ok()
            .flatten()
    }) else {
        return false;
    };

    element_belongs_to_list(&event_owner, list_id) && same_element(&current_draggable, &event_owner)
}

fn event_target_element(target: web_sys::EventTarget) -> Option<web_sys::Element> {
    target.dyn_into::<web_sys::Element>().ok()
}

fn same_element(left: &web_sys::Element, right: &web_sys::Element) -> bool {
    js_sys::Object::is(left.as_ref(), right.as_ref())
}

fn element_belongs_to_list(element: &web_sys::Element, list_id: &str) -> bool {
    element
        .get_attribute("data-draggable-panel-list-id")
        .as_deref()
        == Some(list_id)
}

fn generated_draggable_panel_list_id() -> String {
    let id = NEXT_DRAGGABLE_PANEL_LIST_ID.fetch_add(1, Ordering::Relaxed);
    format!("draggable-panel-list-{id}")
}
