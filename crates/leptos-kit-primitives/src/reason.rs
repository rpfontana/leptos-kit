// from here https://github.com/mui/base-ui/blob/v1.8.0/packages/react/src/internals/reason-parts.ts

crate::keywords! {
    #[derive(Default)]
    #[non_exhaustive]
    pub enum ChangeReason {
        #[default]
        None = "none",
        TriggerPress = "trigger-press",
        TriggerHover = "trigger-hover",
        TriggerFocus = "trigger-focus",
        OutsidePress = "outside-press",
        ItemPress = "item-press",
        ClosePress = "close-press",
        LinkPress = "link-press",
        ClearPress = "clear-press",
        ChipRemovePress = "chip-remove-press",
        TrackPress = "track-press",
        IncrementPress = "increment-press",
        DecrementPress = "decrement-press",
        InputChange = "input-change",
        InputClear = "input-clear",
        InputBlur = "input-blur",
        InputPaste = "input-paste",
        InputPress = "input-press",
        FocusOut = "focus-out",
        EscapeKey = "escape-key",
        CloseWatcher = "close-watcher",
        ListNavigation = "list-navigation",
        Keyboard = "keyboard",
        Pointer = "pointer",
        Drag = "drag",
        Wheel = "wheel",
        Scrub = "scrub",
        CancelOpen = "cancel-open",
        SiblingOpen = "sibling-open",
        Disabled = "disabled",
        Missing = "missing",
        Initial = "initial",
        ImperativeAction = "imperative-action",
        Swipe = "swipe",
        WindowResize = "window-resize",
    }
}
