use std::fmt;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// How the client discovers session hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    /// Ask the RD Connection Broker for collections and hosts.
    Broker,
    /// Only poll the hosts listed in `[servers] list`.
    Servers,
}

impl ConnectionMode {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "broker" => Some(Self::Broker),
            "servers" => Some(Self::Servers),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Broker => "broker",
            Self::Servers => "servers",
        }
    }
}

impl fmt::Display for ConnectionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub connection: Connection,
    pub servers: Vec<String>,
    pub groups: Groups,
    pub audit: Audit,
    pub ui: Ui,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub mode: ConnectionMode,
    pub broker: Option<String>,
    pub rdp_port: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Groups {
    pub view: Option<String>,
    pub control: Option<String>,
    pub logoff: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audit {
    pub log_dir: PathBuf,
    pub share_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Auto,
    Ru,
    En,
}

impl Language {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "ru" => Some(Self::Ru),
            "en" => Some(Self::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ui {
    pub language: Language,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("line {line}: expected `key = value`, got `{raw}`")]
    BadLine { line: usize, raw: String },
    #[error("line {line}: key outside a section")]
    KeyOutsideSection { line: usize },
    #[error("line {line}: section `{name}` is empty")]
    EmptySection { line: usize, name: String },
    #[error("unknown section `{0}`")]
    UnknownSection(String),
    #[error("missing required section `{0}`")]
    MissingSection(String),
    #[error("missing required key `{0}`")]
    MissingKey(String),
    #[error("invalid value for `{key}`: `{value}`")]
    InvalidValue { key: String, value: String },
    #[error("mode = broker requires a non-empty `broker`")]
    BrokerRequired,
    #[error("mode = servers requires at least one host in `[servers] list`")]
    ServersRequired,
    #[error("log_dir is empty")]
    LogDirEmpty,
}

/// Load and validate a configuration file.
pub fn load(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path.as_ref())
        .map_err(|_| ConfigError::MissingSection("file".into()))?;
    parse(&text)
}

/// Parse and validate INI text.
///
/// The format is deliberately small: sections, `key = value`, `;` and `#`
/// comments, blank lines. No escapes, no interpolation, no secrets.
pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut mode: Option<ConnectionMode> = None;
    let mut broker: Option<String> = None;
    let mut rdp_port: Option<u16> = None;
    let mut servers: Option<Vec<String>> = None;
    let mut view: Option<String> = None;
    let mut control: Option<String> = None;
    let mut logoff: Option<String> = None;
    let mut log_dir: Option<PathBuf> = None;
    let mut share_dir: Option<PathBuf> = None;
    let mut language: Option<Language> = None;

    let mut section: Option<String> = None;
    let mut seen_sections = std::collections::HashSet::new();

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let Some(name) = rest.strip_suffix(']') else {
                return Err(ConfigError::BadLine {
                    line: line_no,
                    raw: raw_line.into(),
                });
            };
            let name = name.trim().to_ascii_lowercase();
            if name.is_empty() {
                return Err(ConfigError::EmptySection {
                    line: line_no,
                    name,
                });
            }
            if !matches!(
                name.as_str(),
                "connection" | "servers" | "groups" | "audit" | "ui"
            ) {
                return Err(ConfigError::UnknownSection(name));
            }
            seen_sections.insert(name.clone());
            section = Some(name);
            continue;
        }

        let Some((key_raw, value_raw)) = line.split_once('=') else {
            return Err(ConfigError::BadLine {
                line: line_no,
                raw: raw_line.into(),
            });
        };
        let key = key_raw.trim().to_ascii_lowercase();
        let value = value_raw.trim().to_string();
        let Some(current) = section.as_deref() else {
            return Err(ConfigError::KeyOutsideSection { line: line_no });
        };

        match (current, key.as_str()) {
            ("connection", "mode") => {
                let Some(parsed) = ConnectionMode::parse(&value) else {
                    return Err(ConfigError::InvalidValue {
                        key: "connection.mode".into(),
                        value,
                    });
                };
                mode = Some(parsed);
            }
            ("connection", "broker") => broker = opt_string(&value),
            ("connection", "rdp_port") => {
                let parsed = value
                    .parse::<u16>()
                    .map_err(|_| ConfigError::InvalidValue {
                        key: "connection.rdp_port".into(),
                        value: value.clone(),
                    })?;
                if parsed == 0 {
                    return Err(ConfigError::InvalidValue {
                        key: "connection.rdp_port".into(),
                        value,
                    });
                }
                rdp_port = Some(parsed);
            }
            ("servers", "list") => {
                servers = Some(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToOwned::to_owned)
                        .collect(),
                );
            }
            ("groups", "view") => view = opt_string(&value),
            ("groups", "control") => control = opt_string(&value),
            ("groups", "logoff") => logoff = opt_string(&value),
            ("audit", "log_dir") => log_dir = opt_path(&value),
            ("audit", "share_dir") => share_dir = opt_path(&value),
            ("ui", "language") => {
                let Some(parsed) = Language::parse(&value) else {
                    return Err(ConfigError::InvalidValue {
                        key: "ui.language".into(),
                        value,
                    });
                };
                language = Some(parsed);
            }
            (_, _) => {
                // Unknown keys are ignored on purpose: forward compatibility
                // without silently changing behavior of known keys.
            }
        }
    }

    for required in ["connection", "audit"] {
        if !seen_sections.contains(required) {
            return Err(ConfigError::MissingSection(required.into()));
        }
    }

    let mode = mode.ok_or_else(|| ConfigError::MissingKey("connection.mode".into()))?;
    let broker = broker.filter(|b| !b.trim().is_empty());
    if mode == ConnectionMode::Broker && broker.is_none() {
        return Err(ConfigError::BrokerRequired);
    }

    let servers = servers.unwrap_or_default();
    if mode == ConnectionMode::Servers && servers.is_empty() {
        return Err(ConfigError::ServersRequired);
    }

    let log_dir = log_dir
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(ConfigError::LogDirEmpty)?;

    Ok(Config {
        connection: Connection {
            mode,
            broker,
            rdp_port: rdp_port.unwrap_or(3389),
        },
        servers,
        groups: Groups {
            view: view.filter(|s| !s.trim().is_empty()),
            control: control.filter(|s| !s.trim().is_empty()),
            logoff: logoff.filter(|s| !s.trim().is_empty()),
        },
        audit: Audit {
            log_dir,
            share_dir: share_dir.filter(|p| !p.as_os_str().is_empty()),
        },
        ui: Ui {
            language: language.unwrap_or(Language::Auto),
        },
    })
}

