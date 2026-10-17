//! Locale-selection primitives shared by Human terminal consumers.

/// Supported concrete Human locales.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locale {
    EnUs,
    ZhCn,
}

impl Locale {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::ZhCn => "zh-CN",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().replace('_', "-").to_ascii_lowercase().as_str() {
            "en" | "en-us" => Some(Self::EnUs),
            "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" => Some(Self::ZhCn),
            _ => None,
        }
    }
}

/// The first accepted source of the resolved Human locale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocaleSource {
    Cli,
    Environment,
    Preference,
    OperatingSystem,
    Fallback,
}

/// The accepted locale together with the source that selected it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocaleResolution {
    pub locale: Locale,
    pub source: LocaleSource,
}

/// Consumer-observed locale candidates in normative precedence order.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocaleInputs<'a> {
    pub cli: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub preference: Option<&'a str>,
    pub operating_system: Option<&'a str>,
}

/// Resolves CLI, environment, preference, OS locale, then en-US fallback.
#[must_use]
pub fn resolve_locale(inputs: LocaleInputs<'_>) -> LocaleResolution {
    for (candidate, source) in [
        (inputs.cli, LocaleSource::Cli),
        (inputs.environment, LocaleSource::Environment),
        (inputs.preference, LocaleSource::Preference),
        (inputs.operating_system, LocaleSource::OperatingSystem),
    ] {
        if let Some(locale) = candidate.and_then(Locale::parse) {
            return LocaleResolution { locale, source };
        }
    }
    LocaleResolution {
        locale: Locale::EnUs,
        source: LocaleSource::Fallback,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKey {
    Focus,
    Mode,
    Resize,
    Loading,
    Empty,
    Error,
    Conflict,
    Help,
    Smart,
    Raw,
    Freshness,
    Unknown,
    Current,
    Historical,
    Attention,
    NextSafeStep,
    Navigation,
    Details,
}

#[must_use]
pub fn message(locale: Locale, key: MessageKey) -> &'static str {
    match (locale, key) {
        (Locale::EnUs, MessageKey::Focus) => "Focus",
        (Locale::ZhCn, MessageKey::Focus) => "焦点",
        (Locale::EnUs, MessageKey::Mode) => "Mode",
        (Locale::ZhCn, MessageKey::Mode) => "模式",
        (Locale::EnUs, MessageKey::Resize) => "Resize terminal (minimum 24x10)",
        (Locale::ZhCn, MessageKey::Resize) => "请调整终端（至少 24x10）",
        (Locale::EnUs, MessageKey::Loading) => "Loading",
        (Locale::ZhCn, MessageKey::Loading) => "加载中",
        (Locale::EnUs, MessageKey::Empty) => "Empty",
        (Locale::ZhCn, MessageKey::Empty) => "无数据",
        (Locale::EnUs, MessageKey::Error) => "Error",
        (Locale::ZhCn, MessageKey::Error) => "错误",
        (Locale::EnUs, MessageKey::Conflict) => "Conflict",
        (Locale::ZhCn, MessageKey::Conflict) => "冲突",
        (Locale::EnUs, MessageKey::Help) => "Help",
        (Locale::ZhCn, MessageKey::Help) => "帮助",
        (Locale::EnUs, MessageKey::Smart) => "Smart",
        (Locale::ZhCn, MessageKey::Smart) => "摘要",
        (Locale::EnUs, MessageKey::Raw) => "Raw",
        (Locale::ZhCn, MessageKey::Raw) => "详情",
        (Locale::EnUs, MessageKey::Freshness) => "Freshness",
        (Locale::ZhCn, MessageKey::Freshness) => "时效",
        (Locale::EnUs, MessageKey::Unknown) => "Unknown",
        (Locale::ZhCn, MessageKey::Unknown) => "未证实",
        (Locale::EnUs, MessageKey::Current) => "Current observation",
        (Locale::ZhCn, MessageKey::Current) => "当前观察",
        (Locale::EnUs, MessageKey::Historical) => "Historical",
        (Locale::ZhCn, MessageKey::Historical) => "历史",
        (Locale::EnUs, MessageKey::Attention) => "Attention",
        (Locale::ZhCn, MessageKey::Attention) => "需关注",
        (Locale::EnUs, MessageKey::NextSafeStep) => "Next safe step",
        (Locale::ZhCn, MessageKey::NextSafeStep) => "下一安全步骤",
        (Locale::EnUs, MessageKey::Navigation) => "Navigation",
        (Locale::ZhCn, MessageKey::Navigation) => "导航",
        (Locale::EnUs, MessageKey::Details) => "Details",
        (Locale::ZhCn, MessageKey::Details) => "详情",
    }
}

#[must_use]
pub fn row_count_message(locale: Locale, count: usize) -> String {
    match locale {
        Locale::ZhCn => format!("{count} 行"),
        Locale::EnUs => format!("{count} {}", if count == 1 { "row" } else { "rows" }),
    }
}

#[cfg(test)]
mod tests {
    use super::{Locale, LocaleInputs, LocaleSource, resolve_locale};

    #[test]
    fn source_precedence_is_contractual() {
        let resolved = resolve_locale(LocaleInputs {
            cli: Some("en-US"),
            environment: Some("zh-CN"),
            preference: Some("zh-CN"),
            operating_system: Some("zh-CN"),
        });
        assert_eq!(resolved.locale, Locale::EnUs);
        assert_eq!(resolved.source, LocaleSource::Cli);

        let resolved = resolve_locale(LocaleInputs {
            cli: None,
            environment: None,
            preference: Some("auto"),
            operating_system: Some("zh-Hans"),
        });
        assert_eq!(resolved.locale, Locale::ZhCn);
        assert_eq!(resolved.source, LocaleSource::OperatingSystem);
    }
}
