use std::path::PathBuf;
use std::time::Instant;

use crate::config::Config;
use crate::event::{ActionMode, AuditEvent, EventKind, ResultKind};
use clap::{Parser, Subcommand};

use crate::audit;
use crate::platform;

#[derive(Debug, Parser)]
#[command(
    name = "rds-shadow",
    version,
    about = "RDS shadow client for support staff"
)]
struct Cli {
    /// Path to the INI configuration file.
    #[arg(long, value_name = "FILE", default_value = "rds-shadow.ini")]
    config: PathBuf,
    /// Broker name override for this run. Does not bypass group checks.
    #[arg(long, value_name = "HOST")]
    broker: Option<String>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// List sessions on configured hosts (or broker).
    List {
        /// Print JSON lines instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Open a shadow session. View-only unless --control is given and allowed.
    Shadow {
        #[arg(long)]
        server: String,
        #[arg(long)]
        session: u32,
        #[arg(long)]
        ticket: String,
        #[arg(long)]
        control: bool,
    },
    /// Send a message to a session.
    Message {
        #[arg(long)]
        server: String,
        #[arg(long)]
        session: u32,
        #[arg(long)]
        ticket: String,
        #[arg(long)]
        text: String,
    },
    /// Log off a session. Requires the typed target user name.
    Logoff {
        #[arg(long)]
        server: String,
        #[arg(long)]
        session: u32,
        #[arg(long)]
        ticket: String,
        #[arg(long, value_name = "USER")]
        confirm_user: String,
    },
    /// Validate the configuration and print effective settings.
    CheckConfig,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut cfg = crate::config::load(&cli.config)?;
    if let Some(broker) = cli.broker {
        cfg.connection.broker = Some(broker);
    }

    match cli.command {
        Cmd::CheckConfig => check_config(&cfg),
        Cmd::List { json } => list(&cfg, json),
        Cmd::Shadow {
            server,
            session,
            ticket,
            control,
        } => shadow(&cfg, &server, session, &ticket, control),
        Cmd::Message {
            server,
            session,
            ticket,
            text,
        } => message(&cfg, &server, session, &ticket, &text),
        Cmd::Logoff {
            server,
            session,
            ticket,
            confirm_user,
        } => logoff(&cfg, &server, session, &ticket, &confirm_user),
    }
}

fn check_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("mode: {}", cfg.connection.mode);
    println!(
        "broker: {}",
        cfg.connection.broker.as_deref().unwrap_or("-")
    );
    println!("rdp_port: {}", cfg.connection.rdp_port);
    println!(
        "servers: {}",
        if cfg.servers.is_empty() {
            "-".to_string()
        } else {
            cfg.servers.join(",")
        }
    );
    println!("log_dir: {}", cfg.audit.log_dir.display());
    println!(
        "share_dir: {}",
        cfg.audit
            .share_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".into())
    );
    println!("groups.view: {}", cfg.groups.view.as_deref().unwrap_or("-"));
    println!(
        "groups.control: {}",
        cfg.groups.control.as_deref().unwrap_or("-")
    );
    println!(
        "groups.logoff: {}",
        cfg.groups.logoff.as_deref().unwrap_or("-")
    );
    Ok(())
}

fn list(cfg: &Config, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hosts = platform::resolve_hosts(cfg)?;
    let (sessions, errors) = platform::list_all(&hosts);
    for e in errors {
        eprintln!("warn: {e}");
    }
    if json {
        for s in &sessions {
            println!(
                "{{\"server\":{},\"user\":{},\"session\":{},\"state\":{}}}",
                serde_json::to_string(&s.server)?,
                serde_json::to_string(&s.user)?,
                s.id,
                serde_json::to_string(&s.state)?
            );
        }
        return Ok(());
    }
    println!(
        "{:<20} {:<28} {:<8} {:<14}",
        "server", "user", "id", "state"
    );
    for s in &sessions {
        println!(
            "{:<20} {:<28} {:<8} {:<14}",
            s.server, s.user, s.id, s.state
        );
    }
    println!("\n{} session(s)", sessions.len());
    Ok(())
}

