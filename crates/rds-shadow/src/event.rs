use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

/// One audit line. The same shape is written to local JSONL and copied to a
/// share. Future syslog transport can reuse the same JSON body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub v: u8,
    pub ts: String,
    pub event: EventKind,
    pub result: ResultKind,
    pub specialist: String,
    pub specialist_sid: String,
    pub workstation: String,
    pub target_user: Option<String>,
    pub target_server: Option<String>,
    pub session_id: Option<u32>,
    pub session_state: Option<String>,
    pub mode: Option<ActionMode>,
    pub ticket: Option<String>,
    pub client_version: String,
    pub message_text: Option<String>,
    pub message_len: Option<usize>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

impl AuditEvent {
    pub fn new(event: EventKind, result: ResultKind) -> Self {
        Self {
            v: 1,
            ts: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            event,
            result,
            specialist: String::new(),
            specialist_sid: String::new(),
            workstation: String::new(),
            target_user: None,
            target_server: None,
            session_id: None,
            session_state: None,
            mode: None,
            ticket: None,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            message_text: None,
            message_len: None,
            duration_ms: None,
            error: None,
        }
    }

    /// `YYYY-MM-DD` derived from `ts`. Used as the daily file name.
    pub fn date_part(&self) -> Option<&str> {
        self.ts
            .get(..10)
            .filter(|d| d.len() == 10 && d.as_bytes()[4] == b'-')
    }

    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    #[serde(rename = "shadow.start")]
    ShadowStart,
    #[serde(rename = "shadow.end")]
    ShadowEnd,
    #[serde(rename = "shadow.denied")]
    ShadowDenied,
    #[serde(rename = "message.send")]
    MessageSend,
    #[serde(rename = "message.denied")]
    MessageDenied,
    #[serde(rename = "logoff.requested")]
    LogoffRequested,
    #[serde(rename = "logoff.ok")]
    LogoffOk,
    #[serde(rename = "logoff.fail")]
    LogoffFail,
    #[serde(rename = "logoff.denied")]
    LogoffDenied,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShadowStart => "shadow.start",
            Self::ShadowEnd => "shadow.end",
            Self::ShadowDenied => "shadow.denied",
            Self::MessageSend => "message.send",
            Self::MessageDenied => "message.denied",
            Self::LogoffRequested => "logoff.requested",
            Self::LogoffOk => "logoff.ok",
            Self::LogoffFail => "logoff.fail",
            Self::LogoffDenied => "logoff.denied",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Some(match raw {
            "shadow.start" => Self::ShadowStart,
            "shadow.end" => Self::ShadowEnd,
            "shadow.denied" => Self::ShadowDenied,
            "message.send" => Self::MessageSend,
            "message.denied" => Self::MessageDenied,
            "logoff.requested" => Self::LogoffRequested,
            "logoff.ok" => Self::LogoffOk,
            "logoff.fail" => Self::LogoffFail,
            "logoff.denied" => Self::LogoffDenied,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultKind {
    Ok,
    Denied,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionMode {
    View,
    Control,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_keeps_nulls_and_dotted_names() {
        let mut e = AuditEvent::new(EventKind::ShadowStart, ResultKind::Ok);
        e.ts = "2026-11-18T09:12:33.123Z".into();
        e.specialist = r"CORP\a.ivanov".into();
        e.mode = Some(ActionMode::View);
        let line = e.to_json_line().unwrap();
        assert!(line.contains("\"shadow.start\""));
        assert!(line.contains("\"target_user\":null"));
        assert!(line.ends_with('\n'));
        let back: AuditEvent = serde_json::from_str(line.trim_end()).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn date_part_is_utc_day() {
        let mut e = AuditEvent::new(EventKind::ShadowEnd, ResultKind::Fail);
        e.ts = "2026-12-31T23:59:59.999Z".into();
        assert_eq!(e.date_part(), Some("2026-12-31"));
    }

    #[test]
    fn event_names_are_stable() {
        for (kind, text) in [
            (EventKind::ShadowStart, "shadow.start"),
            (EventKind::LogoffDenied, "logoff.denied"),
        ] {
            assert_eq!(kind.as_str(), text);
            assert_eq!(EventKind::parse(text), Some(kind));
        }
    }
}
