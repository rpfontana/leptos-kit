use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "div")]
pub fn AspectRatio(ratio: f64, #[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div style:--ratio=ratio.to_string() data-slot="aspect-ratio" {..attrs}>
            {children.map(|children| children())}
        </div>
    }
}
