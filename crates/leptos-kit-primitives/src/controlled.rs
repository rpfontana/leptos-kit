use crate::ChangeReason;
use leptos::prelude::*;

// like base ui's useControlled
pub(crate) struct Controlled<T: Send + Sync + 'static> {
    value: Signal<T>,
    owned: Option<RwSignal<T>>,
    on_change: Option<Callback<(T, ChangeReason)>>,
}

impl<T: Send + Sync + 'static> Clone for Controlled<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Copy for Controlled<T> {}

impl<T> Controlled<T>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    pub(crate) fn new(
        controlled: Option<Signal<T>>,
        default: T,
        on_change: Option<Callback<(T, ChangeReason)>>,
    ) -> Self {
        if let Some(value) = controlled {
            Self {
                value,
                owned: None,
                on_change,
            }
        } else {
            let owned = RwSignal::new(default);
            Self {
                value: owned.into(),
                owned: Some(owned),
                on_change,
            }
        }
    }

    pub(crate) fn is_controlled(&self) -> bool {
        self.owned.is_none()
    }

    // one of the reason
    pub(crate) fn set(&self, next: T, reason: ChangeReason) {
        if self.value.with_untracked(|value| *value == next) {
            return;
        }
        if let Some(on_change) = self.on_change {
            on_change.run((next.clone(), reason));
        }
        if let Some(owned) = self.owned {
            owned.set(next);
        }
    }
}

impl<T: Send + Sync + 'static> DefinedAt for Controlled<T> {
    fn defined_at(&self) -> Option<&'static std::panic::Location<'static>> {
        self.value.defined_at()
    }
}

impl<T: Clone + Send + Sync + 'static> ReadUntracked for Controlled<T> {
    type Value = <Signal<T> as ReadUntracked>::Value;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        self.value.try_read_untracked()
    }
}

impl<T: Send + Sync + 'static> Track for Controlled<T> {
    fn track(&self) {
        self.value.track();
    }
}

impl<T: Send + Sync + 'static> From<Controlled<T>> for Signal<T> {
    fn from(controlled: Controlled<T>) -> Self {
        controlled.value
    }
}
