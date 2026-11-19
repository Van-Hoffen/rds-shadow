use crate::config::Config;
use crate::event::{ActionMode, AuditEvent, EventKind, ResultKind};
use crate::writer::JsonlWriter;

use crate::platform;

/// Write an audit event to the local log and, if configured, to the share.
/// A failed write is reported to the caller, never panics and never blocks
/// an already confirmed action.
pub fn record(cfg: &Config, mut event: AuditEvent) {
    let (specialist, workstation) = platform::identity();
    event.specialist = specialist;
    event.specialist_sid = platform::current_user_sid();
    event.workstation = workstation.clone();

    let writer = JsonlWriter::new(&cfg.audit.log_dir);
    if let Err(e) = writer.append(&event) {
        eprintln!("warn: local audit write failed: {e}");
    }
    if let Some(err) = writer.append_optional(cfg.audit.share_dir.as_deref(), &event, &workstation)
    {
        eprintln!("warn: share audit write failed: {err}");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn denied(
    cfg: &Config,
    kind: EventKind,
    server: &str,
    session: u32,
    mode: Option<ActionMode>,
    ticket: &str,
    reason: &str,
) {
    let mut e = AuditEvent::new(kind, ResultKind::Denied);
    e.target_server = Some(server.to_string());
    e.session_id = Some(session);
    e.mode = mode;
    e.ticket = Some(ticket.to_string());
    e.error = Some(reason.to_string());
    record(cfg, e);
}
