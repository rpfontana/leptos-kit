use crate::anchor::{Placement, placement};
use crate::dismiss::dismiss;
use crate::hover::Hover;
use crate::id::ElementId;
use crate::{Align, ChangeReason, Controlled, Native, NativeOpen, Side, keyword, use_id};
use leptos::context::Provider;
use leptos::either::Either;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_kit_macros::component;
use web_sys::{Element, HtmlElement, Node};

const OPEN_DELAY: u32 = 600;
const CLOSE_DELAY: u32 = 300;

#[derive(Clone, Copy)]
struct PreviewCardContext {
    id: ElementId<html::Div>,
    native: NativeOpen<html::Div>,
    positioner: NodeRef<html::Div>,
    popup: NodeRef<html::Div>,
    link: NodeRef<html::A>,
    button: NodeRef<html::Button>,
    close_delay: StoredValue<u32>,
    hover: Hover,
}

impl PreviewCardContext {
    fn open(&self) -> bool {
        self.native.state().get()
    }

    fn anchor(&self) -> String {
        format!("--{}", self.id.get())
    }

    fn set(self, open: bool, reason: ChangeReason) {
        if open != self.native.state().get_untracked() {
            self.native.state().set(open, reason);
        }
    }

    fn opener(self, reason: ChangeReason) -> Callback<()> {
        Callback::new(move |()| self.set(true, reason))
    }

    fn closer(self, reason: ChangeReason) -> Callback<()> {
        Callback::new(move |()| self.set(false, reason))
    }

    fn trigger(&self) -> Option<Element> {
        let link = self.link.try_get_untracked().flatten().map(Element::from);
        link.or_else(|| self.button.try_get_untracked().flatten().map(Element::from))
    }

    // the two boxes the pointer may cross between while the card stays
    fn boxes(&self) -> Option<[Element; 2]> {
        let popup = self.popup.try_get_untracked().flatten()?;
        Some([self.trigger()?, popup.into()])
    }

    fn contains(&self, node: &Node) -> bool {
        let positioner = self
            .positioner
            .try_get_untracked()
            .flatten()
            .map(Node::from);
        [self.trigger().map(Node::from), positioner]
            .iter()
            .flatten()
            .any(|part| part.contains(Some(node)))
    }

    // focus moving elsewhere on the page closes it, into the card or off the page it doesn't
    fn blurred(self, event: &ev::FocusEvent, reason: ChangeReason) {
        let to = event
            .related_target()
            .and_then(|target| target.dyn_into::<Node>().ok());
        if to.is_some_and(|node| !self.contains(&node)) {
            self.set(false, reason);
        }
    }

    fn cross(self, event: &ev::PointerEvent) {
        let close = self.closer(ChangeReason::TriggerHover);
        let delay = self.close_delay.get_value();
        match self.boxes() {
            Some(boxes) if self.open() => {
                let from = (f64::from(event.client_x()), f64::from(event.client_y()));
                self.hover.cross(from, boxes, delay, close);
            }
            _ => self.hover.leave(delay, close),
        }
    }
}

// a manual popover, like the tooltip, but a press outside or esc closes it
#[leptos::component]
pub fn PreviewCardRoot(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    children: Children,
) -> impl IntoView {
    let positioner = NodeRef::new();
    let popup = NodeRef::new();
    let state = Controlled::new(open, default_open, on_open_change);
    let context = PreviewCardContext {
        id: ElementId::new(positioner, use_id()),
        native: NativeOpen::new(Native::Popover, positioner, state, Some(popup)),
        positioner,
        popup,
        link: NodeRef::new(),
        button: NodeRef::new(),
        close_delay: StoredValue::new(CLOSE_DELAY),
        hover: Hover::new(),
    };
    dismiss(
        move || state.get(),
        move |node| context.contains(node),
        move |reason| context.set(false, reason),
    );
    // components share their parent's owner, a sibling root would overwrite the context
    view! { <Provider value=context>{children()}</Provider> }
}

