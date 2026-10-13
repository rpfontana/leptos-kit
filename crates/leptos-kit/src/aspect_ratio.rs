use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn AspectRatio(
    #[prop(into)] ratio: Signal<f64>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div style:--ratio=move || ratio.get().to_string() data-slot="aspect-ratio" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
