use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;
use leptos_kit_primitives as primitives;

#[component(element = "div", skip(role))]
pub fn Slider(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<Vec<f64>>>,
    #[prop(optional)] default_value: Option<Vec<f64>>,
    #[prop(optional, into)] on_value_change: Option<Callback<(Vec<f64>, primitives::ChangeReason)>>,
    #[prop(optional, into)] on_value_committed: Option<
        Callback<(Vec<f64>, primitives::ChangeReason)>,
    >,
    #[prop(into, default = 0.0.into())] min: Signal<f64>,
    #[prop(into, default = 100.0.into())] max: Signal<f64>,
    #[prop(into, default = 1.0.into())] step: Signal<f64>,
    #[prop(into, default = 10.0.into())] large_step: Signal<f64>,
    #[prop(optional, into)] min_steps_between_values: Signal<f64>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(into, default = primitives::ThumbAlignment::Edge.into())] thumb_alignment: Signal<
        primitives::ThumbAlignment,
    >,
    #[prop(into, default = primitives::ThumbCollisionBehavior::Push.into())]
    thumb_collision_behavior: Signal<primitives::ThumbCollisionBehavior>,
    #[prop(optional, into)] name: Option<Signal<String>>,
    #[prop(optional, into)] form: Option<Signal<String>>,
    #[prop(optional, into)] format: Option<Callback<f64, String>>,
) -> impl IntoView {
    // as many thumbs as values, and two without any, as in shadcn
    let defaults = default_value.as_ref().map_or(2, Vec::len);
    let default_value = default_value.unwrap_or_default();
    let thumbs = move || value.map_or(defaults, |value| value.with(Vec::len));
    view! {
        <primitives::SliderRoot
            value=value
            default_value=default_value
            on_value_change=on_value_change
            on_value_committed=on_value_committed
            min=min
            max=max
            step=step
            large_step=large_step
            min_steps_between_values=min_steps_between_values
            disabled=disabled
            orientation=orientation
            thumb_alignment=thumb_alignment
            thumb_collision_behavior=thumb_collision_behavior
            name=name
            form=form
            format=format
            class=classes("data-horizontal:w-full data-vertical:h-full", class)
            attr:data-slot="slider"
            {..attrs}
        >
            <primitives::SliderControl class="cn-slider">
                <primitives::SliderTrack class="cn-slider-track" attr:data-slot="slider-track">
                    <primitives::SliderIndicator
                        class="cn-slider-range"
                        attr:data-slot="slider-range"
                    />
                </primitives::SliderTrack>
                <For each=move || 0..thumbs() key=|index| *index let:_index>
                    <primitives::SliderThumb class="cn-slider-thumb" attr:data-slot="slider-thumb" />
                </For>
            </primitives::SliderControl>
        </primitives::SliderRoot>
    }
}
