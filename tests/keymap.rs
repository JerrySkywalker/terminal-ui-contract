use std::{cell::Cell, rc::Rc, time::Duration};
use terminal_ui_contract::{input::*, keymap::*};

#[derive(Clone)]
struct Clock(Rc<Cell<Duration>>);
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        self.0.get()
    }
}
fn setup(profile: KeymapProfile) -> (KeymapRegistry, Resolver<Clock>, Clock) {
    let clock = Clock(Rc::new(Cell::new(Duration::ZERO)));
    let map = KeymapRegistry::new(
        profile,
        NavigationPolicy::Pane,
        ActionRegistry::standard(NavigationPolicy::Pane),
        vec![],
    )
    .unwrap();
    (map, Resolver::new(clock.clone()), clock)
}
fn key(value: &str) -> InputEvent {
    InputEvent::Key(KeyEvent {
        chord: parse_chord(value).unwrap(),
        kind: InputKind::Press,
    })
}

#[test]
fn modal_write_requests_require_explicit_product_context_opt_in() {
    let mut actions = ActionRegistry::standard(NavigationPolicy::Pane);
    actions
        .register_product(ActionDescriptor {
            id: "consumer.apply".into(),
            label: "Apply request".into(),
            action: Action::Product("consumer.apply".into()),
            effect: ActionEffect::ProductWriteRequest,
            contexts: vec!["preview".into()],
        })
        .unwrap();
    let map = KeymapRegistry::new(
        KeymapProfile::Standard,
        NavigationPolicy::Pane,
        actions,
        vec![Binding::new("preview", &["a"], "consumer.apply").unwrap()],
    )
    .unwrap();
    let mut context = ResolveContext::normal(&["preview", "settings"]);
    context.capture = Capture::Overlay;
    let mut resolver = Resolver::new(SystemClock::default());
    assert_eq!(
        resolver.resolve(&map, &context, &key("a")),
        Resolution::Consumed
    );
    let admitted = map.clone().with_modal_write_requests(&["preview"]).unwrap();
    assert!(
        !map.effective_hints(&context)
            .iter()
            .any(|hint| hint.action_id == "consumer.apply")
    );
    assert!(
        admitted
            .effective_hints(&context)
            .iter()
            .any(|hint| hint.action_id == "consumer.apply" && hint.keys == "a")
    );
    assert_eq!(
        resolver.resolve(&admitted, &context, &key("a")),
        Resolution::Intent(ActionIntent {
            action: Action::Product("consumer.apply".into()),
            effect: ActionEffect::ProductWriteRequest,
            count: 1
        })
    );
    context.contexts[0] = "help".into();
    assert_eq!(
        resolver.resolve(&admitted, &context, &key("a")),
        Resolution::Consumed
    );
    assert!(map.with_modal_write_requests(&["global"]).is_err());
}
fn intent(action: Action, count: u32) -> Resolution {
    Resolution::Intent(ActionIntent {
        action,
        effect: ActionEffect::View,
        count,
    })
}

#[test]
fn vim_sequences_counts_deadline_and_focus_cancel() {
    let (map, mut resolver, clock) = setup(KeymapProfile::Vim);
    let mut context = ResolveContext::normal(&["details"]);
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        Resolution::Pending
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        intent(Action::Home, 1)
    );
    for digit in ["1", "0"] {
        assert_eq!(
            resolver.resolve(&map, &context, &key(digit)),
            Resolution::Pending
        );
    }
    assert_eq!(
        resolver.resolve(&map, &context, &key("k")),
        intent(Action::Move(Direction::Up), 10)
    );
    for _ in 0..20 {
        resolver.resolve(&map, &context, &key("9"));
    }
    assert_eq!(
        resolver.resolve(&map, &context, &key("j")),
        intent(Action::Move(Direction::Down), 9999)
    );
    resolver.resolve(&map, &context, &key("5"));
    assert_eq!(
        resolver.resolve(&map, &context, &key("Ctrl+d")),
        intent(Action::HalfPage(1), 5)
    );
    resolver.resolve(&map, &context, &key("g"));
    clock.0.set(Duration::from_millis(600));
    assert!(resolver.expire());
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        Resolution::Pending
    );
    context.focus_token += 1;
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        Resolution::Pending
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("Esc")),
        intent(Action::Cancel, 1)
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("G")),
        intent(Action::End, 1)
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("i")),
        Resolution::Consumed
    );
    context.editable = true;
    assert_eq!(
        resolver.resolve(&map, &context, &key("i")),
        intent(Action::EnterInsert, 1)
    );
}

