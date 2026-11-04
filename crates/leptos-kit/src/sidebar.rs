use super::button::{ButtonSize, ButtonVariant, button_classes};
use super::sheet::{
    Sheet, SheetDescription, SheetHeader, SheetOverlay, SheetPortal, SheetSide, SheetTitle,
};
use super::skeleton::Skeleton;
use super::tooltip::TooltipContent;
use super::utils::classes;
use leptos::attr::Attribute;
use leptos::attr::custom::custom_attribute;
use leptos::either::{Either, EitherOf3};
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::html::class::class as class_attribute;
use leptos::web_sys::Element;
use leptos_kit_icons::{Icon, semantic};
use leptos_kit_macros::component;
use leptos_kit_primitives::{self as primitives, ChangeReason};
use std::hash::{BuildHasher, RandomState};

pub use primitives::{SidebarContext, SidebarState, use_sidebar};

const WIDTH: &str = "16rem";
const WIDTH_MOBILE: &str = "18rem";
const WIDTH_ICON: &str = "3rem";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum SidebarMenuButtonVariant {
    #[default]
    Default,
    Outline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
#[non_exhaustive]
pub enum SidebarMenuButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, primitives::Keyword)]
pub enum SidebarMenuSubButtonSize {
    Sm,
    #[default]
    Md,
}

// the part renders as that trigger, with the trigger's data-slot and the sidebar's look
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidebarRender {
    CollapsibleTrigger,
    DropdownMenuTrigger,
}

#[component(element = "div")]
pub fn SidebarProvider(
    #[prop(default = true)] default_open: bool,
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional, into)] on_open_change: Option<Callback<bool>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] style: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let style = move || {
        let own = format!("--sidebar-width: {WIDTH}; --sidebar-width-icon: {WIDTH_ICON}");
        match style {
            Some(style) => format!("{own}; {}", style.get()),
            None => own,
        }
    };
    view! {
        <primitives::SidebarRoot default_open=default_open open=open on_open_change=on_open_change>
            <div
                class=classes(
                    "group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-sidebar",
                    class,
                )
                style=style
                data-slot="sidebar-wrapper"
                {..attrs}
            >
                {children.map(|children| children())}
            </div>
        </primitives::SidebarRoot>
    }
}

