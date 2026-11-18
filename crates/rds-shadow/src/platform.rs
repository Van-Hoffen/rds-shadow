use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::{Command, ExitStatus};

use rds_shadow_core::config::{Config, ConnectionMode};
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
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub server: String,
    pub user: String,
    pub id: u32,
    pub state: String,
}

impl Session {
    pub fn is_active(&self) -> bool {
        self.state.eq_ignore_ascii_case("Active")
    }
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

pub fn list_all(hosts: &[String]) -> (Vec<Session>, Vec<String>) {
    let mut sessions = Vec::new();
    let mut errors = Vec::new();
    for host in hosts {
        match list_sessions(host) {
            Ok(mut found) => sessions.append(&mut found),
            Err(e) => errors.push(format!("{host}: {e}")),
        }
    }
    (sessions, errors)
}

unsafe fn query_string(handle: HANDLE, session: u32, class: WTS_INFO_CLASS) -> Option<String> {
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

pub fn send_message(server: &str, session: u32, title: &str, text: &str) -> Result<(), String> {
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

pub fn identity() -> (String, String) {
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

pub fn current_user_sid() -> String {
    unsafe { current_user_sid_inner().unwrap_or_default() }
}

unsafe fn current_user_sid_inner() -> Option<String> {
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
    let ok = GetTokenInformation(token, TokenUser, buf.as_mut_ptr() as *mut _, len, &mut len);
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

pub fn in_group(name: &str) -> bool {
    unsafe { in_group_inner(name) }
}

unsafe fn in_group_inner(name: &str) -> bool {
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
    if CheckTokenMembership(std::ptr::null_mut(), sid.as_ptr() as *mut _, &mut is_member) == 0 {
        return false;
    }
    is_member != 0
}

pub fn resolve_hosts(cfg: &Config) -> Result<Vec<String>, String> {
    match cfg.connection.mode {
        ConnectionMode::Servers => Ok(cfg.servers.clone()),
        ConnectionMode::Broker => {
            if cfg.servers.is_empty() {
                Err("broker mode is configured but broker discovery is not implemented in this build; fill [servers] list".into())
            } else {
                Ok(cfg.servers.clone())
            }
        }
    }
}

pub fn run_mstsc(server: &str, session: u32, control: bool) -> std::io::Result<ExitStatus> {
    let args = rds_shadow_core::command::mstsc_args(server, session, control);
    Command::new("mstsc.exe").args(&args).status()
}

/// Show a fatal error without a console window.
pub fn fatal(message: &str) {
    unsafe {
        let title = wide("RDS Shadow");
        let body = wide(message);
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
