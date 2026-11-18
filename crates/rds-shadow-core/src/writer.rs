use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::event::AuditEvent;

/// Appends events to daily JSONL files under a directory.
///
/// Failure to write is returned to the caller. In v1 the caller does not block
/// an already confirmed action because of a failed write; it only surfaces the
/// error.
#[derive(Debug, Clone)]
pub struct JsonlWriter {
    dir: PathBuf,
}

impl JsonlWriter {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Append one line and flush. Returns the file that was written.
    pub fn append(&self, event: &AuditEvent) -> io::Result<PathBuf> {
        let date = event
            .date_part()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "event ts is not RFC3339"))?;
        fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(format!("{date}.jsonl"));
        let line = event
            .to_json_line()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(path)
    }

    /// Best-effort copy to a share. Never panics.
    pub fn append_optional(
        &self,
        share: Option<&Path>,
        event: &AuditEvent,
        workstation: &str,
    ) -> Option<io::Error> {
        let share = share?;
        let target = share.join(sanitize(workstation));
        let writer = JsonlWriter::new(target);
        writer.append(event).err()
    }
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unknown".into()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, ResultKind};

    #[test]
    fn writes_one_line_per_event_and_appends() {
        let dir = std::env::temp_dir().join(format!("rds-shadow-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let writer = JsonlWriter::new(&dir);

        let mut a = AuditEvent::new(EventKind::ShadowStart, ResultKind::Ok);
        a.ts = "2026-11-18T09:00:00.000Z".into();
        let mut b = AuditEvent::new(EventKind::ShadowEnd, ResultKind::Ok);
        b.ts = "2026-11-18T09:01:00.000Z".into();

        let p1 = writer.append(&a).unwrap();
        let p2 = writer.append(&b).unwrap();
        assert_eq!(p1, p2);
        assert!(p1.ends_with("2026-11-18.jsonl"));

        let text = fs::read_to_string(&p1).unwrap();
        assert_eq!(text.lines().count(), 2);
        for line in text.lines() {
            let parsed: AuditEvent = serde_json::from_str(line).unwrap();
            assert!(parsed.event.as_str().starts_with("shadow."));
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitizes_workstation_for_share() {
        assert_eq!(sanitize("WS-042"), "WS-042");
        assert_eq!(sanitize("../../etc"), "______etc");
        assert_eq!(sanitize(""), "unknown");
    }
}
