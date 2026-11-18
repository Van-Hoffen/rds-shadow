#[cfg(not(windows))]
fn main() {
    eprintln!("rds-shadow: this binary is Windows-only (mstsc /shadow and WTS API)");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() {
    if let Err(e) = windows_app::run() {
        eprintln!("rds-shadow: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_app {
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Instant;

    use clap::{Parser, Subcommand};
    use rds_shadow_core::config::{Config, ConnectionMode};
    use rds_shadow_core::event::{ActionMode, AuditEvent, EventKind, ResultKind};
    use rds_shadow_core::writer::JsonlWriter;

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
        let cfg = rds_shadow_core::config::load(&cli.config)?;
        let mut cfg = cfg;
        if let Some(broker) = cli.broker {
            cfg.connection.broker = Some(broker);
        }

        match cli.command {
            Cmd::CheckConfig => {
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

    fn identity() -> (String, String) {
        let user = std::env::var("USERNAME").unwrap_or_default();
        let domain = std::env::var("USERDOMAIN").unwrap_or_default();
        let workstation = std::env::var("COMPUTERNAME").unwrap_or_default();
        let specialist = if domain.is_empty() {
            user
        } else {
            format!("{domain}\\{user}")
        };
        (specialist, workstation)
    }

    fn writer(cfg: &Config) -> JsonlWriter {
        JsonlWriter::new(&cfg.audit.log_dir)
    }

    fn record(cfg: &Config, event: AuditEvent) {
        let (specialist, workstation) = identity();
        let mut event = event;
        event.specialist = specialist;
        event.specialist_sid = current_user_sid();
        event.workstation = workstation.clone();
        let local = writer(cfg).append(&event);
        if let Err(e) = &local {
            eprintln!("warn: local audit write failed: {e}");
        }
        if let Some(err) =
            writer(cfg).append_optional(cfg.audit.share_dir.as_deref(), &event, &workstation)
        {
            eprintln!("warn: share audit write failed: {err}");
        }
    }

    fn denied(
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
        eprintln!("denied: {reason}");
    }

    fn require_group(_cfg: &Config, group: Option<&str>) -> bool {
        match group {
            None => false,
            Some(name) => in_group(name),
        }
    }

    fn list(cfg: &Config, json: bool) -> Result<(), Box<dyn std::error::Error>> {
        let hosts = resolve_hosts(cfg)?;
        let mut sessions = Vec::new();
        for host in &hosts {
            match wts::list_sessions(host) {
                Ok(mut found) => sessions.append(&mut found),
                Err(e) => eprintln!("warn: {host}: {e}"),
            }
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

    fn resolve_hosts(cfg: &Config) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match cfg.connection.mode {
            ConnectionMode::Servers => Ok(cfg.servers.clone()),
            ConnectionMode::Broker => {
                // v1: broker discovery is not implemented yet. Use the explicit
                // server list instead of guessing collections.
                if cfg.servers.is_empty() {
                    Err("broker mode is configured but broker discovery is not implemented in this build; fill [servers] list".into())
                } else {
                    Ok(cfg.servers.clone())
                }
            }
        }
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
        if !require_group(cfg, gate) {
            denied(
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
        record(cfg, start);

        let args = rds_shadow_core::command::mstsc_args(server, session, control);
        let began = Instant::now();
        let status = Command::new("mstsc.exe").args(&args).status()?;

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
        record(cfg, end);
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
        if !require_group(cfg, cfg.groups.view.as_deref()) {
            denied(
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
        wts::send_message(server, session, "RDS Shadow", text)?;

        let mut e = AuditEvent::new(EventKind::MessageSend, ResultKind::Ok);
        e.target_server = Some(server.to_string());
        e.session_id = Some(session);
        e.ticket = Some(ticket.to_string());
        e.message_text = Some(text.to_string());
        e.message_len = Some(text.chars().count());
        record(cfg, e);
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
        if !require_group(cfg, cfg.groups.logoff.as_deref()) {
            denied(
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
        let target = wts::session_user(server, session)?;
        let typed = confirm_user.trim();
        if !target.eq_ignore_ascii_case(typed) {
            denied(
                cfg,
                EventKind::LogoffDenied,
                server,
                session,
                None,
                ticket,
                "typed user does not match session user",
            );
            return Err(
                format!("typed user `{typed}` does not match session user `{target}`").into(),
            );
        }

        let mut requested = AuditEvent::new(EventKind::LogoffRequested, ResultKind::Ok);
        requested.target_server = Some(server.to_string());
        requested.target_user = Some(target.clone());
        requested.session_id = Some(session);
        requested.ticket = Some(ticket.to_string());
        record(cfg, requested);

        match wts::logoff(server, session) {
            Ok(()) => {
                let mut ok = AuditEvent::new(EventKind::LogoffOk, ResultKind::Ok);
                ok.target_server = Some(server.to_string());
                ok.target_user = Some(target);
                ok.session_id = Some(session);
                ok.ticket = Some(ticket.to_string());
                record(cfg, ok);
                Ok(())
            }
            Err(err) => {
                let mut fail = AuditEvent::new(EventKind::LogoffFail, ResultKind::Fail);
                fail.target_server = Some(server.to_string());
                fail.target_user = Some(target);
                fail.session_id = Some(session);
                fail.ticket = Some(ticket.to_string());
                fail.error = Some(err.clone());
                record(cfg, fail);
                Err(err.into())
            }
        }
    }

    fn current_user_sid() -> String {
        unsafe { wts::current_user_sid().unwrap_or_default() }
    }

    fn in_group(name: &str) -> bool {
        unsafe { wts::in_group(name) }
    }

    mod wts {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        use windows_sys::Win32::Foundation::{
            GetLastError, LocalFree, BOOL, ERROR_INSUFFICIENT_BUFFER, HANDLE, HLOCAL,
        };
        use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
        use windows_sys::Win32::Security::{
            CheckTokenMembership, GetTokenInformation, LookupAccountNameW, TokenUser, TOKEN_QUERY,
            TOKEN_USER,
        };
        use windows_sys::Win32::System::RemoteDesktop::{
            WTSCloseServer, WTSEnumerateSessionsW, WTSFreeMemory, WTSLogoffSession, WTSOpenServerW,
            WTSQuerySessionInformationW, WTSSendMessageW, WTSUserName, WTS_CONNECTSTATE_CLASS,
            WTS_INFO_CLASS, WTS_SESSION_INFOW,
        };
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK};

        pub struct Session {
            pub server: String,
            pub user: String,
            pub id: u32,
            pub state: String,
        }

        fn wide(s: &str) -> Vec<u16> {
            OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        fn state_name(state: WTS_CONNECTSTATE_CLASS) -> String {
            match state {
                0 => "Active".into(),
                1 => "Connected".into(),
                2 => "ConnectQuery".into(),
                3 => "Shadow".into(),
                4 => "Disconnected".into(),
                5 => "Idle".into(),
                6 => "Listen".into(),
                7 => "Reset".into(),
                8 => "Down".into(),
                9 => "Init".into(),
                _ => format!("Unknown({state})"),
            }
        }

        pub fn list_sessions(server: &str) -> Result<Vec<Session>, String> {
            unsafe {
                let name = wide(server);
                let handle = WTSOpenServerW(name.as_ptr());
                if handle.is_null() {
                    return Err(format!("WTSOpenServer failed, error {}", GetLastError()));
                }
                let mut info: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
                let mut count = 0u32;
                let ok = WTSEnumerateSessionsW(handle, 0, 1, &mut info, &mut count);
                if ok == 0 {
                    let err = GetLastError();
                    WTSCloseServer(handle);
                    return Err(format!("WTSEnumerateSessions failed, error {err}"));
                }
                let mut out = Vec::with_capacity(count as usize);
                for i in 0..count as usize {
                    let item = &*info.add(i);
                    let id = item.SessionId;
                    let user = query_string(handle, id, WTSUserName).unwrap_or_default();
                    let domain = query_string(
                        handle,
                        id,
                        windows_sys::Win32::System::RemoteDesktop::WTSDomainName,
                    )
                    .unwrap_or_default();
                    let user = if domain.is_empty() || user.is_empty() {
                        user
                    } else {
                        format!("{domain}\\{user}")
                    };
                    out.push(Session {
                        server: server.to_string(),
                        user,
                        id,
                        state: state_name(item.State),
                    });
                }
                WTSFreeMemory(info as *mut _);
                WTSCloseServer(handle);
                Ok(out)
            }
        }

        unsafe fn query_string(
            handle: HANDLE,
            session: u32,
            class: WTS_INFO_CLASS,
        ) -> Option<String> {
            let mut buf: *mut u16 = std::ptr::null_mut();
            let mut bytes = 0u32;
            let ok = WTSQuerySessionInformationW(handle, session, class, &mut buf, &mut bytes);
            if ok == 0 || buf.is_null() {
                return None;
            }
            let len = (bytes as usize / 2).saturating_sub(1);
            let slice = std::slice::from_raw_parts(buf, len);
            let text = String::from_utf16_lossy(slice);
            WTSFreeMemory(buf as *mut _);
            Some(text)
        }

        pub fn session_user(server: &str, session: u32) -> Result<String, String> {
            unsafe {
                let name = wide(server);
                let handle = WTSOpenServerW(name.as_ptr());
                if handle.is_null() {
                    return Err("WTSOpenServer failed".into());
                }
                let user = query_string(handle, session, WTSUserName).unwrap_or_default();
                let domain = query_string(
                    handle,
                    session,
                    windows_sys::Win32::System::RemoteDesktop::WTSDomainName,
                )
                .unwrap_or_default();
                WTSCloseServer(handle);
                if user.is_empty() {
                    return Err(format!("session {session} has no user"));
                }
                Ok(if domain.is_empty() {
                    user
                } else {
                    format!("{domain}\\{user}")
                })
            }
        }

        pub fn send_message(
            server: &str,
            session: u32,
            title: &str,
            text: &str,
        ) -> Result<(), String> {
            unsafe {
                let name = wide(server);
                let handle = WTSOpenServerW(name.as_ptr());
                if handle.is_null() {
                    return Err("WTSOpenServer failed".into());
                }
                let title = wide(title);
                let text = wide(text);
                let mut response: i32 = 0;
                let ok = WTSSendMessageW(
                    handle,
                    session,
                    title.as_ptr(),
                    (title.len() as u32 - 1) * 2,
                    text.as_ptr(),
                    (text.len() as u32 - 1) * 2,
                    MB_OK | MB_ICONINFORMATION,
                    30,
                    &mut response,
                    0,
                );
                let err = GetLastError();
                WTSCloseServer(handle);
                if ok == 0 {
                    Err(format!("WTSSendMessage failed, error {err}"))
                } else {
                    Ok(())
                }
            }
        }

        pub fn logoff(server: &str, session: u32) -> Result<(), String> {
            unsafe {
                let name = wide(server);
                let handle = WTSOpenServerW(name.as_ptr());
                if handle.is_null() {
                    return Err("WTSOpenServer failed".into());
                }
                let ok = WTSLogoffSession(handle, session, 1);
                let err = GetLastError();
                WTSCloseServer(handle);
                if ok == 0 {
                    Err(format!("WTSLogoffSession failed, error {err}"))
                } else {
                    Ok(())
                }
            }
        }

        pub unsafe fn current_user_sid() -> Option<String> {
            let mut token: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return None;
            }
            let mut len = 0u32;
            GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut len);
            if (GetLastError() as u32) != ERROR_INSUFFICIENT_BUFFER || len == 0 {
                windows_sys::Win32::Foundation::CloseHandle(token);
                return None;
            }
            let mut buf = vec![0u8; len as usize];
            let ok =
                GetTokenInformation(token, TokenUser, buf.as_mut_ptr() as *mut _, len, &mut len);
            windows_sys::Win32::Foundation::CloseHandle(token);
            if ok == 0 {
                return None;
            }
            let user = &*(buf.as_ptr() as *const TOKEN_USER);
            let mut raw: *mut u16 = std::ptr::null_mut();
            if ConvertSidToStringSidW(user.User.Sid, &mut raw) == 0 {
                return None;
            }
            let mut len = 0usize;
            while *raw.add(len) != 0 {
                len += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(raw, len));
            LocalFree(raw as HLOCAL);
            Some(s)
        }

        pub unsafe fn in_group(name: &str) -> bool {
            let wide_name = wide(name);
            let mut sid_size = 0u32;
            let mut domain_size = 0u32;
            let mut use_type = 0i32;
            LookupAccountNameW(
                std::ptr::null(),
                wide_name.as_ptr(),
                std::ptr::null_mut(),
                &mut sid_size,
                std::ptr::null_mut(),
                &mut domain_size,
                &mut use_type,
            );
            if (GetLastError() as u32) != ERROR_INSUFFICIENT_BUFFER || sid_size == 0 {
                return false;
            }
            let mut sid = vec![0u8; sid_size as usize];
            let mut domain = vec![0u16; domain_size as usize];
            let ok = LookupAccountNameW(
                std::ptr::null(),
                wide_name.as_ptr(),
                sid.as_mut_ptr() as *mut _,
                &mut sid_size,
                domain.as_mut_ptr(),
                &mut domain_size,
                &mut use_type,
            );
            if ok == 0 {
                return false;
            }
            let mut is_member: BOOL = 0;
            if CheckTokenMembership(std::ptr::null_mut(), sid.as_ptr() as *mut _, &mut is_member)
                == 0
            {
                return false;
            }
            is_member != 0
        }
    }
}
