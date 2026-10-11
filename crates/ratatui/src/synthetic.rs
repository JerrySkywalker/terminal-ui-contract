//! Runnable dual-consumer fixtures, no product code, user files or provider execution.
use crate::{components::*, geometry::*, shell::*};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Clear, Widget},
};
use std::{collections::BTreeMap, sync::Arc, time::SystemTime};
use terminal_ui_contract::{
    document::*, focus::*, input::*, keymap::*, locale::Locale, navigation::TopLevelNavigation,
    theme::NeutralTheme, viewport::*,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Consumer {
    TabBeacon,
    HookStat,
}
impl Consumer {
    pub fn name(self) -> &'static str {
        match self {
            Self::TabBeacon => "TabBeacon synthetic",
            Self::HookStat => "HookStat synthetic",
        }
    }
    pub fn screens(self) -> &'static [&'static str] {
        match self {
            Self::TabBeacon => &[
                "Overview",
                "Appearance",
                "Workspace",
                "Sessions",
                "Integrations",
                "Hooks",
                "Diagnostics",
                "Interface",
                "Preview",
            ],
            Self::HookStat => &["Events", "Handlers", "Details", "Window", "List"],
        }
    }
}

/// One composite first-wave component gallery; all inputs are synthetic/cleared.
pub fn draw_component_gallery(
    consumer: Consumer,
    locale: Locale,
    theme: NeutralTheme,
    area: Rect,
    buffer: &mut Buffer,
) {
    let document = document(consumer, 0, 1);
    if area.width < 40 || area.height < 20 {
        StatusCard {
            fact: &document.facts()[0],
            theme,
        }
        .render(area, buffer);
        return;
    }
    let rows = ratatui::layout::Layout::vertical([ratatui::layout::Constraint::Ratio(1, 4); 4])
        .split(area);
    let panes = rows
        .iter()
        .map(|row| {
            ratatui::layout::Layout::horizontal([ratatui::layout::Constraint::Ratio(1, 2); 2])
                .split(*row)
        })
        .collect::<Vec<_>>();
    StatusCard {
        fact: &document.facts()[0],
        theme,
    }
    .render(panes[0][0], buffer);
    MetricCard {
        label: "Synthetic events",
        value: "42",
        unit: "events",
        theme,
    }
    .render(panes[0][1], buffer);
    AttentionPanel {
        facts: document.facts(),
        locale,
        theme,
    }
    .render(panes[1][0], buffer);
    FreshnessIndicator {
        fact: &document.facts()[1],
        locale,
        theme,
    }
    .render(panes[1][1], buffer);
    KeyValueList {
        title: "Key/value",
        values: &[("source", consumer.name()), ("scope", "synthetic only")],
        theme,
    }
    .render(panes[2][0], buffer);
    EvidenceList {
        fact: &document.facts()[2],
        theme,
    }
    .render(panes[2][1], buffer);
    DataTable {
        title: "Synthetic DataTable",
        headers: &["ID", "Verdict"],
        rows: &document
            .facts()
            .iter()
            .map(|fact| vec![fact.id.clone(), fact.verdict.marker().into()])
            .collect::<Vec<_>>(),
        theme,
    }
    .render(panes[3][0], buffer);
    MessagePanel {
        kind: MessagePanelKind::Conflict,
        detail: "Synthetic conflict retained; explicit decision required",
        locale,
        theme,
    }
    .render(panes[3][1], buffer);
}

