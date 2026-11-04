#[cfg(not(any(
    feature = "lucide",
    feature = "tabler",
    feature = "hugeicons",
    feature = "phosphor"
)))]
compile_error!("leptos-kit needs an icon feature: lucide, tabler, hugeicons or phosphor");

// the sources name the kit as an app does, so the cli copies them as they are
#[cfg(feature = "components")]
extern crate self as leptos_kit;

pub use leptos_kit_icons as icons;
pub use leptos_kit_macros as macros;
pub use leptos_kit_primitives as primitives;

#[cfg(feature = "components")]
mod accordion;
#[cfg(feature = "components")]
mod alert;
#[cfg(feature = "components")]
mod alert_dialog;
#[cfg(feature = "components")]
mod aspect_ratio;
#[cfg(feature = "components")]
mod attachment;
#[cfg(feature = "components")]
mod avatar;
#[cfg(feature = "components")]
mod badge;
#[cfg(feature = "components")]
mod breadcrumb;
#[cfg(feature = "components")]
mod bubble;
#[cfg(feature = "components")]
mod button;
#[cfg(feature = "components")]
mod button_group;
#[cfg(feature = "components")]
mod card;
#[cfg(feature = "components")]
mod checkbox;
#[cfg(feature = "components")]
mod collapsible;
#[cfg(feature = "components")]
mod combobox;
#[cfg(feature = "components")]
mod command;
#[cfg(feature = "components")]
mod context_menu;
#[cfg(feature = "components")]
mod dialog;
#[cfg(feature = "components")]
mod direction;
#[cfg(feature = "components")]
mod drawer;
#[cfg(feature = "components")]
mod dropdown_menu;
#[cfg(feature = "components")]
mod empty;
#[cfg(feature = "components")]
mod field;
#[cfg(feature = "components")]
mod hover_card;
#[cfg(feature = "components")]
mod input;
#[cfg(feature = "components")]
mod input_group;
#[cfg(feature = "components")]
mod input_otp;
#[cfg(feature = "components")]
mod item;
#[cfg(feature = "components")]
mod kbd;
#[cfg(feature = "components")]
mod label;
#[cfg(feature = "components")]
mod marker;
#[cfg(feature = "components")]
mod menubar;
#[cfg(feature = "components")]
mod message;
#[cfg(feature = "components")]
mod native_select;
#[cfg(feature = "components")]
mod navigation_menu;
#[cfg(feature = "components")]
mod pagination;
#[cfg(feature = "components")]
mod popover;
#[cfg(feature = "components")]
mod progress;
#[cfg(feature = "components")]
mod radio_group;
#[cfg(feature = "components")]
mod scroll_area;
#[cfg(feature = "components")]
mod select;
#[cfg(feature = "components")]
mod separator;
#[cfg(feature = "components")]
mod sheet;
#[cfg(feature = "components")]
mod sidebar;
#[cfg(feature = "components")]
mod skeleton;
#[cfg(feature = "components")]
mod slider;
#[cfg(feature = "components")]
mod spinner;
#[cfg(feature = "components")]
mod switch;
#[cfg(feature = "components")]
mod table;
#[cfg(feature = "components")]
mod tabs;
#[cfg(feature = "components")]
mod textarea;
#[cfg(feature = "components")]
mod toast;
#[cfg(feature = "components")]
mod toggle;
#[cfg(feature = "components")]
mod toggle_group;
#[cfg(feature = "components")]
mod tooltip;
#[cfg(feature = "components")]
mod utils;

#[cfg(feature = "components")]
pub use accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
#[cfg(feature = "components")]
pub use alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
#[cfg(feature = "components")]
pub use alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogMedia, AlertDialogOverlay, AlertDialogPortal,
    AlertDialogSize, AlertDialogTitle, AlertDialogTrigger,
};
#[cfg(feature = "components")]
pub use aspect_ratio::AspectRatio;
#[cfg(feature = "components")]
pub use attachment::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentGroup, AttachmentMedia, AttachmentMediaVariant, AttachmentSize, AttachmentState,
    AttachmentTitle, AttachmentTrigger,
};
#[cfg(feature = "components")]
pub use avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
};
#[cfg(feature = "components")]
pub use badge::{Badge, BadgeVariant};
#[cfg(feature = "components")]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
    BreadcrumbSeparator,
};
#[cfg(feature = "components")]
pub use bubble::{
    Bubble, BubbleAlign, BubbleContent, BubbleGroup, BubbleReactions, BubbleReactionsSide,
    BubbleVariant,
};
#[cfg(feature = "components")]
pub use button::{Button, ButtonSize, ButtonVariant};
#[cfg(feature = "components")]
pub use button_group::{ButtonGroup, ButtonGroupSeparator, ButtonGroupText};
#[cfg(feature = "components")]
pub use card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardSize, CardTitle,
};
#[cfg(feature = "components")]
pub use checkbox::Checkbox;
#[cfg(feature = "components")]
pub use collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
#[cfg(feature = "components")]
pub use combobox::{
    Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxClear, ComboboxCollection,
    ComboboxContent, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxLabel,
    ComboboxList, ComboboxSeparator, ComboboxTrigger, ComboboxValue, use_combobox_anchor,
};
#[cfg(feature = "components")]
pub use command::{
    Command, CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList,
    CommandSeparator, CommandShortcut,
};
#[cfg(feature = "components")]
pub use context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuGroup, ContextMenuItem,
    ContextMenuItemVariant, ContextMenuLabel, ContextMenuPortal, ContextMenuRadioGroup,
    ContextMenuRadioItem, ContextMenuSeparator, ContextMenuShortcut, ContextMenuSub,
    ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
