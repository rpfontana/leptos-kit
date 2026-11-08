use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit::icons::{Icon, semantic};
use leptos_kit::macros::component;
use leptos_kit::primitives::{self, Orientation};

pub use primitives::{
    AnyCarouselPlugin, Autoplay, AutoplayOptions, CarouselApi, CarouselOptions, CarouselPlugin,
    ContainScroll, SlidesToScroll, use_carousel,
};

#[component(element = "div", skip(role))]
pub fn Carousel(
    #[prop(into, default = Orientation::Horizontal.into())] orientation: Signal<Orientation>,
    #[prop(optional, into)] opts: Signal<CarouselOptions>,
    #[prop(optional)] plugins: Vec<AnyCarouselPlugin>,
    #[prop(optional, into)] set_api: Option<Callback<CarouselApi>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CarouselRoot
            orientation=orientation
            opts=opts
            plugins=plugins
            set_api=set_api
            role="region"
            attr:aria-roledescription="carousel"
            attr:data-slot="carousel"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CarouselRoot>
    }
}

#[component(element = "div")]
pub fn CarouselContent(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="overflow-hidden" data-slot="carousel-content">
            <div {..primitives::carousel_container_attributes()} {..attrs}>
                {children.map(|children| children())}
            </div>
        </div>
    }
}

#[component(element = "div", skip(role))]
pub fn CarouselItem(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div
            role="group"
            aria-roledescription="slide"
            data-slot="carousel-item"
            {..primitives::carousel_slide_attributes()}
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "button", skip(r#type))]
pub fn CarouselPrevious(
    #[prop(into, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::IconSm.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
) -> impl IntoView {
    let carousel = use_carousel();
    let own = move || {
        format!(
            "{} cn-carousel-previous",
            button_classes(variant.get(), size.get())
        )
    };
    let disabled =
        move || disabled.map_or_else(|| !carousel.can_scroll_prev(), |disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..primitives::button_attributes(disabled, || false, tabindex)}
            type="button"
            class=classes(own, class)
            data-slot="carousel-previous"
            data-orientation=move || carousel.orientation().as_str()
            on:click=move |_| carousel.scroll_prev()
        >
            <Icon icon=semantic::CAROUSEL_CHEVRON_LEFT class="cn-rtl-flip" />
            <span class="sr-only">"Previous slide"</span>
        </button>
    }
}

#[component(element = "button", skip(r#type))]
pub fn CarouselNext(
    #[prop(into, default = ButtonVariant::Outline.into())] variant: Signal<ButtonVariant>,
    #[prop(into, default = ButtonSize::IconSm.into())] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
) -> impl IntoView {
    let carousel = use_carousel();
    let own = move || {
        format!(
            "{} cn-carousel-next",
            button_classes(variant.get(), size.get())
        )
    };
    let disabled =
        move || disabled.map_or_else(|| !carousel.can_scroll_next(), |disabled| disabled.get());
    view! {
        <button
            {..attrs}
            {..primitives::button_attributes(disabled, || false, tabindex)}
            type="button"
            class=classes(own, class)
            data-slot="carousel-next"
            data-orientation=move || carousel.orientation().as_str()
            on:click=move |_| carousel.scroll_next()
        >
            <Icon icon=semantic::CAROUSEL_CHEVRON_RIGHT class="cn-rtl-flip" />
            <span class="sr-only">"Next slide"</span>
        </button>
    }
}
