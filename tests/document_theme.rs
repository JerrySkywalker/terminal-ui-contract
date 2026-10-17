use std::time::SystemTime;
use terminal_ui_contract::{chrome::*, document::*, keymap::*, locale::*, theme::*};
fn fact(verdict: Verdict, id: &str) -> UiFact {
    UiFact {
        id: id.into(),
        label: "Synthetic".into(),
        verdict,
        severity: Severity::Attention,
        source: "allowlisted adapter".into(),
        scope: "synthetic".into(),
        sampled_at: SystemTime::UNIX_EPOCH,
        freshness: FactFreshness::Unknown,
        summary: PermittedText::from_product_redaction("当前证据不足"),
        impact: PermittedText::from_product_redaction("Current delivery unproven"),
        evidence: vec![EvidenceField {
            label: "Allowed count".into(),
            value: PermittedText::from_product_redaction("3"),
        }],
        next_steps: vec![SafeNextStep {
            label: "Inspect".into(),
            intent: ActionIntent {
                action: Action::Help,
                effect: ActionEffect::View,
                count: 1,
            },
        }],
    }
}
#[test]
fn smart_raw_same_facts_revision_and_safe_steps() {
    let facts = [
        Verdict::Blocked,
        Verdict::Unproven,
        Verdict::StaticPass,
        Verdict::Fail,
        Verdict::Stale,
        Verdict::Unknown,
        Verdict::Pass,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, v)| fact(v, &i.to_string()))
    .collect();
    let document = UiDocument::new(42, SystemTime::UNIX_EPOCH, facts).unwrap();
    let mut view = ViewState::default();
    let smart = document.project(view.density);
    view.toggle();
    let raw = document.project(view.density);
    assert_eq!(smart.revision(), raw.revision());
    assert_eq!(smart.revision(), 42);
    assert!(std::ptr::eq(smart.facts(), raw.facts()));
    for fact in document.facts() {
        assert!(smart.lines().join("\n").contains(fact.verdict.marker()));
        assert!(raw.lines().join("\n").contains(fact.verdict.marker()));
    }
    assert!(!smart.lines().join("\n").contains("Allowed count"));
    assert!(raw.lines().join("\n").contains("Allowed count: 3"));
    let mut invalid = fact(Verdict::Pass, "unsafe");
    invalid.next_steps[0].intent.effect = ActionEffect::ProductWriteRequest;
    assert!(UiDocument::new(1, SystemTime::UNIX_EPOCH, vec![invalid]).is_err());
    assert!(
        UiDocument::new(
            1,
            SystemTime::UNIX_EPOCH,
            vec![
                fact(Verdict::Pass, "duplicate"),
                fact(Verdict::Pass, "duplicate")
            ]
        )
        .is_err()
    );
}
#[test]
fn neutral_theme_no_color_and_locale_plural_are_explicit() {
    for policy in [ColorPolicy::Auto, ColorPolicy::Always, ColorPolicy::Never] {
        for level in [
            ColorLevel::None,
            ColorLevel::Ansi16,
            ColorLevel::Ansi256,
            ColorLevel::TrueColor,
        ] {
            assert_eq!(
                ThemeResolver::resolve(policy, true, true, level).color_level,
                ColorLevel::None
            );
        }
    }
    assert_eq!(
        ThemeResolver::resolve(ColorPolicy::Auto, true, false, ColorLevel::Ansi16).color_level,
        ColorLevel::Ansi16
    );
    assert_eq!(message(Locale::ZhCn, MessageKey::Loading), "加载中");
    assert_eq!(row_count_message(Locale::EnUs, 1), "1 row");
    assert_eq!(row_count_message(Locale::EnUs, 2), "2 rows");
    assert_eq!(row_count_message(Locale::ZhCn, 2), "2 行");
}
