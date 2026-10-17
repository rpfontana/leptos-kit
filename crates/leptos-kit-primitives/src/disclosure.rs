use crate::id::ElementId;
use crate::native::after_animations;
use crate::{Controlled, reason, use_id};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::style::style;

// what collapsible and accordion items share: the panel's presence, size and transition
#[derive(Clone, Copy)]
pub(crate) struct Disclosure {
    pub(crate) open: Controlled<bool>,
    pub(crate) disabled: Signal<bool>,
    pub(crate) panel: NodeRef<html::Div>,
    pub(crate) panel_id: ElementId<html::Div>,
    // base ui's transition status: idle, starting or ending
    phase: RwSignal<&'static str>,
    shown: RwSignal<bool>,
    size: RwSignal<Option<(i32, i32)>>,
    found: StoredValue<bool>,
}

impl Disclosure {
    pub(crate) fn new(open: Controlled<bool>, disabled: Signal<bool>) -> Self {
        let panel = NodeRef::new();
        let this = Self {
            open,
            disabled,
            panel,
            panel_id: ElementId::new(panel, use_id()),
            phase: RwSignal::new("idle"),
            shown: RwSignal::new(open.get_untracked()),
            size: RwSignal::new(None),
            found: StoredValue::new(false),
        };
        Effect::new(move |previous: Option<bool>| {
            let now = open.get();
            if previous.is_some_and(|previous| previous != now) {
                this.animate(now);
            }
            now
        });
        this
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open.get()
    }

    pub(crate) fn is_hidden(&self) -> bool {
        !self.shown.get()
    }

    pub(crate) fn toggle(&self) {
        self.open
            .set(!self.open.get_untracked(), reason::TRIGGER_PRESS);
    }

    pub(crate) fn phase_attributes(self) -> impl leptos::attr::Attribute {
        let phase = self.phase;
        view! {
            <{..}
                data-starting-style=move || (phase.get() == "starting").then_some("")
                data-ending-style=move || (phase.get() == "ending").then_some("")
            />
        }
    }

    pub(crate) fn open_attributes(self) -> impl leptos::attr::Attribute {
        let open = self.open;
        view! {
            <{..}
                data-open=move || open.get().then_some("")
                data-closed=move || (!open.get()).then_some("")
            />
        }
    }

    // hidden or found by find in page, and the size its animation reads
    pub(crate) fn panel_attributes(
        self,
        height: &'static str,
        width: &'static str,
        hidden_until_found: Signal<bool>,
    ) -> impl leptos::attr::Attribute {
        // find in page shows it at once, so it opens without the animation
        let found = leptos::tachys::html::event::on(
            ev::Custom::<ev::Event>::new("beforematch"),
            move |_| {
                self.found.set_value(true);
                self.open.set(true, reason::NONE);
                if !self.open.get_untracked() {
                    self.found.set_value(false);
                }
            },
        );
        let size = self.size;
        let dimension = move |pick: fn((i32, i32)) -> i32| {
            move || {
                size.get()
                    .map_or_else(|| "auto".to_string(), |size| format!("{}px", pick(size)))
            }
        };
        let shown = self.shown;
        let hidden = view! {
            <{..} hidden=move || {
                (!shown.get()).then(|| if hidden_until_found.get() { "until-found" } else { "" })
            } />
        };
        (
            hidden,
            style((height, dimension(|(height, _)| height))),
            style((width, dimension(|(_, width)| width))),
            found,
        )
    }

    fn animate(self, open: bool) {
        let Self {
            panel,
            phase,
            shown,
            size,
            found,
            ..
        } = self;
        let Some(Some(element)) = panel.try_get_untracked() else {
            shown.try_set(open);
            return;
        };
        let still = move |open: bool| self.open.try_get_untracked() == Some(open);
        if open {
            if found.try_update_value(std::mem::take).unwrap_or(false) {
                shown.set(true);
                return;
            }
            let _ = element.remove_attribute("hidden");
            size.set(Some(measure(&element)));
            shown.set(true);
            phase.set("starting");
            after_paint(move || {
                if !still(true) {
                    return;
                }
                phase.try_set("idle");
                after_animations(element.into(), move || {
                    if still(true) {
                        size.try_set(None);
                    }
                });
            });
        } else {
            // auto can't animate to the ending style, so it's pinned to its size first
            size.set(Some(measure(&element)));
            after_paint(move || {
                if !still(false) {
                    return;
                }
                phase.try_set("ending");
                after_animations(element.into(), move || {
                    if still(false) {
                        shown.try_set(false);
                        phase.try_set("idle");
                        size.try_set(None);
                    }
                });
            });
        }
    }
}

// firefox styles the next frame without the one before, so a transition needs a frame painted in between
fn after_paint(done: impl FnOnce() + 'static) {
    request_animation_frame(move || request_animation_frame(done));
}

fn measure(element: &web_sys::HtmlDivElement) -> (i32, i32) {
    (element.scroll_height(), element.scroll_width())
}
