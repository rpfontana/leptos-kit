use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "table")]
pub fn Table(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div data-slot="table-container" class="cn-table-container">
            <table class=classes("cn-table", class) data-slot="table" {..attrs}>
                {children.map(|children| children())}
            </table>
        </div>
    }
}

#[component(element = "thead")]
pub fn TableHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <thead class=classes("cn-table-header", class) data-slot="table-header" {..attrs}>
            {children.map(|children| children())}
        </thead>
    }
}

#[component(element = "tbody")]
pub fn TableBody(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <tbody class=classes("cn-table-body", class) data-slot="table-body" {..attrs}>
            {children.map(|children| children())}
        </tbody>
    }
}

#[component(element = "tfoot")]
pub fn TableFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <tfoot class=classes("cn-table-footer", class) data-slot="table-footer" {..attrs}>
            {children.map(|children| children())}
        </tfoot>
    }
}

#[component(element = "tr")]
pub fn TableRow(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <tr class=classes("cn-table-row", class) data-slot="table-row" {..attrs}>
            {children.map(|children| children())}
        </tr>
    }
}

#[component(element = "th")]
pub fn TableHead(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <th class=classes("cn-table-head", class) data-slot="table-head" {..attrs}>
            {children.map(|children| children())}
        </th>
    }
}

#[component(element = "td")]
pub fn TableCell(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <td class=classes("cn-table-cell", class) data-slot="table-cell" {..attrs}>
            {children.map(|children| children())}
        </td>
    }
}

#[component(element = "caption")]
pub fn TableCaption(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <caption class=classes("cn-table-caption", class) data-slot="table-caption" {..attrs}>
            {children.map(|children| children())}
        </caption>
    }
}