#[test]
fn insert_editor_and_modal_do_not_leak() {
    let (map, mut resolver, _) = setup(KeymapProfile::Vim);
    let mut context = ResolveContext::normal(&["editor", "details"]);
    context.mode = InputMode::Insert;
    context.editable = true;
    for text in ["q", "r", "m", "1", "2", "3", "诊"] {
        assert_eq!(
            resolver.resolve(&map, &context, &key(text)),
            Resolution::Text(text.into())
        );
    }
    assert_eq!(
        resolver.resolve(&map, &context, &InputEvent::Paste("中文 q123".into())),
        Resolution::Text("中文 q123".into())
    );
    assert!(matches!(
        resolver.resolve(&map, &context, &key("Backspace")),
        Resolution::EditorKey(_)
    ));
    assert_eq!(
        resolver.resolve(&map, &context, &key("Esc")),
        intent(Action::Cancel, 1)
    );
    for capture in [Capture::Overlay, Capture::Confirm] {
        context.mode = InputMode::Normal;
        context.capture = capture;
        context.contexts = vec!["confirm".into(), "global".into()];
        for shortcut in ["q", "r", "m", "F2", "F5", "3"] {
            assert_eq!(
                resolver.resolve(&map, &context, &key(shortcut)),
                Resolution::Consumed
            );
        }
        assert_eq!(map.effective_hints(&context).len(), 2);
    }
    context.capture = Capture::Editor;
    assert_eq!(
        resolver.resolve(&map, &context, &key("q")),
        Resolution::Text("q".into())
    );
}

#[test]
fn modifiers_and_press_only_are_preserved() {
    let (map, mut resolver, _) = setup(KeymapProfile::Standard);
    let context = ResolveContext::normal(&[]);
    assert_eq!(
        resolver.resolve(&map, &context, &key("Ctrl+3")),
        Resolution::NoBinding
    );
    assert_ne!(parse_chord("Ctrl+3").unwrap(), parse_chord("3").unwrap());
    assert_ne!(parse_chord("Shift+3").unwrap(), parse_chord("3").unwrap());
    assert_eq!(parse_chord("Shift+g").unwrap(), parse_chord("G").unwrap());
    assert_eq!(
        parse_chord("Shift+Tab").unwrap(),
        parse_chord("BackTab").unwrap()
    );
    for kind in [InputKind::Repeat, InputKind::Release] {
        assert_eq!(
            resolver.resolve(
                &map,
                &context,
                &InputEvent::Key(KeyEvent {
                    chord: parse_chord("q").unwrap(),
                    kind
                })
            ),
            Resolution::Consumed
        );
    }
}

#[test]
fn overlay_changes_dispatch_and_hints_without_mutating_base() {
    let (base, mut resolver, _) = setup(KeymapProfile::Standard);
    let map = base
        .overlay_toml(
            r#"
schema_version = 1
[[unbind]]
context = "global"
keys = ["F2"]
[[bindings]]
context = "global"
keys = ["F3"]
action = "ui.view.toggle_density"
"#,
        )
        .unwrap();
    let context = ResolveContext::normal(&[]);
    assert_eq!(
        resolver.resolve(&map, &context, &key("F2")),
        Resolution::NoBinding
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("F3")),
        intent(Action::ToggleDensity, 1)
    );
    let hints = map.effective_hints(&context);
    assert!(
        hints
            .iter()
            .any(|h| h.keys == "F3" && h.action_id == "ui.view.toggle_density")
    );
    assert!(!hints.iter().any(|h| h.keys == "F2"));
    assert!(
        base.effective_hints(&context)
            .iter()
            .any(|h| h.keys == "F2")
    );
}

#[test]
fn invalid_maps_reject_unknown_shell_recovery_duplicates_and_prefixes() {
    let (base, _, _) = setup(KeymapProfile::Standard);
    for body in [
        "keys=[\"x\"]\naction=\"shell.exec\"",
        "keys=[\"Esc\"]\naction=\"ui.refresh\"",
        "keys=[\"Ctrl+c\"]\naction=\"ui.refresh\"",
        "keys=[\"x\"]\naction=\"ui.refresh\"\ncommand=\"echo unsafe\"",
        "keys=[\"x\"]\naction=\"ui.refresh\"\n[[bindings]]\ncontext=\"global\"\nkeys=[\"x\"]\naction=\"ui.help\"",
        "keys=[\"q\",\"x\"]\naction=\"ui.refresh\"",
        "mode=\"insert\"\nkeys=[\"x\"]\naction=\"ui.quit\"",
    ] {
        assert!(
            base.overlay_toml(&format!(
                "schema_version=1\n[[bindings]]\ncontext=\"global\"\n{body}"
            ))
            .is_err(),
            "{body}"
        );
    }
    assert!(base.overlay_toml("schema_version=2").is_err());
    assert!(
        base.overlay_toml("schema_version=1\nexecute='bad'")
            .is_err()
    );
}