pub fn document(consumer: Consumer, screen: usize, revision: u64) -> UiDocument {
    let source = consumer.name();
    let scope = consumer.screens()[screen];
    let facts = [
        (
            "upstream",
            Verdict::Blocked,
            "Upstream qualification",
            "上游资格阻塞",
        ),
        (
            "current",
            Verdict::Unproven,
            "Current delivery",
            "当前交付未证实",
        ),
        (
            "static",
            Verdict::StaticPass,
            "Static structure",
            "静态结构通过",
        ),
    ]
    .into_iter()
    .map(|(id, verdict, label, summary)| UiFact {
        id: format!("synthetic.{id}"),
        label: label.into(),
        verdict,
        severity: if verdict == Verdict::StaticPass {
            Severity::Information
        } else {
            Severity::Attention
        },
        source: source.into(),
        scope: scope.into(),
        sampled_at: SystemTime::UNIX_EPOCH,
        freshness: FactFreshness::Unknown,
        summary: PermittedText::from_product_redaction(summary),
        impact: PermittedText::from_product_redaction(
            "Synthetic evidence; no real provider or ledger qualification",
        ),
        evidence: vec![
            EvidenceField {
                label: "Synthetic count".into(),
                value: PermittedText::from_product_redaction("3"),
            },
            EvidenceField {
                label: "Evidence boundary".into(),
                value: PermittedText::from_product_redaction(
                    "static only; current delivery remains UNPROVEN",
                ),
            },
        ],
        next_steps: vec![SafeNextStep {
            label: "Inspect synthetic evidence".into(),
            intent: ActionIntent {
                action: Action::Help,
                effect: ActionEffect::View,
                count: 1,
            },
        }],
    })
    .collect();
    UiDocument::new(revision, SystemTime::UNIX_EPOCH, facts).expect("known synthetic facts")
}

/// Synthetic permission gate exercises intent/permission separation without any
/// real writer/config/child/terminal handle. A positive control can increment counters.
pub struct DemoApp {
    pub consumer: Consumer,
    pub navigation: TopLevelNavigation<usize>,
    pub focus: FocusState,
    pub mode: InputMode,
    pub view: ViewState,
    pub document: Arc<UiDocument>,
    pub keymap: KeymapRegistry,
    pub locale: Locale,
    pub theme: NeutralTheme,
    pub viewports: BTreeMap<FocusPath, BoundedViewport>,
    pub selection: StableSelection<String>,
    pub dirty: bool,
    pub conflict: bool,
    pub write_authorized: bool,
    pub write_confirmed: bool,
    pub writer_calls: usize,
    pub write_requests: usize,
    pub query_requests: usize,
    pub editor_text: String,
    pub editor_open: bool,
    resolver: Resolver<SystemClock>,
    generation: u64,
    class: LayoutClass,
    ensure_selection: bool,
}

