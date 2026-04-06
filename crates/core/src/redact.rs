//! Secret redaction for command output.
//!
//! Mirrors GoReleaser's `internal/redact/redact.go`: scans environment
//! variables for secret-looking entries and replaces their values in
//! output strings with `$KEY_NAME`.

/// Key suffixes that indicate a secret value.
const SECRET_KEY_SUFFIXES: &[&str] = &["_KEY", "_SECRET", "_PASSWORD", "_TOKEN"];

/// Value prefixes that indicate a secret regardless of key name.
const SECRET_VALUE_PREFIXES: &[&str] = &[
    "sk-", "ghp_", "ghs_", "gho_", "ghu_", "dckr_pat_", "glpat-", "AIZA", "xox",
];

/// Minimum value length to consider for redaction.
const MIN_SECRET_LEN: usize = 10;

/// Returns true if this env entry looks like it contains a secret.
fn is_secret(key: &str, value: &str) -> bool {
    if value.len() < MIN_SECRET_LEN {
        return false;
    }
    let key_upper = key.to_uppercase();
    if SECRET_KEY_SUFFIXES.iter().any(|s| key_upper.ends_with(s)) {
        return true;
    }
    SECRET_VALUE_PREFIXES.iter().any(|p| value.starts_with(p))
}

/// Redact secret values in a string, replacing them with `$KEY_NAME`.
///
/// Longer values are replaced first to prevent partial matches.
pub fn redact_string(input: &str, env: &[(String, String)]) -> String {
    let mut secrets: Vec<(&str, &str)> = env
        .iter()
        .filter(|(k, v)| is_secret(k, v))
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    secrets.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let mut result = input.to_string();
    for (key, value) in secrets {
        result = result.replace(value, &format!("${}", key));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_by_key_suffix() {
        let env = vec![
            ("DOCKER_PASSWORD".to_string(), "mysecretpassword123".to_string()),
            ("PLAIN_VAR".to_string(), "not-a-secret".to_string()),
        ];
        let result = redact_string("Login with mysecretpassword123 succeeded", &env);
        assert_eq!(result, "Login with $DOCKER_PASSWORD succeeded");
        assert!(!result.contains("mysecretpassword123"));
    }

    #[test]
    fn test_redact_by_value_prefix() {
        let env = vec![
            ("MY_TOKEN".to_string(), "ghp_abc123def456ghi789".to_string()),
        ];
        let result = redact_string("Using token ghp_abc123def456ghi789", &env);
        assert_eq!(result, "Using token $MY_TOKEN");
    }

    #[test]
    fn test_redact_ignores_short_values() {
        let env = vec![
            ("API_KEY".to_string(), "short".to_string()),
        ];
        let result = redact_string("Value is short", &env);
        assert_eq!(result, "Value is short");
    }

    #[test]
    fn test_redact_longer_values_first() {
        let env = vec![
            ("SHORT_TOKEN".to_string(), "abcdefghij".to_string()),
            ("LONG_TOKEN".to_string(), "abcdefghijklmnop".to_string()),
        ];
        let result = redact_string("secret: abcdefghijklmnop", &env);
        // Longer match should be replaced first
        assert_eq!(result, "secret: $LONG_TOKEN");
    }

    #[test]
    fn test_redact_no_secrets() {
        let env = vec![
            ("PATH".to_string(), "/usr/bin:/usr/local/bin".to_string()),
        ];
        let result = redact_string("PATH is set", &env);
        assert_eq!(result, "PATH is set");
    }

    #[test]
    fn test_redact_multiple_occurrences() {
        let env = vec![
            ("REGISTRY_PASSWORD".to_string(), "supersecret123".to_string()),
        ];
        let result = redact_string("auth supersecret123 retry supersecret123", &env);
        assert_eq!(result, "auth $REGISTRY_PASSWORD retry $REGISTRY_PASSWORD");
    }

    #[test]
    fn test_is_secret_key_suffixes() {
        assert!(is_secret("DOCKER_PASSWORD", "longvalue1234"));
        assert!(is_secret("API_TOKEN", "longvalue1234"));
        assert!(is_secret("signing_key", "longvalue1234")); // case insensitive
        assert!(is_secret("MY_SECRET", "longvalue1234"));
        assert!(!is_secret("MY_CONFIG", "longvalue1234"));
    }

    #[test]
    fn test_is_secret_value_prefixes() {
        assert!(is_secret("ANYTHING", "ghp_1234567890"));
        assert!(is_secret("ANYTHING", "sk-1234567890"));
        assert!(is_secret("ANYTHING", "dckr_pat_1234567890"));
        assert!(is_secret("ANYTHING", "glpat-1234567890"));
        assert!(!is_secret("ANYTHING", "regular_value1234"));
    }
}