#[component(element = "a")]
pub fn PreviewCardTrigger(
    #[prop(optional_no_strip, into)] delay: Option<Signal<u32>>,
    #[prop(optional_no_strip, into)] close_delay: Option<Signal<u32>>,
    // a button in place of base ui's link, as `render={<button />}`
    #[prop(optional)] native_button: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PreviewCardContext>();
    let open = move || context.open();
    let delay = move || delay.map_or(OPEN_DELAY, |delay| delay.get_untracked());
    let close_delay = move || close_delay.map_or(CLOSE_DELAY, |delay| delay.get_untracked());
    let mouse = |event: &ev::PointerEvent| event.pointer_type() == "mouse";
    let enter = move |event: ev::PointerEvent| {
        if mouse(&event) {
            context.close_delay.set_value(close_delay());
            context
                .hover
                .enter(delay(), context.opener(ChangeReason::TriggerHover));
        }
    };
    let leave = move |event: ev::PointerEvent| {
        if mouse(&event) {
            context.cross(&event);
        }
    };
    let focus = move |event: ev::FocusEvent| {
        // focus from the keyboard only, as base ui's
        let visible = event_target::<HtmlElement>(&event)
            .matches(":focus-visible")
            .unwrap_or(false);
        if visible {
            context.close_delay.set_value(close_delay());
            context
                .hover
                .enter(delay(), context.opener(ChangeReason::TriggerFocus));
        }
    };
    let blur = move |event: ev::FocusEvent| {
        context.hover.cancel();
        context.blurred(&event, ChangeReason::TriggerFocus);
    };
    let popup_open = move || open().then_some("");

    if native_button {
        Either::Left(view! {
            <button
                {..attrs}
                node_ref=context.button
                type="button"
                style:anchor-name=context.anchor()
                data-popup-open=popup_open
                on:pointerenter=enter
                on:pointerleave=leave
                on:focus=focus
                on:blur=blur
            >
                {children.map(|children| children())}
            </button>
        })
    } else {
        Either::Right(view! {
            <a
                {..attrs}
                node_ref=context.link
                style:anchor-name=context.anchor()
                data-popup-open=popup_open
                on:pointerenter=enter
                on:pointerleave=leave
                on:focus=focus
                on:blur=blur
            >
                {children.map(|children| children())}
            </a>
        })
    }
}

#[component(element = "div", skip(id, style, role))]
pub fn PreviewCardPositioner(
    #[prop(into, default = Side::Bottom.into())] side: Signal<Side>,
    #[prop(into, default = Align::Center.into())] align: Signal<Align>,
    #[prop(optional, into)] side_offset: Signal<f64>,
    #[prop(optional, into)] align_offset: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let context = expect_context::<PreviewCardContext>();
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
    // show_popover runs in the root's effect, so the boxes are in place a microtask later
    Effect::new(move || {
        if context.native.state().get() {
            queue_microtask(move || {
                if let (Some(trigger), Some(Some(positioner))) =
                    (context.trigger(), context.positioner.try_get_untracked())
                {
                    placed.settle(&trigger, &positioner);
                }
            });
        } else {
            placed.flipped.try_set(false);
        }
    });
    view! {
        <div
            {..attrs}
            node_ref=context.positioner
            id=context.id.get()
            popover="manual"
            role="presentation"
            style=style
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(align)
            on:toggle=move |_| context.native.sync(ChangeReason::None)
        >
            <Provider value=placed>{children.map(|children| children())}</Provider>
        </div>
    }
}

#[component(element = "div", skip(tabindex))]
pub fn PreviewCardPopup(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let context = expect_context::<PreviewCardContext>();
    let placed = expect_context::<Placement>();
    let open = move || context.open();
    view! {
        <div
            {..attrs}
            node_ref=context.popup
            tabindex="-1"
            style:position="relative"
            style:left=placed.nudge(|(x, _)| x)
            style:top=placed.nudge(|(_, y)| y)
            data-open=move || open().then_some("")
            data-closed=move || (!open()).then_some("")
            data-side=move || placed.side().as_str()
            data-align=keyword(placed.align)
            on:focusout=move |event: ev::FocusEvent| context.blurred(&event, ChangeReason::FocusOut)
            on:pointerenter=move |_| context.hover.hold()
            on:pointerleave=move |event: ev::PointerEvent| {
                if event.pointer_type() == "mouse" {
                    context.cross(&event);
                }
            }
        >
            {children.map(|children| children())}
        </div>
    }
}
