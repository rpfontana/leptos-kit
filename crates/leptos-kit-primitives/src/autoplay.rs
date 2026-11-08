use crate::carousel::{CarouselApi, CarouselPlugin};
use crate::native::listen;
use leptos::prelude::*;
use std::time::Duration;
use web_sys::{ScrollBehavior, VisibilityState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "embla's autoplay options, one flag each"
)]
pub struct AutoplayOptions {
    pub delay: Duration,
    pub jump: bool,
    pub play_on_init: bool,
    pub stop_on_interaction: bool,
    pub stop_on_mouse_enter: bool,
    pub stop_on_focus_in: bool,
    pub stop_on_last_snap: bool,
}

impl Default for AutoplayOptions {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(4000),
            jump: false,
            play_on_init: true,
            stop_on_interaction: true,
            stop_on_mouse_enter: false,
            stop_on_focus_in: true,
            stop_on_last_snap: false,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct State {
    api: Option<CarouselApi>,
    timer: Option<TimeoutHandle>,
    resume: bool,
    destroyed: bool,
}

// embla-carousel-autoplay
#[derive(Clone, Copy)]
pub struct Autoplay {
    options: AutoplayOptions,
    state: StoredValue<State>,
    playing: RwSignal<bool>,
}

impl std::fmt::Debug for Autoplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Autoplay")
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

impl Autoplay {
    pub fn new(options: AutoplayOptions) -> Self {
        Self {
            options,
            state: StoredValue::new(State::default()),
            playing: RwSignal::new(false),
        }
    }

    pub fn play(self) {
        self.start();
    }

    pub fn stop(self) {
        if self.playing.get_untracked() {
            self.halt();
        }
    }

    pub fn reset(self) {
        if self.playing.get_untracked() {
            self.start();
        }
    }

    pub fn is_playing(self) -> bool {
        self.playing.get()
    }

    fn start(self) {
        let Some(state) = self.state.try_get_value() else {
            return;
        };
        if state.destroyed || state.api.is_none() {
            return;
        }
        if hidden() {
            self.state.update_value(|state| state.resume = true);
            return;
        }
        self.clear();
        let timer = set_timeout_with_handle(move || self.next(), self.options.delay).ok();
        self.state.update_value(|state| state.timer = timer);
        self.playing.set(true);
    }

    fn halt(self) {
        if self
            .state
            .try_get_value()
            .is_none_or(|state| state.destroyed)
        {
            return;
        }
        self.clear();
        self.playing.try_set(false);
    }

    fn clear(self) {
        if let Some(Some(timer)) = self.state.try_update_value(|state| state.timer.take()) {
            timer.clear();
        }
    }

    fn next(self) {
        let Some(api) = self.state.try_get_value().and_then(|state| state.api) else {
            return;
        };
        let (count, selected, can_scroll) = untrack(|| {
            (
                api.scroll_snap_list().len(),
                api.selected_scroll_snap(),
                api.can_scroll_next(),
            )
        });
        let last = self.options.stop_on_last_snap && selected + 2 == count;
        let behavior = if self.options.jump {
            ScrollBehavior::Instant
        } else {
            ScrollBehavior::Smooth
        };
        if can_scroll {
            api.step(true, behavior);
        } else {
            api.go(0, behavior);
        }
        if last {
            self.halt();
        } else {
            self.start();
        }
    }
}

impl CarouselPlugin for Autoplay {
    fn init(&self, api: CarouselApi) {
        let this = *self;
        this.state.update_value(|state| {
            state.api = Some(api);
            state.destroyed = false;
        });
        if untrack(|| api.scroll_snap_list().len()) <= 1 {
            return;
        }
        let Some(container) = api.container_node() else {
            return;
        };
        let root = container
            .parent_element()
            .unwrap_or_else(|| container.clone().into());
        let options = this.options;
        let playing = this.playing;

        listen(&document(), "visibilitychange", move || {
            if hidden() {
                this.state
                    .update_value(|state| state.resume = playing.get_untracked());
                this.halt();
            } else if this.state.try_get_value().is_some_and(|state| state.resume) {
                this.start();
            }
        });
        if api.watches_drag() {
            listen(&container, "pointerdown", move || this.halt());
            if !options.stop_on_interaction {
                listen(&container, "pointerup", move || this.start());
            }
        }
        if options.stop_on_mouse_enter {
            listen(&root, "mouseenter", move || this.halt());
            if !options.stop_on_interaction {
                listen(&root, "mouseleave", move || this.start());
            }
        }
        if options.stop_on_focus_in {
            listen(&container, "focusin", move || this.halt());
            if !options.stop_on_interaction {
                listen(&container, "focusout", move || this.start());
            }
        }
        if options.play_on_init {
            this.start();
        }
    }

    fn destroy(&self) {
        self.halt();
        self.state.try_update_value(|state| state.destroyed = true);
        self.playing.try_set(false);
    }
}

fn hidden() -> bool {
    document().visibility_state() == VisibilityState::Hidden
}
