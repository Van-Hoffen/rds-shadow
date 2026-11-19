use chrono::{DateTime, Utc};

use crate::event::{AuditEvent, EventKind, ResultKind};

/// Filters used by the viewer. All set fields must match (AND).
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub specialist: Option<String>,
    pub target_user: Option<String>,
    pub server: Option<String>,
    pub event: Option<EventKind>,
    pub ticket: Option<String>,
    pub result: Option<ResultKind>,
}

impl EventFilter {
    pub fn matches(&self, e: &AuditEvent) -> bool {
        if let Some(since) = self.since {
            match parse_ts(&e.ts) {
                Some(ts) if ts >= since => {}
                _ => return false,
            }
        }
        if let Some(until) = self.until {
            match parse_ts(&e.ts) {
                Some(ts) if ts <= until => {}
                _ => return false,
            }
        }
        if let Some(v) = &self.specialist {
            if !contains_ci(&e.specialist, v) {
                return false;
            }
        }
        if let Some(v) = &self.target_user {
            if !e.target_user.as_deref().is_some_and(|s| contains_ci(s, v)) {
                return false;
            }
        }
        if let Some(v) = &self.server {
            if !e
                .target_server
                .as_deref()
                .is_some_and(|s| contains_ci(s, v))
            {
                return false;
            }
        }
        if let Some(v) = self.event {
            if e.event != v {
                return false;
            }
        }
        if let Some(v) = &self.ticket {
            if !e.ticket.as_deref().is_some_and(|s| contains_ci(s, v)) {
                return false;
            }
        }
        if let Some(v) = self.result {
            if e.result != v {
                return false;
            }
        }
        true
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn parse_ts(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ActionMode, ResultKind};

    fn sample() -> AuditEvent {
        let mut e = AuditEvent::new(EventKind::ShadowStart, ResultKind::Ok);
        e.ts = "2026-11-18T09:12:33.123Z".into();
        e.specialist = r"CORP\A.Ivanov".into();
        e.target_user = Some(r"CORP\M.Petrov".into());
        e.target_server = Some("ts03".into());
        e.mode = Some(ActionMode::View);
        e.ticket = Some("SD-12345".into());
        e
    }

    #[test]
    fn empty_filter_matches() {
        assert!(EventFilter::default().matches(&sample()));
    }

    #[test]
    fn matches_case_insensitive_fields() {
        let f = EventFilter {
            specialist: Some("a.ivanov".into()),
            target_user: Some("petrov".into()),
            server: Some("TS03".into()),
            ..Default::default()
        };
        assert!(f.matches(&sample()));
    }

    #[test]
    fn rejects_wrong_ticket() {
        let f = EventFilter {
            ticket: Some("SD-999".into()),
            ..Default::default()
        };
        assert!(!f.matches(&sample()));
    }

    #[test]
    fn respects_time_window() {
        let f = EventFilter {
            since: Some("2026-11-18T09:00:00Z".parse().unwrap()),
            until: Some("2026-11-18T10:00:00Z".parse().unwrap()),
            ..Default::default()
        };
        assert!(f.matches(&sample()));
        let f = EventFilter {
            since: Some("2026-11-18T10:00:00Z".parse().unwrap()),
            ..Default::default()
        };
        assert!(!f.matches(&sample()));
    }
}
