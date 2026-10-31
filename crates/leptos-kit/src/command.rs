use super::dialog::{Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle};
use super::input_group::{InputGroup, InputGroupAddon};
use super::utils::classes;
use leptos::prelude::*;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives::{self as primitives, ChangeReason, CommandFilter};

#[component(element = "div")]
pub fn Command(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] label: Option<Signal<String>>,
    #[prop(into, default = true.into())] should_filter: Signal<bool>,
    #[prop(optional, into)] filter: Option<CommandFilter>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: String,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional, into)] r#loop: Signal<bool>,
    #[prop(optional, into)] disable_pointer_selection: Signal<bool>,
    #[prop(into, default = true.into())] vim_bindings: Signal<bool>,
    #[prop(optional, into)] tabindex: Option<Signal<i32>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CommandRoot
            nostrip:label=label
            should_filter=should_filter
            nostrip:filter=filter
            nostrip:value=value
            default_value=default_value
            nostrip:on_value_change=on_value_change
            r#loop=r#loop
            disable_pointer_selection=disable_pointer_selection
            vim_bindings=vim_bindings
            nostrip:tabindex=tabindex
            class=classes("cn-command", class)
            attr:data-slot="command"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CommandRoot>
    }
}

#[leptos::component]
pub fn CommandDialog(
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(into, default = true.into())] modal: Signal<bool>,
    #[prop(optional, into)] disable_pointer_dismissal: Signal<bool>,
    #[prop(into, default = String::from("Command Palette"))] title: String,
    #[prop(into, default = String::from("Search for a command to run..."))] description: String,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] show_close_button: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <Dialog
            nostrip:open=open
            default_open=default_open
            nostrip:on_open_change=on_open_change
            modal=modal
            disable_pointer_dismissal=disable_pointer_dismissal
        >
            <DialogHeader class="sr-only">
                <DialogTitle>{title}</DialogTitle>
                <DialogDescription>{description}</DialogDescription>
            </DialogHeader>
            <DialogContent
                class=classes("cn-command-dialog", class)
                show_close_button=show_close_button
            >
                {children()}
            </DialogContent>
        </Dialog>
    }
}

#[component(element = "input", skip(r#type, role, id, autocomplete))]
pub fn CommandInput(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
) -> impl IntoView {
    view! {
        <div data-slot="command-input-wrapper" class="cn-command-input-wrapper">
            <InputGroup class="cn-command-input-group">
                <primitives::CommandInput
                    nostrip:value=value
                    nostrip:on_value_change=on_value_change
                    class=classes("cn-command-input", class)
                    attr:data-slot="command-input"
                    {..attrs}
                />
                <InputGroupAddon>
                    <Icon icon=semantic::COMMAND_SEARCH class="cn-command-input-icon" />
                </InputGroupAddon>
            </InputGroup>
        </div>
    }
}

#[component(element = "div", skip(role, tabindex, id))]
pub fn CommandList(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(into, default = Signal::stored(String::from("Suggestions")))] label: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CommandList
            label=label
            class=classes("cn-command-list", class)
            attr:data-slot="command-list"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CommandList>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandEmpty(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CommandEmpty
            class=classes("cn-command-empty", class)
            attr:data-slot="command-empty"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CommandEmpty>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] heading: Option<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] force_mount: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CommandGroup
            nostrip:heading=heading
            nostrip:value=value
            force_mount=force_mount
            class=classes("cn-command-group", class)
            attr:data-slot="command-group"
            {..attrs}
        >
            {children.map(|children| children())}
        </primitives::CommandGroup>
    }
}

#[component(element = "div", skip(role))]
pub fn CommandSeparator(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] always_render: bool,
) -> impl IntoView {
    view! {
        <primitives::CommandSeparator
            always_render=always_render
            class=classes("cn-command-separator", class)
            attr:data-slot="command-separator"
            {..attrs}
        />
    }
}

#[component(element = "div", skip(id, role))]
pub fn CommandItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] keywords: Vec<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_select: Option<Callback<String>>,
    #[prop(optional)] force_mount: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <primitives::CommandItem
            nostrip:value=value
            keywords=keywords
            disabled=disabled
            nostrip:on_select=on_select
            force_mount=force_mount
            class=classes("cn-command-item group/command-item", class)
            attr:data-slot="command-item"
            {..attrs}
        >
            {children.map(|children| children())}
            <Icon
                icon=semantic::COMMAND_CHECK
                class="cn-command-item-indicator"
            />
        </primitives::CommandItem>
    }
}

#[component(element = "span")]
pub fn CommandShortcut(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <span data-slot="command-shortcut" class=classes("cn-command-shortcut", class) {..attrs}>
            {children.map(|children| children())}
        </span>
    }
}
