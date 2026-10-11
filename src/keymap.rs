//! One effective map for dispatch, help and footer. No executable or writer is stored here.

use crate::input::{InputEvent, KeyChord, KeyCode, Modifiers, admits_input};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeymapProfile {
    #[default]
    Standard,
    Vim,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NavigationPolicy {
    Direct,
    #[default]
    Pane,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Product actions are identities, never authorization or executable commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Move(Direction),
    MoveScreen(i8),
    Page(i8),
    HalfPage(i8),
    Home,
    End,
    FocusNext,
    FocusPrevious,
    Open,
    Cancel,
    RequestQuit,
    Help,
    Refresh,
    ToggleDensity,
    Search,
    EnterInsert,
    Product(String),
}

impl Action {
    pub fn accepts_count(&self) -> bool {
        matches!(
            self,
            Self::Move(_) | Self::MoveScreen(_) | Self::Page(_) | Self::HalfPage(_)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionEffect {
    View,
    ReadOnlyQuery,
    ProductWriteRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionDescriptor {
    pub id: String,
    pub label: String,
    pub action: Action,
    pub effect: ActionEffect,
    /// Empty means any normal context. Nonempty restricts registration to these contexts.
    pub contexts: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionIntent {
    pub action: Action,
    pub effect: ActionEffect,
    pub count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ActionRegistry {
    actions: BTreeMap<String, ActionDescriptor>,
}

impl ActionRegistry {
    pub fn standard(navigation: NavigationPolicy) -> Self {
        let mut registry = Self::default();
        for (id, label, action, effect) in [
            (
                "ui.move.up",
                "Up",
                match navigation {
                    NavigationPolicy::Direct => Action::MoveScreen(-1),
                    NavigationPolicy::Pane => Action::Move(Direction::Up),
                },
                ActionEffect::View,
            ),
            (
                "ui.move.down",
                "Down",
                match navigation {
                    NavigationPolicy::Direct => Action::MoveScreen(1),
                    NavigationPolicy::Pane => Action::Move(Direction::Down),
                },
                ActionEffect::View,
            ),
            (
                "ui.move.left",
                "Left",
                Action::Move(Direction::Left),
                ActionEffect::View,
            ),
            (
                "ui.move.right",
                "Right",
                Action::Move(Direction::Right),
                ActionEffect::View,
            ),
            (
                "ui.viewport.line_up",
                "Scroll up",
                Action::Move(Direction::Up),
                ActionEffect::View,
            ),
            (
                "ui.viewport.line_down",
                "Scroll down",
                Action::Move(Direction::Down),
                ActionEffect::View,
            ),
            (
                "ui.viewport.page_up",
                "Page up",
                Action::Page(-1),
                ActionEffect::View,
            ),
            (
                "ui.viewport.page_down",
                "Page down",
                Action::Page(1),
                ActionEffect::View,
            ),
            (
                "ui.viewport.half_up",
                "Half page up",
                Action::HalfPage(-1),
                ActionEffect::View,
            ),
            (
                "ui.viewport.half_down",
                "Half page down",
                Action::HalfPage(1),
                ActionEffect::View,
            ),
            (
                "ui.viewport.home",
                "First row",
                Action::Home,
                ActionEffect::View,
            ),
            (
                "ui.viewport.end",
                "Last row",
                Action::End,
                ActionEffect::View,
            ),
            (
                "ui.focus.next",
                "Next pane",
                Action::FocusNext,
                ActionEffect::View,
            ),
            (
                "ui.focus.previous",
                "Previous pane",
                Action::FocusPrevious,
                ActionEffect::View,
            ),
            ("ui.open", "Open", Action::Open, ActionEffect::View),
            (
                "ui.cancel",
                "Back / cancel",
                Action::Cancel,
                ActionEffect::View,
            ),
            (
                "ui.quit",
                "Request quit",
                Action::RequestQuit,
                ActionEffect::View,
            ),
            ("ui.help", "Help", Action::Help, ActionEffect::View),
            (
                "ui.refresh",
                "Refresh",
                Action::Refresh,
                ActionEffect::ReadOnlyQuery,
            ),
            (
                "ui.view.toggle_density",
                "Smart / Raw",
                Action::ToggleDensity,
                ActionEffect::View,
            ),
            ("ui.search", "Search", Action::Search, ActionEffect::View),
            (
                "ui.mode.insert",
                "Insert",
                Action::EnterInsert,
                ActionEffect::View,
            ),
        ] {
            registry.actions.insert(
                id.into(),
                ActionDescriptor {
                    id: id.into(),
                    label: label.into(),
                    action,
                    effect,
                    contexts: vec![],
                },
            );
        }
        registry
    }

    /// Namespace and immutable descriptor prevent user files from inventing actions.
    pub fn register_product(&mut self, descriptor: ActionDescriptor) -> Result<(), KeymapError> {
        if descriptor.id.starts_with("ui.")
            || !descriptor.id.contains('.')
            || self.actions.contains_key(&descriptor.id)
            || descriptor.action != Action::Product(descriptor.id.clone())
        {
            return Err(KeymapError(
                "product action must be a unique namespaced identity".into(),
            ));
        }
        self.actions.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ActionDescriptor> {
        self.actions.get(id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capture {
    Normal,
    Editor,
    Overlay,
    Confirm,
}

/// Contexts are ordered deepest owner first, global last. `focus_token` changes
/// whenever the product changes pane/screen identity, even for identical contexts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveContext {
    pub contexts: Vec<String>,
    pub mode: InputMode,
    pub capture: Capture,
    pub editable: bool,
    pub focus_token: u64,
}

impl ResolveContext {
    pub fn normal(contexts: &[&str]) -> Self {
        let mut seen = BTreeSet::new();
        Self {
            contexts: contexts
                .iter()
                .filter(|s| !s.is_empty() && **s != "global" && seen.insert(**s))
                .map(|s| (*s).into())
                .chain(std::iter::once("global".into()))
                .collect(),
            mode: InputMode::Normal,
            capture: Capture::Normal,
            editable: false,
            focus_token: 0,
        }
    }
    fn owners(&self) -> &[String] {
        if matches!(
            self.capture,
            Capture::Overlay | Capture::Confirm | Capture::Editor
        ) {
            &self.contexts[..self.contexts.len().min(1)]
        } else {
            &self.contexts
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    Intent(ActionIntent),
    Text(String),
    EditorKey(crate::input::KeyEvent),
    Pending,
    Consumed,
    NoBinding,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingOrigin {
    BuiltIn,
    Product,
    User,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub context: String,
    pub mode: InputMode,
    pub keys: Vec<KeyChord>,
    pub action_id: String,
    pub origin: BindingOrigin,
}

impl Binding {
    pub fn new(context: &str, keys: &[&str], action: &str) -> Result<Self, KeymapError> {
        Ok(Self {
            context: context.into(),
            mode: InputMode::Normal,
            keys: keys
                .iter()
                .map(|s| parse_chord(s))
                .collect::<Result<_, _>>()?,
            action_id: action.into(),
            origin: BindingOrigin::Product,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Diagnostic {
    Shadow {
        context: String,
        keys: String,
        hidden_context: String,
    },
    UnreachableAction(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeymapError(pub String);
impl fmt::Display for KeymapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for KeymapError {}

#[derive(Clone, Debug)]
pub struct KeymapRegistry {
    actions: ActionRegistry,
    bindings: Vec<Binding>,
    pub profile: KeymapProfile,
    pub navigation: NavigationPolicy,
    pub sequence_timeout: Duration,
    modal_write_requests: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveHint {
    pub keys: String,
    pub label: String,
    pub action_id: String,
    pub origin: BindingOrigin,
}

impl KeymapRegistry {
    /// Initial load from explicitly supplied configuration text. The product owns
    /// file selection and whether failure uses its last accepted/base registry.
    pub fn from_toml(
        text: &str,
        product_actions: Vec<ActionDescriptor>,
        product_bindings: Vec<Binding>,
    ) -> Result<Self, KeymapError> {
        let config: Config = toml::from_str(text).map_err(|e| KeymapError(e.to_string()))?;
        let profile = config.preset.unwrap_or_default();
        let navigation = config.navigation.unwrap_or_default();
        let mut actions = ActionRegistry::standard(navigation);
        for action in product_actions {
            actions.register_product(action)?;
        }
        Self::new(profile, navigation, actions, product_bindings)?.overlay_toml(text)
    }
    pub fn new(
        profile: KeymapProfile,
        navigation: NavigationPolicy,
        actions: ActionRegistry,
        product: Vec<Binding>,
    ) -> Result<Self, KeymapError> {
        let mut map = Self {
            actions,
            bindings: vec![],
            profile,
            navigation,
            sequence_timeout: Duration::from_millis(600),
            modal_write_requests: BTreeSet::new(),
        };
        for (key, id) in [
            ("Up", "ui.move.up"),
            ("Down", "ui.move.down"),
            ("Left", "ui.move.left"),
            ("Right", "ui.move.right"),
            ("PageUp", "ui.viewport.page_up"),
            ("PageDown", "ui.viewport.page_down"),
            ("Home", "ui.viewport.home"),
            ("End", "ui.viewport.end"),
            ("Tab", "ui.focus.next"),
            ("BackTab", "ui.focus.previous"),
            ("Enter", "ui.open"),
            ("q", "ui.quit"),
            ("?", "ui.help"),
            ("F2", "ui.view.toggle_density"),
            ("F5", "ui.refresh"),
        ] {
            map.builtin("global", &[key], id)?;
        }
        if profile == KeymapProfile::Vim {
            for (key, id) in [
                ("h", "ui.move.left"),
                ("j", "ui.move.down"),
                ("k", "ui.move.up"),
                ("l", "ui.move.right"),
                ("G", "ui.viewport.end"),
                ("Ctrl+d", "ui.viewport.half_down"),
                ("Ctrl+u", "ui.viewport.half_up"),
                ("/", "ui.search"),
                ("i", "ui.mode.insert"),
            ] {
                map.builtin("global", &[key], id)?;
            }
            map.builtin("global", &["g", "g"], "ui.viewport.home")?;
        }
        // Overlay has its own read-only scrolling; no global fallback is used.
        for context in ["overlay", "help"] {
            for (key, id) in [
                ("Up", "ui.viewport.line_up"),
                ("Down", "ui.viewport.line_down"),
                ("PageUp", "ui.viewport.page_up"),
                ("PageDown", "ui.viewport.page_down"),
                ("Home", "ui.viewport.home"),
                ("End", "ui.viewport.end"),
            ] {
                map.builtin(context, &[key], id)?;
            }
        }
        for binding in product {
            map.bindings.retain(|b| {
                !(b.origin == BindingOrigin::BuiltIn
                    && b.context == binding.context
                    && b.mode == binding.mode
                    && b.keys == binding.keys)
            });
            map.bindings.push(binding);
        }
        map.validate()?;
        Ok(map)
    }

    /// Explicit opt-in for a product-owned confirmation/preview context.
    /// Returns requests only; permission, ownership and CAS remain product-owned.
    /// Defaults keep all modal write requests suppressed. Global/editor fallback
    /// is never admitted, and the action must explicitly declare this context.
    pub fn with_modal_write_requests(mut self, contexts: &[&str]) -> Result<Self, KeymapError> {
        for context in contexts {
            if matches!(
                *context,
                "global" | "editor" | "help" | "overlay" | "confirm" | ""
            ) {
                return Err(KeymapError(
                    "modal requests require an explicit product context".into(),
                ));
            }
            self.modal_write_requests.insert((*context).into());
        }
        Ok(self)
    }

    fn modal_request_allowed(&self, owner: &str, descriptor: &ActionDescriptor) -> bool {
        descriptor.effect == ActionEffect::ProductWriteRequest
            && self.modal_write_requests.contains(owner)
            && descriptor.contexts.iter().any(|context| context == owner)
    }

    fn builtin(&mut self, context: &str, keys: &[&str], action: &str) -> Result<(), KeymapError> {
        let mut binding = Binding::new(context, keys, action)?;
        binding.origin = BindingOrigin::BuiltIn;
        self.bindings.push(binding);
        Ok(())
    }

    pub fn action_registry(&self) -> &ActionRegistry {
        &self.actions
    }
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn validate(&self) -> Result<(), KeymapError> {
        let expected = match self.navigation {
            NavigationPolicy::Direct => Action::MoveScreen(-1),
            NavigationPolicy::Pane => Action::Move(Direction::Up),
        };
        if self
            .actions
            .get("ui.move.up")
            .is_none_or(|descriptor| descriptor.action != expected)
        {
            return Err(KeymapError(
                "action registry/navigation policy mismatch".into(),
            ));
        }
        for (i, binding) in self.bindings.iter().enumerate() {
            let descriptor = self
                .actions
                .get(&binding.action_id)
                .ok_or_else(|| KeymapError(format!("unknown action: {}", binding.action_id)))?;
            if binding.context.is_empty() || binding.keys.is_empty() || binding.keys.len() > 4 {
                return Err(KeymapError(
                    "context required; sequence length must be 1..=4".into(),
                ));
            }
            if !descriptor.contexts.is_empty() && !descriptor.contexts.contains(&binding.context) {
                return Err(KeymapError(format!(
                    "action {} not admitted in {}",
                    descriptor.id, binding.context
                )));
            }
            if binding.mode == InputMode::Insert {
                return Err(KeymapError(
                    "Insert text ownership is protected; bind Normal actions instead".into(),
                ));
            }
            if binding.keys.iter().any(|key| protected(*key)) {
                return Err(KeymapError(
                    "Esc and Ctrl+C recovery cannot be overridden or sequenced".into(),
                ));
            }
            if self.profile == KeymapProfile::Vim
                && matches!(binding.keys[0].code, KeyCode::Char('0'..='9'))
                && binding.keys[0].modifiers == Modifiers::NONE
            {
                return Err(KeymapError(
                    "Vim digits are reserved for navigation counts".into(),
                ));
            }
            if matches!(
                binding.context.as_str(),
                "confirm" | "overlay" | "help" | "editor"
            ) && descriptor.effect != ActionEffect::View
            {
                return Err(KeymapError(
                    "capture context cannot acquire product or collector authority".into(),
                ));
            }
            for other in &self.bindings[..i] {
                if binding.context == other.context
                    && binding.mode == other.mode
                    && (binding.keys.starts_with(&other.keys)
                        || other.keys.starts_with(&binding.keys))
                {
                    return Err(KeymapError(format!(
                        "duplicate or ambiguous prefix in {}: {}",
                        binding.context,
                        format_keys(&binding.keys)
                    )));
                }
            }
        }
        Ok(())
    }

    /// Parses a supplied string. Paths, filesystem persistence and fallback selection
    /// are product-owned. Failure never mutates this accepted map.
    pub fn overlay_toml(&self, text: &str) -> Result<Self, KeymapError> {
        let config: Config = toml::from_str(text).map_err(|e| KeymapError(e.to_string()))?;
        if config.schema_version != 1 {
            return Err(KeymapError("unsupported schema_version".into()));
        }
        if config.preset.is_some_and(|p| p != self.profile)
            || config.navigation.is_some_and(|n| n != self.navigation)
        {
            return Err(KeymapError(
                "select profile/navigation when constructing the base map".into(),
            ));
        }
        let mut result = self.clone();
        if let Some(ms) = config.sequence_timeout_ms {
            if !(50..=5000).contains(&ms) {
                return Err(KeymapError("sequence_timeout_ms must be 50..=5000".into()));
            }
            result.sequence_timeout = Duration::from_millis(ms);
        }
        let mut seen = BTreeSet::new();
        for unbind in config.unbind {
            let keys = unbind.chords()?;
            if keys.iter().any(|k| protected(*k)) {
                return Err(KeymapError("cannot unbind recovery".into()));
            }
            result.bindings.retain(|b| {
                !(b.context == unbind.context && b.mode == unbind.mode && b.keys == keys)
            });
        }
        for binding in config.bindings {
            let keys = binding
                .keys
                .iter()
                .map(|s| parse_chord(s))
                .collect::<Result<Vec<_>, _>>()?;
            let identity = (binding.context.clone(), binding.mode, keys.clone());
            if !seen.insert(identity) {
                return Err(KeymapError("duplicate user override".into()));
            }
            result.bindings.retain(|b| {
                !(b.context == binding.context && b.mode == binding.mode && b.keys == keys)
            });
            result.bindings.push(Binding {
                context: binding.context,
                mode: binding.mode,
                keys,
                action_id: binding.action,
                origin: BindingOrigin::User,
            });
        }
        result.validate()?;
        Ok(result)
    }

    /// Reverting is simply keeping/reconstructing the accepted base map, no file write.
    pub fn diagnostics(
        &self,
        context: &ResolveContext,
        required_actions: &[&str],
    ) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        let mut prior: Vec<&Binding> = vec![];
        for owner in context.owners() {
            for binding in self
                .bindings
                .iter()
                .filter(|b| &b.context == owner && b.mode == context.mode)
            {
                if let Some(hidden) = prior
                    .iter()
                    .find(|b| b.keys.first() == binding.keys.first())
                {
                    diagnostics.push(Diagnostic::Shadow {
                        context: hidden.context.clone(),
                        keys: format_keys(&binding.keys),
                        hidden_context: binding.context.clone(),
                    });
                }
            }
            prior.extend(
                self.bindings
                    .iter()
                    .filter(|b| &b.context == owner && b.mode == context.mode),
            );
        }
        let effective = self.effective_hints(context);
        for action in required_actions {
            if !effective.iter().any(|h| h.action_id == *action) {
                diagnostics.push(Diagnostic::UnreachableAction((*action).into()));
            }
        }
        diagnostics
    }

    pub fn effective_hints(&self, context: &ResolveContext) -> Vec<EffectiveHint> {
        let mut result = vec![EffectiveHint {
            keys: "Esc".into(),
            label: "Back / cancel".into(),
            action_id: "ui.cancel".into(),
            origin: BindingOrigin::BuiltIn,
        }];
        // Emergency quit is a request; the product's dirty gate remains mandatory.
        result.push(EffectiveHint {
            keys: "Ctrl+c".into(),
            label: "Request quit".into(),
            action_id: "ui.quit".into(),
            origin: BindingOrigin::BuiltIn,
        });
        if context.mode == InputMode::Insert || context.capture == Capture::Editor {
            if context.mode == InputMode::Normal
                && context.capture == Capture::Editor
                && context.editable
                && self.profile == KeymapProfile::Vim
            {
                result.push(EffectiveHint {
                    keys: "i".into(),
                    label: "Insert".into(),
                    action_id: "ui.mode.insert".into(),
                    origin: BindingOrigin::BuiltIn,
                });
            }
            return result;
        }
        let mut claimed: Vec<Vec<KeyChord>> = vec![];
        for owner in context.owners() {
            for binding in self
                .bindings
                .iter()
                .filter(|b| &b.context == owner && b.mode == context.mode)
            {
                if claimed
                    .iter()
                    .any(|keys| keys.first() == binding.keys.first())
                {
                    continue;
                }
                let descriptor = &self.actions.actions[&binding.action_id];
                if descriptor.action == Action::EnterInsert && !context.editable {
                    continue;
                }
                if matches!(context.capture, Capture::Overlay | Capture::Confirm)
                    && descriptor.effect != ActionEffect::View
                    && !self.modal_request_allowed(owner, descriptor)
                {
                    continue;
                }
                result.push(EffectiveHint {
                    keys: format_keys(&binding.keys),
                    label: descriptor.label.clone(),
                    action_id: binding.action_id.clone(),
                    origin: binding.origin,
                });
            }
            claimed.extend(
                self.bindings
                    .iter()
                    .filter(|b| &b.context == owner && b.mode == context.mode)
                    .map(|b| b.keys.clone()),
            );
        }
        result
    }

    /// Compact footer projection from the same effective registry, with density,
    /// Help and recovery first so essential keys remain visible in narrow frames.
    pub fn footer_hints(&self, context: &ResolveContext) -> Vec<EffectiveHint> {
        let mut hints = self.effective_hints(context);
        let priority = [
            "ui.view.toggle_density",
            "ui.help",
            "ui.cancel",
            "ui.quit",
            "ui.focus.next",
            "ui.focus.previous",
            "ui.viewport.page_up",
            "ui.viewport.page_down",
        ];
        hints.sort_by_key(|hint| {
            priority
                .iter()
                .position(|id| *id == hint.action_id)
                .unwrap_or(priority.len())
        });
        let mut ids = BTreeSet::new();
        hints.retain(|hint| ids.insert(hint.action_id.clone()));
        hints
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema_version: u32,
    preset: Option<KeymapProfile>,
    navigation: Option<NavigationPolicy>,
    sequence_timeout_ms: Option<u64>,
    #[serde(default)]
    bindings: Vec<ConfigBinding>,
    #[serde(default)]
    unbind: Vec<ConfigKey>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigKey {
    context: String,
    #[serde(default)]
    mode: InputMode,
    keys: Vec<String>,
}
impl ConfigKey {
    fn chords(&self) -> Result<Vec<KeyChord>, KeymapError> {
        if self.context.is_empty()
            || !(1..=4).contains(&self.keys.len())
            || self.mode == InputMode::Insert
        {
            return Err(KeymapError(
                "unbind requires Normal context and 1..=4 keys".into(),
            ));
        }
        self.keys.iter().map(|s| parse_chord(s)).collect()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigBinding {
    context: String,
    #[serde(default)]
    mode: InputMode,
    keys: Vec<String>,
    action: String,
}

fn protected(key: KeyChord) -> bool {
    key.code == KeyCode::Escape
        || (matches!(key.code, KeyCode::Char('c' | 'C')) && key.modifiers == Modifiers::CONTROL)
}

pub fn parse_chord(text: &str) -> Result<KeyChord, KeymapError> {
    let mut modifiers = Modifiers::NONE;
    let mut parts = text.split('+').peekable();
    let mut code = None;
    while let Some(part) = parts.next() {
        if parts.peek().is_some() {
            modifiers = modifiers.union(match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => Modifiers::CONTROL,
                "shift" => Modifiers::SHIFT,
                "alt" => Modifiers::ALT,
                "super" => Modifiers::SUPER,
                "hyper" => Modifiers::HYPER,
                "meta" => Modifiers::META,
                _ => return Err(KeymapError(format!("unknown modifier: {part}"))),
            });
        } else {
            code = Some(match part.to_ascii_lowercase().as_str() {
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "left" => KeyCode::Left,
                "right" => KeyCode::Right,
                "enter" => KeyCode::Enter,
                "esc" | "escape" => KeyCode::Escape,
                "tab" => KeyCode::Tab,
                "backtab" => KeyCode::BackTab,
                "home" => KeyCode::Home,
                "end" => KeyCode::End,
                "pageup" | "pgup" => KeyCode::PageUp,
                "pagedown" | "pgdn" => KeyCode::PageDown,
                "backspace" => KeyCode::Backspace,
                "delete" => KeyCode::Delete,
                "insert" => KeyCode::Insert,
                "space" => KeyCode::Char(' '),
                "plus" => KeyCode::Char('+'),
                s if s.starts_with('f')
                    && s.len() > 1
                    && s[1..].parse::<u8>().is_ok_and(|v| (1..=24).contains(&v)) =>
                {
                    KeyCode::Function(s[1..].parse().unwrap())
                }
                _ if part.chars().count() == 1 => KeyCode::Char(part.chars().next().unwrap()),
                _ => return Err(KeymapError(format!("unknown key: {part}"))),
            });
        }
    }
    Ok(KeyChord::new(
        code.ok_or_else(|| KeymapError("empty chord".into()))?,
        modifiers,
    )
    .normalized())
}

pub fn format_keys(keys: &[KeyChord]) -> String {
    keys.iter()
        .map(|key| {
            let mut value = String::new();
            for (modifier, label) in [
                (Modifiers::CONTROL, "Ctrl+"),
                (Modifiers::ALT, "Alt+"),
                (Modifiers::SUPER, "Super+"),
                (Modifiers::HYPER, "Hyper+"),
                (Modifiers::META, "Meta+"),
                (Modifiers::SHIFT, "Shift+"),
            ] {
                if key.modifiers.contains(modifier) {
                    value.push_str(label);
                }
            }
            value.push_str(&match key.code {
                KeyCode::Char(' ') => "Space".into(),
                KeyCode::Char('+') => "Plus".into(),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Function(n) => format!("F{n}"),
                KeyCode::Escape => "Esc".into(),
                other => format!("{other:?}"),
            });
            value
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub trait MonotonicClock {
    fn now(&self) -> Duration;
}
pub struct SystemClock(Instant);
impl Default for SystemClock {
    fn default() -> Self {
        Self(Instant::now())
    }
}
impl MonotonicClock for SystemClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}

/// Bounded nonblocking sequence reducer. Tick via expire() even without input.
pub struct Resolver<C> {
    clock: C,
    pending: Vec<KeyChord>,
    pending_owner: Option<String>,
    count: u32,
    deadline: Option<Duration>,
    context: Option<ResolveContext>,
}

impl<C: MonotonicClock> Resolver<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            pending: vec![],
            pending_owner: None,
            count: 0,
            deadline: None,
            context: None,
        }
    }
    pub fn cancel(&mut self) {
        self.pending.clear();
        self.pending_owner = None;
        self.count = 0;
        self.deadline = None;
    }
    pub fn expire(&mut self) -> bool {
        if self
            .deadline
            .is_some_and(|deadline| self.clock.now() >= deadline)
        {
            self.cancel();
            true
        } else {
            false
        }
    }
    pub fn resolve(
        &mut self,
        map: &KeymapRegistry,
        context: &ResolveContext,
        event: &InputEvent,
    ) -> Resolution {
        if self.context.as_ref() != Some(context) {
            self.cancel();
            self.context = Some(context.clone());
        }
        self.expire();
        let InputEvent::Key(event) = event else {
            self.cancel();
            return match event {
                InputEvent::Paste(text)
                    if context.capture == Capture::Editor
                        || (context.mode == InputMode::Insert && context.editable) =>
                {
                    Resolution::Text(text.clone())
                }
                _ if context.capture != Capture::Normal => Resolution::Consumed,
                _ => Resolution::NoBinding,
            };
        };
        if !admits_input(event.kind) {
            return Resolution::Consumed;
        }
        let chord = event.chord.normalized();
        if protected(chord) {
            self.cancel();
            return Resolution::Intent(ActionIntent {
                action: if chord.code == KeyCode::Escape {
                    Action::Cancel
                } else {
                    Action::RequestQuit
                },
                effect: ActionEffect::View,
                count: 1,
            });
        }
        if context.mode == InputMode::Insert || context.capture == Capture::Editor {
            self.cancel();
            if context.mode == InputMode::Normal
                && context.capture == Capture::Editor
                && context.editable
                && map.profile == KeymapProfile::Vim
                && chord == KeyChord::new(KeyCode::Char('i'), Modifiers::NONE)
            {
                return Resolution::Intent(ActionIntent {
                    action: Action::EnterInsert,
                    effect: ActionEffect::View,
                    count: 1,
                });
            }
            return if context.editable && event.chord.modifiers.is_text() {
                match chord.code {
                    KeyCode::Char(c) => Resolution::Text(c.to_string()),
                    _ => Resolution::EditorKey(*event),
                }
            } else if context.editable {
                Resolution::EditorKey(*event)
            } else {
                Resolution::Consumed
            };
        }
        if map.profile == KeymapProfile::Vim
            && context.capture == Capture::Normal
            && self.pending.is_empty()
            && chord.modifiers == Modifiers::NONE
            && let KeyCode::Char(c @ '0'..='9') = chord.code
        {
            self.count = self
                .count
                .saturating_mul(10)
                .saturating_add(c as u32 - '0' as u32)
                .min(9999);
            self.deadline = Some(self.clock.now().saturating_add(map.sequence_timeout));
            return Resolution::Pending;
        }
        self.pending.push(chord);
        for owner in context.owners() {
            if self
                .pending_owner
                .as_ref()
                .is_some_and(|pending| pending != owner)
            {
                continue;
            }
            let candidates: Vec<_> = map
                .bindings
                .iter()
                .filter(|b| {
                    &b.context == owner
                        && b.mode == context.mode
                        && b.keys.starts_with(&self.pending)
                })
                .collect();
            if candidates.is_empty() {
                continue;
            }
            if let Some(binding) = candidates.iter().find(|b| b.keys == self.pending) {
                let descriptor = map.actions.get(&binding.action_id).unwrap();
                let action = descriptor.action.clone();
                let count = if action.accepts_count() {
                    self.count.max(1)
                } else {
                    1
                };
                self.cancel();
                if (action == Action::EnterInsert && !context.editable)
                    || (matches!(context.capture, Capture::Overlay | Capture::Confirm)
                        && descriptor.effect != ActionEffect::View
                        && !map.modal_request_allowed(owner, descriptor))
                {
                    return Resolution::Consumed;
                }
                return Resolution::Intent(ActionIntent {
                    action,
                    effect: descriptor.effect,
                    count,
                });
            }
            self.deadline = Some(self.clock.now().saturating_add(map.sequence_timeout));
            self.pending_owner = Some(owner.clone());
            return Resolution::Pending;
        }
        self.cancel();
        if context.capture != Capture::Normal {
            Resolution::Consumed
        } else {
            Resolution::NoBinding
        }
    }
}
