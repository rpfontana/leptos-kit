use leptos::prelude::*;

/// libraries don't agree on these: lucide draws in a 24 unit box and is stroked,
/// phosphor uses 256, remix icon is filled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconStyle {
    pub view_box: &'static str,
    pub fill: &'static str,
    pub stroke: &'static str,
    pub stroke_width: &'static str,
    pub stroke_linecap: &'static str,
    pub stroke_linejoin: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconData {
    pub style: &'static IconStyle,
    /// inner SVG markup (`<path d="m6 9 6 6 6-6"/>`)
    pub body: &'static str,
}

/// size and colour come from CSS
#[component]
pub fn Icon(
    icon: IconData,
    #[prop(optional, into)] class: MaybeProp<String>,
    /// without it the icon is `aria-hidden`
    #[prop(optional, into)]
    label: MaybeProp<String>,
    #[prop(optional)] role: Option<&'static str>,
) -> impl IntoView {
    let style = icon.style;
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox=style.view_box
            fill=style.fill
            stroke=style.stroke
            stroke-width=style.stroke_width
            stroke-linecap=style.stroke_linecap
            stroke-linejoin=style.stroke_linejoin
            class=move || class.get()
            role=move || role.or_else(|| label.with(|label| label.as_ref().map(|_| "img")))
            aria-label=move || label.get()
            aria-hidden=move || label.with(|label| label.is_none().then_some("true"))
            inner_html=icon.body
        />
    }
}