#[component(element = "div")]
pub fn Sidebar(
    #[prop(into, default = SidebarSide::Left.into())] side: Signal<SidebarSide>,
    #[prop(into, default = SidebarVariant::Sidebar.into())] variant: Signal<SidebarVariant>,
    #[prop(into, default = SidebarCollapsible::Offcanvas.into())] collapsible: Signal<
        SidebarCollapsible,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] dir: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    if collapsible.get_untracked() == SidebarCollapsible::None {
        return Either::Left(view! {
            <aside
                class=classes("flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground", class)
                data-slot="sidebar"
                {..attrs}
            >
                {children.map(|children| children())}
            </aside>
        });
    }
    let sidebar = use_sidebar();
    let floating = move || {
        matches!(
            variant.get(),
            SidebarVariant::Floating | SidebarVariant::Inset
        )
    };
    let gap = move || {
        if floating() {
            "cn-sidebar-gap group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]"
        } else {
            "cn-sidebar-gap group-data-[collapsible=icon]:w-(--sidebar-width-icon)"
        }
    };
    let container = move || {
        let own = "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear data-[side=left]:left-0 data-[side=left]:group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)] data-[side=right]:right-0 data-[side=right]:group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)] md:flex";
        if floating() {
            format!(
                "{own} p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]"
            )
        } else {
            format!(
                "{own} group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l"
            )
        }
    };
    let collapsed = move || match sidebar.state() {
        SidebarState::Collapsed => collapsible.get().as_str(),
        SidebarState::Expanded => "",
    };
    let sheet_side = Signal::derive(move || match side.get() {
        SidebarSide::Left => SheetSide::Left,
        SidebarSide::Right => SheetSide::Right,
    });

    // a phone gets the sheet, so the children move into it and back
    let inner = NodeRef::<html::Div>::new();
    let mobile = NodeRef::<html::Div>::new();
    Effect::new(move |host: Option<Option<Element>>| {
        let host = host.flatten();
        let is_mobile = sidebar.is_mobile();
        let (Some(inner), wrapper) = (inner.get(), mobile.get()) else {
            return host;
        };
        let inner: Element = inner.into();
        let target = match wrapper {
            Some(wrapper) if is_mobile && wrapper.is_connected() => {
                // its parent is hidden on a phone, and a hidden dialog never shows
                if let (Ok(Some(dialog)), Some(body)) =
                    (wrapper.closest("dialog"), document().body())
                {
                    let _ = body.append_child(&dialog);
                }
                wrapper.into()
            }
            _ => inner.clone(),
        };
        let from = host.unwrap_or(inner);
        if from != target {
            while let Some(child) = from.first_child() {
                let _ = target.append_child(&child);
            }
        }
        Some(target)
    });

    Either::Right(view! {
        <aside
            class="group peer hidden text-sidebar-foreground md:block"
            data-state=move || sidebar.state().as_str()
            data-collapsible=collapsed
            data-variant=primitives::keyword(variant)
            data-side=primitives::keyword(side)
            data-slot="sidebar"
        >
            <div data-slot="sidebar-gap" class=gap></div>
            <div
                data-slot="sidebar-container"
                data-side=primitives::keyword(side)
                class=classes(container, class)
                {..attrs}
            >
                <div node_ref=inner data-sidebar="sidebar" data-slot="sidebar-inner" class="cn-sidebar-inner">
                    {children.map(|children| children())}
                </div>
            </div>
            {move || {
                sidebar
                    .is_mobile()
                    .then(|| {
                        view! {
                            <Sheet
                                open=Signal::derive(move || sidebar.open_mobile())
                                on_open_change=Callback::new(move |(open, _): (bool, ChangeReason)| {
                                    sidebar.set_open_mobile(open);
                                })
                            >
                                // sheet content's own parts, as its popup takes the sidebar's data-slot
                                <SheetPortal>
                                    <SheetOverlay />
                                    <primitives::DialogPopup
                                        class="cn-sheet-content w-(--sidebar-width) bg-sidebar p-0 text-sidebar-foreground [&>button]:hidden"
                                        style=format!("--sidebar-width: {WIDTH_MOBILE}")
                                        attr:dir=move || dir.map(|dir| dir.get())
                                        attr:data-side=primitives::keyword(sheet_side)
                                        attr:data-sidebar="sidebar"
                                        attr:data-slot="sidebar"
                                        attr:data-mobile="true"
                                    >
                                        <SheetHeader class="sr-only">
                                            <SheetTitle>"Sidebar"</SheetTitle>
                                            <SheetDescription>"Displays the mobile sidebar."</SheetDescription>
                                        </SheetHeader>
                                        <div node_ref=mobile class="flex h-full w-full flex-col"></div>
                                        <primitives::DialogClose
                                            class=format!("{} cn-sheet-close", button_classes(ButtonVariant::Ghost, ButtonSize::IconSm))
                                            attr:data-slot="sheet-close"
                                        >
                                            <Icon icon=semantic::SHEET_X />
                                            <span class="sr-only">"Close"</span>
                                        </primitives::DialogClose>
                                    </primitives::DialogPopup>
                                </SheetPortal>
                            </Sheet>
                        }
                    })
            }}
        </aside>
    })
}

#[component(element = "button", skip(r#type))]
pub fn SidebarTrigger(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    let sidebar = use_sidebar();
    let own = move || {
        format!(
            "{} cn-sidebar-trigger",
            button_classes(ButtonVariant::Ghost, ButtonSize::IconSm)
        )
    };
    view! {
        <button
            {..attrs}
            {..primitives::button_attributes(|| false, || false, None)}
            type="button"
            class=classes(own, class)
            data-slot="sidebar-trigger"
            data-sidebar="trigger"
            on:click=move |_| sidebar.toggle_sidebar()
        >
            <Icon icon=semantic::SIDEBAR_PANEL_LEFT class="cn-rtl-flip" />
            <span class="sr-only">"Toggle Sidebar"</span>
        </button>
    }
}