impl DemoApp {
    pub fn new(
        consumer: Consumer,
        profile: KeymapProfile,
        navigation: NavigationPolicy,
        locale: Locale,
        theme: NeutralTheme,
    ) -> Result<Self, KeymapError> {
        let mut actions = ActionRegistry::standard(navigation);
        let mut bindings = vec![];
        let contexts = match consumer {
            Consumer::TabBeacon => vec!["diagnostics", "integrations"],
            Consumer::HookStat => vec!["window", "list"],
        };
        for digit in ["1", "2", "3"] {
            let id = format!("synthetic.query.{digit}");
            actions.register_product(ActionDescriptor {
                id: id.clone(),
                label: format!("Query {digit}"),
                action: Action::Product(id.clone()),
                effect: ActionEffect::ReadOnlyQuery,
                contexts: contexts.iter().map(|s| (*s).into()).collect(),
            })?;
            // Vim digits remain counts; explicit modified query keys avoid stealing counts.
            let key = if profile == KeymapProfile::Vim {
                format!("Alt+{digit}")
            } else {
                digit.into()
            };
            for context in &contexts {
                bindings.push(Binding::new(context, &[&key], &id)?);
            }
        }
        let mode_id = "synthetic.query.next_mode";
        actions.register_product(ActionDescriptor {
            id: mode_id.into(),
            label: "Query next mode".into(),
            action: Action::Product(mode_id.into()),
            effect: ActionEffect::ReadOnlyQuery,
            contexts: vec![contexts[0].into()],
        })?;
        bindings.push(Binding::new(contexts[0], &["m"], mode_id)?);
        actions.register_product(ActionDescriptor {
            id: "synthetic.apply".into(),
            label: "Apply request (gated)".into(),
            action: Action::Product("synthetic.apply".into()),
            effect: ActionEffect::ProductWriteRequest,
            contexts: vec!["settings".into()],
        })?;
        bindings.push(Binding::new("settings", &["a"], "synthetic.apply")?);
        let document = Arc::new(document(consumer, 0, 1));
        let mut selection = StableSelection::default();
        selection.reconcile(
            &document
                .facts()
                .iter()
                .map(|fact| fact.id.clone())
                .collect::<Vec<_>>(),
        );
        Ok(Self {
            consumer,
            navigation: TopLevelNavigation::new(0),
            focus: FocusState::default(),
            mode: InputMode::Normal,
            view: ViewState::default(),
            document,
            keymap: KeymapRegistry::new(profile, navigation, actions, bindings)?,
            locale,
            theme,
            viewports: BTreeMap::new(),
            selection,
            dirty: false,
            conflict: false,
            write_authorized: false,
            write_confirmed: false,
            writer_calls: 0,
            write_requests: 0,
            query_requests: 0,
            editor_text: String::new(),
            editor_open: false,
            resolver: Resolver::new(SystemClock::default()),
            generation: 0,
            class: LayoutClass::Full,
            ensure_selection: true,
        })
    }
    pub fn set_keymap(&mut self, map: KeymapRegistry) {
        self.resolver.cancel();
        self.keymap = map;
    }
    pub fn select_screen(&mut self, index: usize) {
        self.navigation
            .set_current(index.min(self.consumer.screens().len() - 1));
        self.ensure_selection = true;
        self.accept_document(Arc::new(document(
            self.consumer,
            self.navigation.current(),
            self.document.revision() + 1,
        )));
        self.resolver.cancel();
    }
    pub fn accept_document(&mut self, document: Arc<UiDocument>) {
        self.selection.reconcile(
            &document
                .facts()
                .iter()
                .map(|fact| fact.id.clone())
                .collect::<Vec<_>>(),
        );
        self.document = document; // Draft/conflict/permission/focus remain independent.
    }
    pub fn context(&self) -> ResolveContext {
        let screen = self.consumer.screens()[self.navigation.current()].to_ascii_lowercase();
        let owner = self
            .focus
            .current()
            .and_then(|p| p.parts().last())
            .cloned()
            .unwrap_or_else(|| "navigation".into());
        let mut context = ResolveContext::normal(&[&owner, &screen]);
        if matches!(screen.as_str(), "appearance" | "interface") {
            context.contexts.insert(2, "settings".into());
        }
        context.focus_token = self.focus.revision();
        context.mode = self.mode;
        context.editable = owner == "editor";
        if self.focus.modal_root().is_some() {
            context.capture = Capture::Overlay;
            context.mode = InputMode::Normal;
        } else if self.mode == InputMode::Insert || owner == "editor" {
            context.capture = Capture::Editor;
        }
        context
    }
    fn set_focus(&mut self, path: FocusPath) {
        let mut visible = self.focus.visible().to_vec();
        if !visible.contains(&path) {
            visible.push(path.clone());
        }
        self.focus.reconcile(visible);
        self.focus.focus(&path);
        self.resolver.cancel();
    }
    pub fn handle(&mut self, event: InputEvent, geometry: Option<&HitMap>) -> Resolution {
        if let InputEvent::Mouse(mouse) = event {
            if let Some(geometry) = geometry {
                match geometry.pointer(mouse, self.generation, &mut self.focus) {
                    PointerEffect::Focus(_) => self.resolver.cancel(),
                    PointerEffect::Scroll { path, lines } => {
                        self.viewports.entry(path).or_default().lines(lines)
                    }
                    _ => {}
                }
            }
            return Resolution::Consumed;
        }
        if matches!(event, InputEvent::Resize { .. }) {
            self.generation = self.generation.wrapping_add(1);
            self.resolver.cancel();
        }
        let resolution = self.resolver.resolve(&self.keymap, &self.context(), &event);
        match &resolution {
            Resolution::Text(text) => self.editor_text.push_str(text),
            Resolution::Intent(intent) => self.dispatch(intent),
            _ => {}
        }
        if let Resolution::Intent(intent) = &resolution
            && matches!(
                intent.action,
                Action::FocusNext
                    | Action::FocusPrevious
                    | Action::Open
                    | Action::Cancel
                    | Action::Help
                    | Action::ToggleDensity
                    | Action::Search
            )
        {
            self.generation = self.generation.wrapping_add(1);
        }
        resolution
    }
    fn dispatch(&mut self, intent: &ActionIntent) {
        let focused = self
            .focus
            .current()
            .cloned()
            .unwrap_or_else(|| FocusPath::new(&["navigation"]));
        let count = intent.count as isize;
        match &intent.action {
            Action::Move(Direction::Up | Direction::Down)
                if focused == FocusPath::new(&["navigation"]) =>
            {
                let delta = if intent.action == Action::Move(Direction::Up) {
                    -count
                } else {
                    count
                };
                self.move_screen(delta);
            }
            Action::Move(Direction::Up) => self.viewports.entry(focused).or_default().lines(-count),
            Action::Move(Direction::Down) => {
                self.viewports.entry(focused).or_default().lines(count)
            }
            Action::MoveScreen(delta) => self.move_screen(*delta as isize * count),
            Action::Page(delta) => self
                .viewports
                .entry(focused)
                .or_default()
                .page(*delta as isize * count),
            Action::HalfPage(delta) => self
                .viewports
                .entry(focused)
                .or_default()
                .half_page(*delta as isize * count),
            Action::Home => self.viewports.entry(focused).or_default().home(),
            Action::End => self.viewports.entry(focused).or_default().end(),
            Action::FocusNext | Action::FocusPrevious => {
                if self.class == LayoutClass::Compact && self.focus.modal_root().is_none() {
                    self.set_focus(FocusPath::new(&[
                        if focused == FocusPath::new(&["navigation"]) {
                            "details"
                        } else {
                            "navigation"
                        },
                    ]));
                } else {
                    self.focus.cycle(intent.action == Action::FocusPrevious);
                }
            }
            Action::Open if focused == FocusPath::new(&["navigation"]) => {
                self.set_focus(FocusPath::new(&["details"]))
            }
            Action::Open => {
                self.focus.enter_child();
            }
            Action::Cancel => {
                if self.mode == InputMode::Insert {
                    self.mode = InputMode::Normal;
                } else if self.focus.modal_root().is_some() {
                    self.focus.pop_modal();
                } else {
                    self.editor_open = false;
                    if self.focus.leave_child().is_none() {
                        self.set_focus(FocusPath::new(&["navigation"]));
                    }
                }
            }
            Action::Help => {
                self.focus.push_modal(FocusPath::new(&["help"]));
            }
            Action::ToggleDensity => self.view.toggle(),
            Action::Search => {
                self.editor_open = true;
                self.set_focus(FocusPath::new(&["details", "editor"]));
                self.mode = InputMode::Insert;
            }
            Action::EnterInsert if self.context().editable => self.mode = InputMode::Insert,
            Action::Refresh => self.query_requests += 1,
            Action::Product(_) if intent.effect == ActionEffect::ProductWriteRequest => {
                self.write_requests += 1;
                if self.write_authorized && self.write_confirmed {
                    self.writer_calls += 1;
                }
            }
            Action::Product(_) => self.query_requests += 1,
            _ => {} // RequestQuit remains a request; real exit and dirty confirmation are product-owned.
        }
    }
    fn move_screen(&mut self, delta: isize) {
        let screens = (0..self.consumer.screens().len()).collect::<Vec<_>>();
        if self.navigation.move_by(&screens, delta) {
            self.select_screen(self.navigation.current());
        }
    }
    pub fn draw(&mut self, area: Rect, buffer: &mut Buffer) -> HitMap {
        self.generation = self.generation.wrapping_add(1);
        let navigation = FocusPath::new(&["navigation"]);
        let details = FocusPath::new(&["details"]);
        let layout = ShellLayout::compute(
            area,
            ResponsivePolicy::default(),
            self.focus.current().is_none_or(|p| p == &navigation),
        );
        self.class = layout.class;
        let mut regions = vec![];
        let mut add = |path: FocusPath, rect: Rect, z: u16, scrollable: bool| {
            regions.push(HitRegion {
                path,
                rect,
                clip: area,
                z,
                enabled: true,
                focusable: true,
                scrollable,
            });
        };
        if let Some(rect) = layout.panes.master {
            add(navigation.clone(), rect, 0, true);
        }
        if let Some(rect) = layout.panes.detail {
            add(details.clone(), rect, 0, true);
            if self.editor_open {
                add(
                    FocusPath::new(&["details", "editor"]),
                    Rect::new(
                        rect.x,
                        rect.bottom().saturating_sub(4),
                        rect.width,
                        4.min(rect.height),
                    ),
                    1,
                    false,
                );
            }
        }
        let modal_rect =
            if self.focus.modal_root().is_some() && layout.class != LayoutClass::TooSmall {
                let width = area.width.saturating_sub(4);
                let height = area.height.saturating_sub(4);
                let rect = Rect::new(area.x + 2, area.y + 2, width, height);
                add(FocusPath::new(&["help"]), rect, 10, true);
                Some(rect)
            } else {
                None
            };
        let geometry = HitMap::new(self.generation, regions);
        geometry.reconcile_focus(&mut self.focus);
        if let Some(rect) = layout.panes.master {
            Sidebar {
                items: self.consumer.screens(),
                selected: self.navigation.current(),
                focused: self.focus.current() == Some(&navigation),
                theme: self.theme,
            }
            .render(
                rect,
                buffer,
                self.viewports.entry(navigation.clone()).or_default(),
                self.ensure_selection,
            );
            self.ensure_selection = false;
        }
        if let Some(mut rect) = layout.panes.detail {
            if self.editor_open {
                let editor = Rect::new(
                    rect.x,
                    rect.bottom().saturating_sub(4),
                    rect.width,
                    4.min(rect.height),
                );
                rect.height = rect.height.saturating_sub(editor.height);
                ratatui::widgets::Paragraph::new(self.editor_text.as_str())
                    .block(pane_block(
                        "Editor (synthetic text)",
                        self.focus.current() == Some(&FocusPath::new(&["details", "editor"])),
                        self.theme,
                    ))
                    .render(editor, buffer);
            }
            let text = self.document.project(self.view.density).lines().join("\n");
            ScrollPane {
                title: match self.view.density {
                    ViewDensity::Smart => "Smart",
                    ViewDensity::Raw => "Raw (permitted)",
                },
                text: &text,
                focused: self.focus.current() == Some(&details),
                theme: self.theme,
            }
            .render(rect, buffer, self.viewports.entry(details).or_default());
        }
        let context = self.context();
        let hints = self.keymap.footer_hints(&context);
        let focus = self
            .focus
            .current()
            .map_or_else(|| "none".into(), FocusPath::label);
        render_chrome(
            layout,
            area,
            buffer,
            ShellChrome {
                identity: self.consumer.name(),
                focus: &focus,
                mode: self.mode,
                locale: self.locale,
                hints: &hints,
            },
        );
        if let Some(rect) = modal_rect {
            Clear.render(rect, buffer);
            Help {
                registry: &self.keymap,
                context: &context,
                locale: self.locale,
                theme: self.theme,
            }
            .render(
                rect,
                buffer,
                self.viewports.entry(FocusPath::new(&["help"])).or_default(),
            );
        }
        geometry
    }
}