fn shadow(
    cfg: &Config,
    server: &str,
    session: u32,
    ticket: &str,
    control: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if ticket.trim().is_empty() {
        return Err("ticket is required".into());
    }
    let mode = if control {
        ActionMode::Control
    } else {
        ActionMode::View
    };
    let gate = if control {
        cfg.groups.control.as_deref()
    } else {
        cfg.groups.view.as_deref()
    };
    if !gate.is_some_and(platform::in_group) {
        audit::denied(
            cfg,
            EventKind::ShadowDenied,
            server,
            session,
            Some(mode),
            ticket,
            "required group is not configured or current user is not a member",
        );
        return Err("permission denied".into());
    }

    let mut start = AuditEvent::new(EventKind::ShadowStart, ResultKind::Ok);
    start.target_server = Some(server.to_string());
    start.session_id = Some(session);
    start.mode = Some(mode);
    start.ticket = Some(ticket.to_string());
    audit::record(cfg, start);

    let began = Instant::now();
    let status = platform::run_mstsc(server, session, control, cfg.connection.rdp_port)?;

    let mut end = AuditEvent::new(
        EventKind::ShadowEnd,
        if status.success() {
            ResultKind::Ok
        } else {
            ResultKind::Fail
        },
    );
    end.target_server = Some(server.to_string());
    end.session_id = Some(session);
    end.mode = Some(mode);
    end.ticket = Some(ticket.to_string());
    end.duration_ms = Some(began.elapsed().as_millis() as u64);
    if !status.success() {
        end.error = Some(format!("mstsc exited with {status}"));
    }
    audit::record(cfg, end);
    Ok(())
}

fn message(
    cfg: &Config,
    server: &str,
    session: u32,
    ticket: &str,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if ticket.trim().is_empty() {
        return Err("ticket is required".into());
    }
    if text.trim().is_empty() {
        return Err("message text is required".into());
    }
    if !cfg.groups.view.as_deref().is_some_and(platform::in_group) {
        audit::denied(
            cfg,
            EventKind::MessageDenied,
            server,
            session,
            None,
            ticket,
            "view group is not configured or current user is not a member",
        );
        return Err("permission denied".into());
    }
    platform::send_message(server, session, "RDS Shadow", text)?;

    let mut e = AuditEvent::new(EventKind::MessageSend, ResultKind::Ok);
    e.target_server = Some(server.to_string());
    e.session_id = Some(session);
    e.ticket = Some(ticket.to_string());
    e.message_text = Some(text.to_string());
    e.message_len = Some(text.chars().count());
    audit::record(cfg, e);
    Ok(())
}

fn logoff(
    cfg: &Config,
    server: &str,
    session: u32,
    ticket: &str,
    confirm_user: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if ticket.trim().is_empty() {
        return Err("ticket is required".into());
    }
    if !cfg.groups.logoff.as_deref().is_some_and(platform::in_group) {
        audit::denied(
            cfg,
            EventKind::LogoffDenied,
            server,
            session,
            None,
            ticket,
            "logoff group is not configured or current user is not a member",
        );
        return Err("permission denied".into());
    }
    let target = platform::session_user(server, session)?;
    let typed = confirm_user.trim();
    if !target.eq_ignore_ascii_case(typed) {
        audit::denied(
            cfg,
            EventKind::LogoffDenied,
            server,
            session,
            None,
            ticket,
            "typed user does not match session user",
        );
        return Err(format!("typed user `{typed}` does not match session user `{target}`").into());
    }

    let mut requested = AuditEvent::new(EventKind::LogoffRequested, ResultKind::Ok);
    requested.target_server = Some(server.to_string());
    requested.target_user = Some(target.clone());
    requested.session_id = Some(session);
    requested.ticket = Some(ticket.to_string());
    audit::record(cfg, requested);

    match platform::logoff(server, session) {
        Ok(()) => {
            let mut ok = AuditEvent::new(EventKind::LogoffOk, ResultKind::Ok);
            ok.target_server = Some(server.to_string());
            ok.target_user = Some(target);
            ok.session_id = Some(session);
            ok.ticket = Some(ticket.to_string());
            audit::record(cfg, ok);
            Ok(())
        }
        Err(err) => {
            let mut fail = AuditEvent::new(EventKind::LogoffFail, ResultKind::Fail);
            fail.target_server = Some(server.to_string());
            fail.target_user = Some(target);
            fail.session_id = Some(session);
            fail.ticket = Some(ticket.to_string());
            fail.error = Some(err.clone());
            audit::record(cfg, fail);
            Err(err.into())
        }
    }
}