#[component(element = "button", skip(tabindex, title))]
pub fn SidebarRail(#[prop(optional, into)] class: Option<Signal<String>>) -> impl IntoView {
    let sidebar = use_sidebar();
    view! {
        <button
            data-sidebar="rail"
            data-slot="sidebar-rail"
            aria-label="Toggle Sidebar"
            tabindex="-1"
            title="Toggle Sidebar"
            class=classes("cn-sidebar-rail", class)
            on:click=move |_| sidebar.toggle_sidebar()
            {..attrs}
        ></button>
    }
}

#[component(element = "main")]
pub fn SidebarInset(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <main class=classes("cn-sidebar-inset", class) data-slot="sidebar-inset" {..attrs}>
            {children.map(|children| children())}
        </main>
    }
}

#[component(element = "input")]
pub fn SidebarInput(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional, into)] on_value_change: Option<Callback<(String, ChangeReason)>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <input
            {..attrs}
            {..primitives::input_attributes(value, default_value, on_value_change, disabled)}
            class=classes("cn-input cn-sidebar-input", class)
            data-slot="sidebar-input"
            data-sidebar="input"
        />
    }
}

#[component(element = "header")]
pub fn SidebarHeader(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header
            class=classes("cn-sidebar-header", class)
            data-slot="sidebar-header"
            data-sidebar="header"
            {..attrs}
        >
            {children.map(|children| children())}
        </header>
    }
}

#[component(element = "footer")]
pub fn SidebarFooter(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <footer
            class=classes("cn-sidebar-footer", class)
            data-slot="sidebar-footer"
            data-sidebar="footer"
            {..attrs}
        >
            {children.map(|children| children())}
        </footer>
    }
}

#[component(element = "div", skip(role))]
pub fn SidebarSeparator(
    #[prop(into, default = primitives::Orientation::Horizontal.into())] orientation: Signal<
        primitives::Orientation,
    >,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <div
            role="separator"
            aria-orientation=primitives::keyword(orientation)
            data-orientation=primitives::keyword(orientation)
            class=classes("cn-sidebar-separator", class)
            data-slot="sidebar-separator"
            data-sidebar="separator"
            {..attrs}
        />
    }
}

#[component(element = "nav")]
pub fn SidebarContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <nav
            class=classes("cn-sidebar-content", class)
            data-slot="sidebar-content"
            data-sidebar="content"
            {..attrs}
        >
            {children.map(|children| children())}
        </nav>
    }
}

#[component(element = "section")]
pub fn SidebarGroup(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <section
            class=classes("cn-sidebar-group", class)
            data-slot="sidebar-group"
            data-sidebar="group"
            {..attrs}
        >
            {children.map(|children| children())}
        </section>
    }
}

#[component(element = "div")]
pub fn SidebarGroupLabel(
    #[prop(optional)] render: Option<SidebarRender>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = (
        attrs,
        class_attribute(classes("cn-sidebar-group-label", class)),
        custom_attribute("data-sidebar", "group-label"),
    );
    match render {
        Some(render) => Either::Left(rendered(render, look, children)),
        None => Either::Right(view! {
            <div data-slot="sidebar-group-label" {..look}>
                {children.map(|children| children())}
            </div>
        }),
    }
}

#[component(element = "button", skip(r#type))]
pub fn SidebarGroupAction(
    #[prop(optional)] render: Option<SidebarRender>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let look = (
        attrs,
        class_attribute(classes("cn-sidebar-group-action", class)),
        custom_attribute("data-sidebar", "group-action"),
    );
    match render {
        Some(render) => Either::Left(rendered(render, look, children)),
        None => Either::Right(view! {
            <button type="button" data-slot="sidebar-group-action" {..look}>
                {children.map(|children| children())}
            </button>
        }),
    }
}

