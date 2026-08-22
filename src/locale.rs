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
