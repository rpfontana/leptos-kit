use crate::anchor::{Placement, placement};
use crate::button::button_attributes;
use crate::id::ElementId;
use crate::{Align, ChangeReason, Controlled, Native, NativeOpen, Side, keyword, use_id};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[derive(Clone, Copy)]
struct PopoverContext {
    id: ElementId<html::Div>,
    native: NativeOpen<html::Div>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    trigger: NodeRef<html::Button>,
    title: RwSignal<Option<String>>,
    description: RwSignal<Option<String>>,
}

impl PopoverContext {
    fn open(&self) -> bool {
        self.native.state().get()
    }

    fn popup_id(&self) -> String {
        format!("{}-popup", self.id.get())
    }

    fn anchor(&self) -> String {
        format!("--{}", self.id.get())
    }
}

#[leptos::component]
pub fn PopoverRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    children: Children,
) -> impl IntoView {
    let positioner = NodeRef::new();
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    let context = PopoverContext {
        id: ElementId::new(positioner, use_id()),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        positioner,
        popup,
        trigger: NodeRef::new(),
        title: RwSignal::new(None),
        description: RwSignal::new(None),
    };
    // components share their parent's owner, a sibling root would overwrite the context
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(element = "button", skip(r#type, popovertarget, popovertargetaction))]
pub fn PopoverTrigger(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    let open = move || context.open();

    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            node_ref=context.trigger
            type="button"
            style:anchor-name=context.anchor()
            popovertarget=context.id.get()
            aria-haspopup="dialog"
            aria-expanded=move || open().to_string()
            aria-controls=move || open().then(|| context.popup_id())
            data-popup-open=move || open().then_some("")
            data-pressed=move || open().then_some("")
            on:click=move |event: ev::MouseEvent| {
                // a native close would skip the exit animation
                if context.native.state().get_untracked() {
                    event.prevent_default();
                    context.native.state().set(false, ChangeReason::TriggerPress);
                } else {
                    context.native.expect(ChangeReason::TriggerPress);
                }
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "div", skip(id, style, role))]
pub fn PopoverPositioner(
    #[prop(into, default = Side::Bottom.into())] side: Signal<Side>,
    #[prop(into, default = Align::Center.into())] align: Signal<Align>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let placed = Placement::new(side, align);
    let open = move || context.open();
    let style = move || {
        placement(
            &context.anchor(),
            side.get(),
            align.get(),
            side_offset.get(),
            align_offset.get(),
        )
    };

    view! {
        <div
            {..attrs}
            node_ref=context.positioner
            id=context.id.get()
            popover="auto"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(align)
            on:toggle=move |_| {
                context.native.sync(ChangeReason::OutsidePress);
                match context.native.state().try_get_untracked() {
                    Some(true) => request_animation_frame(move || settle(context, placed)),
                    Some(false) => {
                        placed.flipped.try_set(false);
                    }
                    None => {}
                }
            }
        >
            <Provider value=placed>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(id, role))]
pub fn PopoverPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let placed = expect_context::<Placement>();
    let open = move || context.open();
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            id=context.popup_id()
            role="dialog"
            tabindex="-1"
            aria-labelledby=move || context.title.get()
            aria-describedby=move || context.description.get()
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(placed.align)
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button", skip(r#type, popovertarget, popovertargetaction))]
pub fn PopoverClose(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let disabled = move || disabled.is_some_and(|disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..button_attributes(disabled, || false, tabindex)}
            type="button"
            popovertarget=context.id.get()
            popovertargetaction="hide"
            on:click=move |event: ev::MouseEvent| {
                event.prevent_default();
                context.native.state().set(false, ChangeReason::ClosePress);
            }
        >
            {children.map(|children| children())}
        </button>
    }
}

#[component(element = "h2")]
pub fn PopoverTitle(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let id = id.unwrap_or_else(|| format!("{}-title", context.id.get()));
    context.title.set(Some(id.clone()));
    on_cleanup(move || context.title.set(None));
    view! {
        <h2 {..attrs} id=id>
            {children.map(|children| children())}
        </h2>
    }
}

#[component(element = "p")]
pub fn PopoverDescription(
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PopoverContext>();
    let id = id.unwrap_or_else(|| format!("{}-description", context.id.get()));
    context.description.set(Some(id.clone()));
    on_cleanup(move || context.description.set(None));
    view! {
        <p {..attrs} id=id>
            {children.map(|children| children())}
        </p>
    }
}

fn settle(context: PopoverContext, placed: Placement) {
    if let (Some(Some(trigger)), Some(Some(positioner))) = (
        context.trigger.try_get_untracked(),
        context.positioner.try_get_untracked(),
    ) {
        placed.settle(&trigger, &positioner);
    }
}
