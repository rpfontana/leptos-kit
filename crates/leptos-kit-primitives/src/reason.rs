// from here https://github.com/mui/base-ui/blob/v1.8.0/packages/react/src/internals/reason-parts.ts

use crate::Keyword;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Keyword)]
#[non_exhaustive]
pub enum ChangeReason {
    #[default]
    None,
    TriggerPress,
    TriggerHover,
    TriggerFocus,
    OutsidePress,
    ItemPress,
    ClosePress,
    LinkPress,
    ClearPress,
    ChipRemovePress,
    TrackPress,
    IncrementPress,
    DecrementPress,
    InputChange,
    InputClear,
    InputBlur,
    InputPaste,
    InputPress,
    FocusOut,
    EscapeKey,
    CloseWatcher,
    ListNavigation,
    Keyboard,
    Pointer,
    Drag,
    Wheel,
    Scrub,
    CancelOpen,
    SiblingOpen,
    Disabled,
    Missing,
    Initial,
    ImperativeAction,
    Swipe,
    WindowResize,
}