#[test]
fn two_consumers_contexts_write_requests_and_diagnostics() {
    for (namespace, contexts, keys) in [
        ("tabbeacon", ["diagnostics", "integrations"], ["m", "3"]),
        ("hookstat", ["window", "list"], ["3", "w"]),
    ] {
        let mut actions = ActionRegistry::standard(NavigationPolicy::Pane);
        let id = format!("{namespace}.query");
        actions
            .register_product(ActionDescriptor {
                id: id.clone(),
                label: "Query".into(),
                action: Action::Product(id.clone()),
                effect: ActionEffect::ReadOnlyQuery,
                contexts: vec![contexts[0].into()],
            })
            .unwrap();
        let map = KeymapRegistry::new(
            KeymapProfile::Standard,
            NavigationPolicy::Pane,
            actions,
            vec![Binding::new(contexts[0], &[keys[0]], &id).unwrap()],
        )
        .unwrap();
        let mut resolver = Resolver::new(SystemClock::default());
        let context = ResolveContext::normal(&[contexts[0]]);
        assert!(matches!(
            resolver.resolve(&map, &context, &key(keys[0])),
            Resolution::Intent(ActionIntent {
                effect: ActionEffect::ReadOnlyQuery,
                ..
            })
        ));
        assert_eq!(
            resolver.resolve(&map, &context, &key("Ctrl+3")),
            Resolution::NoBinding
        );
        let other = ResolveContext::normal(&[contexts[1]]);
        assert!(
            !map.effective_hints(&other)
                .iter()
                .any(|h| h.action_id == id)
        );
        assert_eq!(
            resolver.resolve(&map, &other, &key(keys[0])),
            Resolution::NoBinding
        );
        assert_eq!(
            map.diagnostics(&other, &[&id]),
            vec![Diagnostic::UnreachableAction(id)]
        );
    }
    let mut actions = ActionRegistry::standard(NavigationPolicy::Pane);
    let write = "synthetic.apply";
    actions
        .register_product(ActionDescriptor {
            id: write.into(),
            label: "Apply request".into(),
            action: Action::Product(write.into()),
            effect: ActionEffect::ProductWriteRequest,
            contexts: vec!["settings".into()],
        })
        .unwrap();
    let map = KeymapRegistry::new(
        KeymapProfile::Standard,
        NavigationPolicy::Pane,
        actions,
        vec![
            Binding::new("settings", &["a"], write).unwrap(),
            Binding::new("settings", &["q"], "ui.help").unwrap(),
        ],
    )
    .unwrap();
    assert!(
        !map.diagnostics(&ResolveContext::normal(&["settings"]), &[])
            .is_empty()
    );
    assert!(map.overlay_toml("schema_version=1\n[[bindings]]\ncontext='confirm'\nkeys=['a']\naction='synthetic.apply'").is_err());
}

#[test]
fn legacy_direct_navigation_remains_available() {
    let map = KeymapRegistry::new(
        KeymapProfile::Standard,
        NavigationPolicy::Direct,
        ActionRegistry::standard(NavigationPolicy::Direct),
        vec![],
    )
    .unwrap();
    let mut resolver = Resolver::new(SystemClock::default());
    assert_eq!(
        resolver.resolve(&map, &ResolveContext::normal(&[]), &key("Down")),
        intent(Action::MoveScreen(1), 1)
    );
    let mut nav = terminal_ui_contract::navigation::TopLevelNavigation::new(0);
    assert!(nav.move_by(&[0, 1, 2], 1));
    assert_eq!(nav.current(), 1);
    assert!(!admits_input(InputKind::Repeat));
    assert!(
        !terminal_ui_contract::footer::FooterState::Detail
            .hints()
            .is_empty()
    );
}

#[test]
fn initial_toml_profile_and_cross_context_prefix_ownership() {
    let map = KeymapRegistry::from_toml(
        "schema_version=1\npreset='vim'\nnavigation='direct'",
        vec![],
        vec![],
    )
    .unwrap();
    let mut resolver = Resolver::new(SystemClock::default());
    assert_eq!(
        resolver.resolve(&map, &ResolveContext::normal(&[]), &key("j")),
        intent(Action::MoveScreen(1), 1)
    );
    let map = map
        .overlay_toml(
            "schema_version=1\n[[bindings]]\ncontext='details'\nkeys=['g','x']\naction='ui.help'",
        )
        .unwrap();
    let context = ResolveContext::normal(&["details"]);
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        Resolution::Pending
    );
    assert_eq!(
        resolver.resolve(&map, &context, &key("g")),
        Resolution::NoBinding
    );
    assert!(
        !map.effective_hints(&context)
            .iter()
            .any(|h| h.keys == "g g")
    );
    assert!(
        map.diagnostics(&context, &["ui.viewport.home"])
            .iter()
            .any(|d| matches!(d, Diagnostic::Shadow { .. }))
    );
}
