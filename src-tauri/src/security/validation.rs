//! Validation of model output before it can become a file operation.

const MAX_FOLDER_NAME_CHARS: usize = 64;

/// Turns a model-proposed category into a safe single-component folder name,
/// or `None` if it cannot be made safe.
pub fn sanitize_folder_name(raw: &str) -> Option<String> {
    let name = raw.trim();
    if name.is_empty()
        || name.starts_with('.')
        || name.chars().count() > MAX_FOLDER_NAME_CHARS
        || name
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '\0'))
    {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_ordinary_names() {
        assert_eq!(sanitize_folder_name("  Business "), Some("Business".into()));
        assert_eq!(
            sanitize_folder_name("Lecture Notes"),
            Some("Lecture Notes".into())
        );
    }

    #[test]
    fn rejects_unsafe_names() {
        for bad in ["", "  ", ".", "..", ".hidden", "a/b", "..\\x", "a:b", "x\ny"] {
            assert_eq!(sanitize_folder_name(bad), None, "{bad:?}");
        }
        assert_eq!(sanitize_folder_name(&"a".repeat(65)), None);
    }
}