#[component(element = "div")]
pub fn SidebarGroupContent(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-sidebar-group-content", class)
            data-slot="sidebar-group-content"
            data-sidebar="group-content"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "ul")]
pub fn SidebarMenu(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <ul class=classes("cn-sidebar-menu", class) data-slot="sidebar-menu" data-sidebar="menu" {..attrs}>
            {children.map(|children| children())}
        </ul>
    }
}

// collapsible makes the item itself the collapsible's root, rather than a div around it
#[component(element = "li")]
pub fn SidebarMenuItem(
    #[prop(optional)] collapsible: bool,
    #[prop(optional, into)] open: Option<Signal<bool>>,
    #[prop(optional)] default_open: bool,
    #[prop(optional, into)] on_open_change: Option<Callback<(bool, ChangeReason)>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let class = classes("group/menu-item relative", class);
    if collapsible {
        return Either::Left(view! {
            <primitives::CollapsibleRoot
                list_item=true
                open=open
                default_open=default_open
                on_open_change=on_open_change
                class=class
                attr:data-slot="collapsible"
                attr:data-sidebar="menu-item"
                {..attrs}
            >
                {children.map(|children| children())}
            </primitives::CollapsibleRoot>
        });
    }
    Either::Right(view! {
        <li class=class data-slot="sidebar-menu-item" data-sidebar="menu-item" {..attrs}>
            {children.map(|children| children())}
        </li>
    })
}

#[component(element = "button", skip(r#type))]
pub fn SidebarMenuButton(
    #[prop(optional)] render: Option<SidebarRender>,
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(optional, into)] variant: Signal<SidebarMenuButtonVariant>,
    #[prop(optional, into)] size: Signal<SidebarMenuButtonSize>,
    #[prop(optional, into)] tooltip: Option<Signal<String>>,
    #[prop(optional, into)] tooltip_hidden: Option<Signal<bool>>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        format!(
            "cn-sidebar-menu-button peer/menu-button group/menu-button cn-sidebar-menu-button-variant-{} cn-sidebar-menu-button-size-{}",
            variant.get().as_str(),
            size.get().as_str()
        )
    };
    let look = (
        attrs,
        class_attribute(classes(own, class)),
        custom_attribute("data-sidebar", "menu-button"),
        custom_attribute("data-size", primitives::keyword(size)),
        custom_attribute("data-active", move || is_active.get().then_some("")),
    );
    // a menu's trigger names its own anchor, which a tooltip's would replace
    let tooltip = tooltip.filter(|_| render != Some(SidebarRender::DropdownMenuTrigger));
    let Some(tooltip) = tooltip else {
        return Either::Left(menu_button(render, href, look, children));
    };
    let sidebar = use_sidebar();
    let hidden = tooltip_hidden.unwrap_or_else(|| {
        Signal::derive(move || sidebar.state() != SidebarState::Collapsed || sidebar.is_mobile())
    });
    Either::Right(view! {
        <primitives::TooltipRoot disabled=hidden>
            {menu_button(
                render,
                href,
                (look, primitives::tooltip_trigger_attributes(None, None, true.into())),
                children,
            )}
            <TooltipContent side=primitives::Side::Right align=primitives::Align::Center>
                {move || tooltip.get()}
            </TooltipContent>
        </primitives::TooltipRoot>
    })
}

fn menu_button(
    render: Option<SidebarRender>,
    href: Option<Signal<String>>,
    look: impl Attribute,
    children: Option<Children>,
) -> impl IntoView {
    if let Some(render) = render {
        return EitherOf3::A(rendered(render, look, children));
    }
    match href {
        Some(href) => EitherOf3::B(view! {
            <a href=href data-slot="sidebar-menu-button" {..look}>
                {children.map(|children| children())}
            </a>
        }),
        None => EitherOf3::C(view! {
            <button type="button" data-slot="sidebar-menu-button" {..look}>
                {children.map(|children| children())}
            </button>
        }),
    }
}

