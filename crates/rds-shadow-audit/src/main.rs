use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use clap::Parser;
use rds_shadow_core::event::{AuditEvent, EventKind, ResultKind};
use rds_shadow_core::filter::EventFilter;

/// Read-only viewer for RDS Shadow JSONL audit files.
///
/// It does not authenticate users and does not enforce access itself: who can
/// read the directory can read the events. Protect the directory with OS ACLs.
#[derive(Debug, Parser)]
#[command(name = "rds-shadow-audit", version, about)]
struct Args {
    /// Directory with daily `*.jsonl` files. Scanned recursively.
    #[arg(long, value_name = "DIR")]
    dir: PathBuf,

    /// Start of the interval, RFC3339, e.g. 2026-11-18T00:00:00Z.
    #[arg(long)]
    since: Option<String>,

    /// End of the interval, RFC3339.
    #[arg(long)]
    until: Option<String>,

    /// Specialist account substring, case-insensitive.
    #[arg(long)]
    specialist: Option<String>,

    /// Target user substring, case-insensitive.
    #[arg(long, value_name = "USER")]
    user: Option<String>,

    /// Session host substring, case-insensitive.
    #[arg(long)]
    server: Option<String>,

    /// Exact event name, e.g. `shadow.start`.
    #[arg(long)]
    event: Option<String>,

    /// Ticket substring, case-insensitive.
    #[arg(long)]
    ticket: Option<String>,

    /// Exact result: ok | denied | fail.
    #[arg(long)]
    result: Option<String>,

    /// Print CSV instead of a table.
    #[arg(long)]
    csv: bool,

    /// Maximum rows to print after filtering.
    #[arg(long, default_value_t = 1000)]
    limit: usize,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("rds-shadow-audit: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let filter = build_filter(&args)?;
    let mut events = read_all(&args.dir)?;
    events.retain(|e| filter.matches(e));
    events.sort_by(|a, b| a.ts.cmp(&b.ts));
    events.truncate(args.limit);

    let stdout = io::stdout();
    let mut out = stdout.lock();
    if args.csv {
        write_csv(&mut out, &events)?;
    } else {
        write_table(&mut out, &events)?;
        writeln!(out, "\n{} event(s)", events.len())?;
    }
    Ok(())
}

fn build_filter(args: &Args) -> Result<EventFilter, String> {
    let parse_ts = |raw: &str, name: &str| -> Result<DateTime<Utc>, String> {
        DateTime::parse_from_rfc3339(raw)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|_| format!("--{name} must be RFC3339, got `{raw}`"))
    };
    let event = match &args.event {
        None => None,
        Some(raw) => Some(EventKind::parse(raw).ok_or_else(|| format!("unknown event `{raw}`"))?),
    };
    let result = match &args.result {
        None => None,
        Some(raw) => Some(match raw.to_ascii_lowercase().as_str() {
            "ok" => ResultKind::Ok,
            "denied" => ResultKind::Denied,
            "fail" => ResultKind::Fail,
            other => return Err(format!("unknown result `{other}`")),
        }),
    };
    Ok(EventFilter {
        since: args
            .since
            .as_deref()
            .map(|v| parse_ts(v, "since"))
            .transpose()?,
        until: args
            .until
            .as_deref()
            .map(|v| parse_ts(v, "until"))
            .transpose()?,
        specialist: args.specialist.clone(),
        target_user: args.user.clone(),
        server: args.server.clone(),
        event,
        ticket: args.ticket.clone(),
        result,
    })
}

fn read_all(dir: &Path) -> Result<Vec<AuditEvent>, Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()).into());
    }
    let mut files = Vec::new();
    collect_jsonl(dir, &mut files)?;
    files.sort();

    let mut events = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path)?;
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<AuditEvent>(line) {
                Ok(e) => events.push(e),
                Err(err) => eprintln!("skip {}:{}: {err}", path.display(), idx + 1),
            }
        }
    }
    Ok(events)
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            out.push(path);
        }
    }
    Ok(())
}

fn write_table(out: &mut impl Write, events: &[AuditEvent]) -> io::Result<()> {
    writeln!(
        out,
        "{:<23} {:<19} {:<7} {:<24} {:<20} {:<16} {:<8} {:<10}",
        "ts", "event", "result", "specialist", "user", "server", "session", "ticket"
    )?;
    for e in events {
        writeln!(
            out,
            "{:<23} {:<19} {:<7} {:<24} {:<20} {:<16} {:<8} {:<10}",
            e.ts,
            e.event.as_str(),
            result_str(e.result),
            truncate(&e.specialist, 24),
            truncate(e.target_user.as_deref().unwrap_or("-"), 20),
            truncate(e.target_server.as_deref().unwrap_or("-"), 16),
            e.session_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".into()),
            truncate(e.ticket.as_deref().unwrap_or("-"), 10),
        )?;
    }
    Ok(())
}

fn write_csv(out: &mut impl Write, events: &[AuditEvent]) -> io::Result<()> {
    writeln!(
        out,
        "ts,event,result,specialist,user,server,session,mode,ticket,duration_ms"
    )?;
    for e in events {
        writeln!(
            out,
            "{},{},{},{},{},{},{},{},{},{}",
            csv(&e.ts),
            e.event.as_str(),
            result_str(e.result),
            csv(&e.specialist),
            csv(e.target_user.as_deref().unwrap_or("")),
            csv(e.target_server.as_deref().unwrap_or("")),
            e.session_id.map(|v| v.to_string()).unwrap_or_default(),
            e.mode.map(|m| format!("{m:?}")).unwrap_or_default(),
            csv(e.ticket.as_deref().unwrap_or("")),
            e.duration_ms.map(|v| v.to_string()).unwrap_or_default(),
        )?;
    }
    Ok(())
}

fn result_str(r: ResultKind) -> &'static str {
    match r {
        ResultKind::Ok => "ok",
        ResultKind::Denied => "denied",
        ResultKind::Fail => "fail",
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}

fn csv(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
