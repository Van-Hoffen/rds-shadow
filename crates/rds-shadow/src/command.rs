use std::path::Path;

/// Build the `mstsc` arguments for a shadow session.
///
/// Contract: this function never emits a consent-bypass flag. `control` adds
/// keyboard/mouse input; without it the session is view-only.
pub fn mstsc_args(server: &str, session_id: u32, control: bool) -> Vec<String> {
    let mut args = vec![format!("/v:{server}"), format!("/shadow:{session_id}")];
    if control {
        args.push("/control".to_string());
    }
    args
}

/// Build the `.rdp` connection file for a shadow session that will be captured.
///
/// Windows.Graphics.Capture needs native pixels, so smart sizing is off
/// (`smart sizing:i:0`) and the session runs on a single surface
/// (`use multimon:i:0`). The file never enables a consent bypass and stores no
/// credentials. A non-default port is appended to the address.
pub fn rdp_file(server: &str, port: u16) -> String {
    let addr = if port == 3389 {
        server.to_string()
    } else {
        format!("{server}:{port}")
    };
    let mut out = String::new();
    out.push_str("screen mode id:i:1\n");
    out.push_str("use multimon:i:0\n");
    out.push_str("session bpp:i:32\n");
    out.push_str("smart sizing:i:0\n");
    out.push_str(&format!("full address:s:{addr}\n"));
    out
}

/// Build `mstsc` arguments that open a generated `.rdp` file as a shadow
/// session. `control` adds input; without it the session is view-only. Never
/// emits a consent-bypass flag.
pub fn mstsc_shadow_args(rdp_path: &Path, session_id: u32, control: bool) -> Vec<String> {
    let mut args = vec![
        rdp_path.to_string_lossy().into_owned(),
        format!("/shadow:{session_id}"),
    ];
    if control {
        args.push("/control".to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_has_no_control_and_no_consent_bypass() {
        let args = mstsc_args("ts03", 14, false);
        assert_eq!(args, vec!["/v:ts03", "/shadow:14"]);
        assert!(!args.iter().any(|a| a.eq_ignore_ascii_case("/control")));
        assert!(!args
            .iter()
            .any(|a| a.eq_ignore_ascii_case("/noConsentPrompt")));
    }

    #[test]
    fn control_is_opt_in_only() {
        let args = mstsc_args("ts03", 14, true);
        assert!(args.contains(&"/control".to_string()));
        assert!(!args
            .iter()
            .any(|a| a.eq_ignore_ascii_case("/noConsentPrompt")));
    }

    #[test]
    fn server_and_session_are_single_arguments() {
        let args = mstsc_args("ts03.example.domain", 7, false);
        assert_eq!(args[0], "/v:ts03.example.domain");
        assert_eq!(args[1], "/shadow:7");
    }

    #[test]
    fn rdp_file_is_native_single_surface_no_bypass() {
        let rdp = rdp_file("ts03", 3389);
        assert!(rdp.contains("screen mode id:i:1"));
        assert!(rdp.contains("use multimon:i:0"));
        assert!(rdp.contains("smart sizing:i:0"));
        assert!(rdp.contains("full address:s:ts03\n"));
        assert!(!rdp.contains("smart sizing:i:1"));
        assert!(!rdp.to_ascii_lowercase().contains("noconsentprompt"));
        assert!(!rdp.to_ascii_lowercase().contains("password"));
    }

    #[test]
    fn rdp_file_appends_non_default_port() {
        assert!(rdp_file("ts03", 3389).contains("full address:s:ts03\n"));
        let rdp = rdp_file("ts03", 3390);
        assert!(rdp.contains("full address:s:ts03:3390\n"));
    }

    #[test]
    fn shadow_args_open_file_and_opt_in_control() {
        let view = mstsc_shadow_args(Path::new("x.rdp"), 14, false);
        assert_eq!(view, vec!["x.rdp".to_string(), "/shadow:14".to_string()]);
        assert!(!view
            .iter()
            .any(|a| a.eq_ignore_ascii_case("/noConsentPrompt")));

        let ctl = mstsc_shadow_args(Path::new("x.rdp"), 7, true);
        assert!(ctl.contains(&"/control".to_string()));
        assert!(!ctl
            .iter()
            .any(|a| a.eq_ignore_ascii_case("/noConsentPrompt")));
    }
}