fn rendered(
    render: SidebarRender,
    look: impl Attribute,
    children: Option<Children>,
) -> impl IntoView {
    match render {
        SidebarRender::CollapsibleTrigger => Either::Left(view! {
            <primitives::CollapsibleTrigger attr:data-slot="collapsible-trigger" {..look}>
                {children.map(|children| children())}
            </primitives::CollapsibleTrigger>
        }),
        SidebarRender::DropdownMenuTrigger => Either::Right(view! {
            <primitives::MenuTrigger attr:data-slot="dropdown-menu-trigger" {..look}>
                {children.map(|children| children())}
            </primitives::MenuTrigger>
        }),
    }
}

#[component(element = "button", skip(r#type))]
pub fn SidebarMenuAction(
    #[prop(optional)] render: Option<SidebarRender>,
    #[prop(optional, into)] show_on_hover: Signal<bool>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let own = move || {
        if show_on_hover.get() {
            "cn-sidebar-menu-action group-focus-within/menu-item:opacity-100 group-hover/menu-item:opacity-100 peer-data-active/menu-button:text-sidebar-accent-foreground aria-expanded:opacity-100 md:opacity-0".to_string()
        } else {
            "cn-sidebar-menu-action".to_string()
        }
    };
    let look = (
        attrs,
        class_attribute(classes(own, class)),
        custom_attribute("data-sidebar", "menu-action"),
    );
    match render {
        Some(render) => Either::Left(rendered(render, look, children)),
        None => Either::Right(view! {
            <button type="button" data-slot="sidebar-menu-action" {..look}>
                {children.map(|children| children())}
            </button>
        }),
    }
}

#[component(element = "div")]
pub fn SidebarMenuBadge(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=classes("cn-sidebar-menu-badge", class)
            data-slot="sidebar-menu-badge"
            data-sidebar="menu-badge"
            {..attrs}
        >
            {children.map(|children| children())}
        </div>
    }
}

#[component(element = "div")]
pub fn SidebarMenuSkeleton(
    #[prop(optional)] show_icon: bool,
    #[prop(optional, into)] class: Option<Signal<String>>,
) -> impl IntoView {
    // between 50 and 90%, kept from the server's html when the page hydrates
    let width = RandomState::new().hash_one(()) % 40 + 50;
    view! {
        <div
            class=classes("cn-sidebar-menu-skeleton", class)
            data-slot="sidebar-menu-skeleton"
            data-sidebar="menu-skeleton"
            {..attrs}
        >
            {show_icon
                .then(|| {
                    view! {
                        <Skeleton
                            class="cn-sidebar-menu-skeleton-icon"
                            attr:data-sidebar="menu-skeleton-icon"
                        />
                    }
                })}
            <Skeleton
                class="cn-sidebar-menu-skeleton-text"
                style=format!("--skeleton-width: {width}%")
                attr:data-sidebar="menu-skeleton-text"
            />
        </div>
    }
}

#[component(element = "ul")]
pub fn SidebarMenuSub(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <ul
            class=classes("cn-sidebar-menu-sub", class)
            data-slot="sidebar-menu-sub"
            data-sidebar="menu-sub"
            {..attrs}
        >
            {children.map(|children| children())}
        </ul>
    }
}

#[component(element = "li")]
pub fn SidebarMenuSubItem(
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <li
            class=classes("group/menu-sub-item relative", class)
            data-slot="sidebar-menu-sub-item"
            data-sidebar="menu-sub-item"
            {..attrs}
        >
            {children.map(|children| children())}
        </li>
    }
}

#[component(element = "a")]
pub fn SidebarMenuSubButton(
    #[prop(into, default = SidebarMenuSubButtonSize::Md.into())] size: Signal<
        SidebarMenuSubButtonSize,
    >,
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <a
            class=classes("cn-sidebar-menu-sub-button", class)
            data-slot="sidebar-menu-sub-button"
            data-sidebar="menu-sub-button"
            data-size=primitives::keyword(size)
            data-active=move || is_active.get().then_some("")
            {..attrs}
        >
            {children.map(|children| children())}
        </a>
    }
}
