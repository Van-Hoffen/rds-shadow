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
}