#[cfg(feature = "components")]
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};
#[cfg(feature = "components")]
pub use direction::{Direction, DirectionProvider, use_direction};
#[cfg(feature = "components")]
pub use drawer::{
    Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader,
    DrawerOverlay, DrawerPortal, DrawerSnapPoint, DrawerSwipeHandle, DrawerTitle, DrawerTrigger,
};
#[cfg(feature = "components")]
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup,
    DropdownMenuItem, DropdownMenuItemVariant, DropdownMenuLabel, DropdownMenuPortal,
    DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuSeparator, DropdownMenuShortcut,
    DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
#[cfg(feature = "components")]
pub use empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
#[cfg(feature = "components")]
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
#[cfg(feature = "components")]
pub use hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};
#[cfg(feature = "components")]
pub use input::Input;
#[cfg(feature = "components")]
pub use input_group::{
    InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton, InputGroupButtonSize,
    InputGroupInput, InputGroupText, InputGroupTextarea,
};
#[cfg(feature = "components")]
pub use input_otp::{
    InputMode, InputOtp, InputOtpGroup, InputOtpSeparator, InputOtpSlot, REGEXP_ONLY_CHARS,
    REGEXP_ONLY_DIGITS, REGEXP_ONLY_DIGITS_AND_CHARS,
};
#[cfg(feature = "components")]
pub use item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant,
};
#[cfg(feature = "components")]
pub use kbd::{Kbd, KbdGroup};
#[cfg(feature = "components")]
pub use label::Label;
#[cfg(feature = "components")]
pub use marker::{Marker, MarkerContent, MarkerIcon, MarkerVariant};
#[cfg(feature = "components")]
pub use menubar::{
    Menubar, MenubarCheckboxItem, MenubarContent, MenubarGroup, MenubarItem, MenubarItemVariant,
    MenubarLabel, MenubarMenu, MenubarPortal, MenubarRadioGroup, MenubarRadioItem,
    MenubarSeparator, MenubarShortcut, MenubarSub, MenubarSubContent, MenubarSubTrigger,
    MenubarTrigger,
};
#[cfg(feature = "components")]
pub use message::{
    Message, MessageAlign, MessageAvatar, MessageContent, MessageFooter, MessageGroup,
    MessageHeader,
};
#[cfg(feature = "components")]
pub use native_select::{NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize};
#[cfg(feature = "components")]
pub use navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem,
    NavigationMenuLink, NavigationMenuList, NavigationMenuPositioner, NavigationMenuTrigger,
    navigation_menu_trigger_style,
};
#[cfg(feature = "components")]
pub use pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationNext, PaginationPrevious,
};
#[cfg(feature = "components")]
pub use popover::{
    Popover, PopoverContent, PopoverDescription, PopoverHeader, PopoverTitle, PopoverTrigger,
};
#[cfg(feature = "components")]
pub use progress::{Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue};
#[cfg(feature = "components")]
pub use radio_group::{RadioGroup, RadioGroupItem};
#[cfg(feature = "components")]
pub use scroll_area::{ScrollArea, ScrollBar};
#[cfg(feature = "components")]
pub use select::{
    Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectScrollDownButton,
    SelectScrollUpButton, SelectSeparator, SelectTrigger, SelectTriggerSize, SelectValue,
};
#[cfg(feature = "components")]
pub use separator::Separator;
#[cfg(feature = "components")]
pub use sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay,
    SheetPortal, SheetSide, SheetTitle, SheetTrigger,
};
#[cfg(feature = "components")]
pub use sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarContext, SidebarFooter, SidebarGroup,
    SidebarGroupAction, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput,
    SidebarInset, SidebarMenu, SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton,
    SidebarMenuButtonSize, SidebarMenuButtonVariant, SidebarMenuItem, SidebarMenuSkeleton,
    SidebarMenuSub, SidebarMenuSubButton, SidebarMenuSubButtonSize, SidebarMenuSubItem,
    SidebarProvider, SidebarRail, SidebarRender, SidebarSeparator, SidebarSide, SidebarState,
    SidebarTrigger, SidebarVariant, use_sidebar,
};
#[cfg(feature = "components")]
pub use skeleton::Skeleton;
#[cfg(feature = "components")]
pub use slider::Slider;
#[cfg(feature = "components")]
pub use spinner::Spinner;
#[cfg(feature = "components")]
pub use switch::{Switch, SwitchSize};
#[cfg(feature = "components")]
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
#[cfg(feature = "components")]
pub use tabs::{Tabs, TabsContent, TabsList, TabsListVariant, TabsTrigger};
#[cfg(feature = "components")]
pub use textarea::Textarea;
#[cfg(feature = "components")]
pub use toast::{
    SwipeDirection, Toast, ToastAction, ToastActionOptions, ToastClose, ToastContent,
    ToastDescription, ToastManager, ToastObject, ToastOptions, ToastPortal, ToastPriority,
    ToastProvider, ToastTitle, ToastViewport, Toaster, UseToastManager, create_toast_manager,
    toast, use_toast_manager,
};
#[cfg(feature = "components")]
pub use toggle::{Toggle, ToggleSize, ToggleVariant};
#[cfg(feature = "components")]
pub use toggle_group::{ToggleGroup, ToggleGroupItem};
#[cfg(feature = "components")]
pub use tooltip::{Tooltip, TooltipContent, TooltipProvider, TooltipTrigger};

pub use leptos_kit_primitives::{
    Align, ChangeReason, ImageLoadingStatus, Orientation, Side, ThumbAlignment,
    ThumbCollisionBehavior,
};
