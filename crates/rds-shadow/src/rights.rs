use std::process::Command;

/// One permission bit we expose in the editor. `id` is the `PermissionMask`
/// value accepted by `Win32_TSAccount.ModifyPermissions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perm {
    Query,
    Shadow,
    Message,
    Logoff,
}

impl Perm {
    pub const ALL: [Perm; 4] = [Perm::Query, Perm::Shadow, Perm::Message, Perm::Logoff];

    /// `ModifyPermissions` `PermissionMask` value.
    pub fn id(self) -> u32 {
        match self {
            Perm::Query => 0,   // WINSTATION_QUERY
            Perm::Shadow => 4,  // WINSTATION_SHADOW (Remote Control)
            Perm::Message => 7, // WINSTATION_MSG
            Perm::Logoff => 2,  // WINSTATION_LOGOFF
        }
    }

    /// Bit in `Win32_TSAccount.PermissionsAllowed`.
    pub fn bit(self) -> u32 {
        match self {
            Perm::Query => 1,     // WINSTATION_QUERY
            Perm::Shadow => 16,   // WINSTATION_SHADOW
            Perm::Message => 128, // WINSTATION_MSG
            Perm::Logoff => 4,    // WINSTATION_LOGOFF
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermAccount {
    pub name: String,
    pub sid: String,
    pub mask: u32,
}

impl PermAccount {
    pub fn allows(&self, perm: Perm) -> bool {
        self.mask & perm.bit() != 0
    }
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn run(script: &str) -> Result<String, String> {
    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("powershell: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !out.status.success() {
        return Err(format!(
            "powershell exit {}: {}",
            out.status,
            if stderr.is_empty() { stdout } else { stderr }
        ));
    }
    Ok(stdout)
}

fn cim_filter(server: &str, extra: Option<&str>) -> String {
    let mut s = String::new();
    if !server.trim().is_empty() {
        s.push_str(&format!(
            "$cimArgs.ComputerName = {}; ",
            ps_quote(server.trim())
        ));
    }
    if let Some(f) = extra {
        s.push_str(&format!("$cimArgs.Filter = {}; ", ps_quote(f)));
    }
    s
}

/// List accounts and their permission bitmasks on an RDP-Tcp listener.
pub fn list_accounts(server: &str) -> Result<Vec<PermAccount>, String> {
    let script = format!(
        "$cimArgs = @{{ Namespace = 'root\\CIMV2\\TerminalServices'; ClassName = 'Win32_TSAccount' }}; \
         $cimArgs.Filter = \"TerminalName='RDP-Tcp'\"; {} \
         Get-CimInstance @cimArgs | Select-Object AccountName, SID, PermissionsAllowed | ConvertTo-Json -Compress",
        cim_filter(server, None)
    );
    let stdout = run(&script)?;
    if stdout.is_empty() {
        return Ok(Vec::new());
    }
    let v: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| format!("parse: {e}"))?;
    let arr = match v {
        serde_json::Value::Array(a) => a,
        other => vec![other],
    };
    let mut out = Vec::new();
    for item in arr {
        let name = item
            .get("AccountName")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let sid = item
            .get("SID")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let mask = item
            .get("PermissionsAllowed")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as u32;
        out.push(PermAccount { name, sid, mask });
    }
    Ok(out)
}

/// Add an account entry (group/user/computer) to the listener.
pub fn add_account(server: &str, group: &str) -> Result<(), String> {
    let script = format!(
        "$cimArgs = @{{ Namespace = 'root\\CIMV2\\TerminalServices'; ClassName = 'Win32_TSPermissionsSetting' }}; \
         $cimArgs.Filter = \"TerminalName='RDP-Tcp'\"; {} \
         $tsp = Get-CimInstance @cimArgs; \
         $r = Invoke-CimMethod -InputObject $tsp -MethodName AddAccount -Arguments @{{ AccountName = {}; PermissionPreSet = 0 }}; \
         if ($r.ReturnValue -ne 0) {{ 'ERR:' + $r.ReturnValue }} else {{ 'OK' }}",
        cim_filter(server, None),
        ps_quote(group)
    );
    let stdout = run(&script)?;
    if stdout == "OK" {
        Ok(())
    } else {
        Err(format!("AddAccount: {stdout}"))
    }
}

/// Allow or deny one permission for an account.
pub fn set_permission(server: &str, group: &str, perm: Perm, allow: bool) -> Result<(), String> {
    let filter = format!(
        "TerminalName='RDP-Tcp' AND AccountName='{}'",
        group.replace('\'', "''")
    );
    let allow_s = if allow { "$true" } else { "$false" };
    let script = format!(
        "$cimArgs = @{{ Namespace = 'root\\CIMV2\\TerminalServices'; ClassName = 'Win32_TSAccount' }}; \
         $cimArgs.Filter = {}; {} \
         $acct = Get-CimInstance @cimArgs; \
         $r = Invoke-CimMethod -InputObject $acct -MethodName ModifyPermissions -Arguments @{{ PermissionMask = {}; Allow = {} }}; \
         if ($r.ReturnValue -ne 0) {{ 'ERR:' + $r.ReturnValue }} else {{ 'OK' }}",
        ps_quote(&filter),
        cim_filter(server, None),
        perm.id(),
        allow_s
    );
    let stdout = run(&script)?;
    if stdout == "OK" {
        Ok(())
    } else {
        Err(format!("ModifyPermissions: {stdout}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_ids_and_bits_are_stable() {
        assert_eq!(Perm::Query.id(), 0);
        assert_eq!(Perm::Shadow.id(), 4);
        assert_eq!(Perm::Message.id(), 7);
        assert_eq!(Perm::Logoff.id(), 2);
        assert_eq!(Perm::Query.bit(), 1);
        assert_eq!(Perm::Shadow.bit(), 16);
        assert_eq!(Perm::Message.bit(), 128);
        assert_eq!(Perm::Logoff.bit(), 4);
    }

    #[test]
    fn view_mask_is_query_shadow_message() {
        let view = Perm::Query.bit() | Perm::Shadow.bit() | Perm::Message.bit();
        assert_eq!(view, 1 | 16 | 128);
    }

    #[test]
    fn ps_quote_escapes_single_quotes() {
        assert_eq!(
            ps_quote("DOMAIN\\RDS-Shadow-View"),
            "'DOMAIN\\RDS-Shadow-View'"
        );
        assert_eq!(ps_quote("a'b"), "'a''b'");
    }
}
