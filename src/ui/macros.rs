//! Builder methods shared by many `ui` types.

/// Adds `set`, the escape hatch that writes any key into the type's `extra`
/// map.
macro_rules! extra_methods {
    () => {
        /// Writes `key` with `value` into this object's JSON as is.
        ///
        /// Use it for a property Alfred added or renamed after this version of
        /// alfrusco, or to send a value the typed setter can't express, such as
        /// a placeholder string for a non-string property or a different
        /// `type`. A key set here replaces the typed property of the same name
        /// in the JSON built by [`Ui`](crate::ui::Ui),
        /// [`JsonAction`](crate::ui::JsonAction) and `serde_json::to_value`.
        /// Serializing a single component straight to text, as
        /// `serde_json::to_string` does, writes both keys.
        pub fn set(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
            self.extra.insert(key.into(), value.into());
            self
        }
    };
}

/// Adds the builder methods for the properties every component shares, which
/// live in the component's `common` field.
macro_rules! common_methods {
    () => {
        /// The component's identifier, used by `{value:id.property}`
        /// placeholders and by actions that target it. Keep ids unique across
        /// the whole UI.
        pub fn id(mut self, id: impl Into<String>) -> Self {
            self.common.id = Some(id.into());
            self
        }

        /// Hints that tell the parent view's layout where to place this
        /// component.
        pub fn constraints(mut self, constraints: $crate::ui::Constraints) -> Self {
            self.common.constraints = Some(constraints);
            self
        }

        /// Shorthand for the `region` constraint of a border layout, keeping
        /// any other constraints already set.
        pub fn region(mut self, region: $crate::ui::Region) -> Self {
            let constraints = self.common.constraints.take().unwrap_or_default();
            self.common.constraints = Some(constraints.region(region));
            self
        }

        /// Adds an event handler.
        pub fn event(mut self, event: $crate::ui::Event) -> Self {
            self.common.events.push(event);
            self
        }

        /// Adds an event handler that runs `action` when `trigger` fires.
        pub fn on(
            self,
            trigger: $crate::ui::Trigger,
            action: impl Into<$crate::ui::Action>,
        ) -> Self {
            self.event($crate::ui::Event::new(trigger).action(action))
        }

        /// Gives this component the initial focus. Set it on one component.
        pub fn focus(mut self, focus: bool) -> Self {
            self.common.focus = Some(focus);
            self
        }

        /// Hides the component. It still takes up its space in the layout.
        pub fn hidden(mut self, hidden: bool) -> Self {
            self.common.hidden = Some(hidden);
            self
        }

        /// Opacity from 0.0 to 1.0. Alfred's default is 1.0.
        pub fn opacity(mut self, opacity: f64) -> Self {
            self.common.opacity = Some(opacity);
            self
        }

        /// When false, the component is disabled and ignores user input.
        pub fn enabled(mut self, enabled: bool) -> Self {
            self.common.enabled = Some(enabled);
            self
        }

        /// Text shown when the pointer hovers over the component.
        pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
            self.common.tooltip = Some(tooltip.into());
            self
        }

        /// Adds an entry to the component's right-click menu.
        pub fn menu_item(mut self, item: $crate::ui::MenuItem) -> Self {
            self.common.menu.push(item);
            self
        }

        /// A key combination such as `"⌘S"` that triggers this component's
        /// action. Buttons, checkboxes and pickers support it.
        pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
            self.common.shortcut = Some(shortcut.into());
            self
        }

        /// VoiceOver label and help text.
        pub fn accessibility(mut self, accessibility: $crate::ui::Accessibility) -> Self {
            self.common.accessibility = Some(accessibility);
            self
        }

        extra_methods!();
    };
}