fn strip_comment(line: &str) -> &str {
    let mut cut = line.len();
    for (i, ch) in line.char_indices() {
        if ch == ';' || ch == '#' {
            cut = i;
            break;
        }
    }
    &line[..cut]
}

fn opt_string(value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

fn opt_path(value: &str) -> Option<PathBuf> {
    opt_string(value).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
; example
[connection]
mode = broker
broker = broker.example.domain
rdp_port = 3389

[servers]
list =

[groups]
view =
control =
logoff =

[audit]
log_dir = C:\ProgramData\rds-shadow\logs
share_dir =

[ui]
language = auto
"#;

    #[test]
    fn parses_example() {
        let cfg = parse(SAMPLE).expect("example must parse");
        assert_eq!(cfg.connection.mode, ConnectionMode::Broker);
        assert_eq!(
            cfg.connection.broker.as_deref(),
            Some("broker.example.domain")
        );
        assert_eq!(cfg.connection.rdp_port, 3389);
        assert!(cfg.groups.view.is_none());
        assert!(cfg.audit.share_dir.is_none());
        assert_eq!(cfg.ui.language, Language::Auto);
    }

    #[test]
    fn broker_requires_name() {
        let text = "[connection]\nmode = broker\n[audit]\nlog_dir = /tmp/x\n";
        assert_eq!(parse(text), Err(ConfigError::BrokerRequired));
    }

    #[test]
    fn servers_requires_host() {
        let text =
            "[connection]\nmode = servers\n[servers]\nlist = , ,\n[audit]\nlog_dir = /tmp/x\n";
        assert_eq!(parse(text), Err(ConfigError::ServersRequired));
    }

    #[test]
    fn servers_split_and_trim() {
        let text = "[connection]\nmode = servers\n[servers]\nlist = ts01, ts02 ,ts03\n[audit]\nlog_dir = /tmp/x\n";
        let cfg = parse(text).unwrap();
        assert_eq!(cfg.servers, vec!["ts01", "ts02", "ts03"]);
    }

    #[test]
    fn comments_and_blank_lines() {
        let text = "# top\n[connection] ; inline\nmode = servers\n\n[servers]\nlist = a\n[audit]\nlog_dir = /tmp/x # no\n";
        let cfg = parse(text).unwrap();
        assert_eq!(cfg.audit.log_dir, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn bad_port_is_rejected() {
        let text = "[connection]\nmode = servers\nrdp_port = 0\n[servers]\nlist = a\n[audit]\nlog_dir = /tmp/x\n";
        assert!(matches!(parse(text), Err(ConfigError::InvalidValue { .. })));
    }

    #[test]
    fn unknown_mode_is_rejected() {
        let text = "[connection]\nmode = cloud\n[audit]\nlog_dir = /tmp/x\n";
        assert!(matches!(parse(text), Err(ConfigError::InvalidValue { .. })));
    }
}
