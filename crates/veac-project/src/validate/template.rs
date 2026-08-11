use crate::{IssueCode, ProjectIssue, ProjectTarget, TargetInstance};

use super::primitives::valid_path;

pub(crate) fn validate_template(
    value: &str,
    target: &ProjectTarget,
    has_default_profile: bool,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let placeholders = match placeholders(value) {
        Ok(placeholders) => placeholders,
        Err(message) => {
            issues.push(ProjectIssue::new(IssueCode::InvalidTemplate, path, message));
            return;
        }
    };
    let mut valid = true;
    for placeholder in &placeholders {
        let known = match placeholder.as_str() {
            "target" => true,
            "profile" => !target.profiles.is_empty() || has_default_profile,
            "locale" => target.localized,
            _ => placeholder
                .strip_prefix("axis.")
                .is_some_and(|axis| target.axes.iter().any(|item| item.id.as_str() == axis)),
        };
        if !known {
            valid = false;
            issues.push(ProjectIssue::new(
                IssueCode::InvalidTemplate,
                path,
                format!("placeholder {{{placeholder}}} is not available for this target"),
            ));
        }
    }
    let sample = replace(value, |placeholder| {
        if placeholder.starts_with("axis.")
            || matches!(placeholder, "target" | "profile" | "locale")
        {
            Some("value")
        } else {
            None
        }
    });
    if valid
        && sample
            .as_deref()
            .is_none_or(|path| !valid_path(path, false))
    {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidTemplate,
            path,
            "delivery template must resolve to a canonical project-relative path",
        ));
    }
}

pub(crate) fn expand_template(value: &str, instance: &TargetInstance) -> Option<String> {
    replace(value, |placeholder| match placeholder {
        "target" => Some(instance.target.as_str()),
        "profile" => instance.profile.as_ref().map(|value| value.as_str()),
        "locale" => instance.locale.as_ref().map(|value| value.as_str()),
        _ => placeholder
            .strip_prefix("axis.")
            .and_then(|id| instance.matrix.iter().find(|(key, _)| key.as_str() == id))
            .map(|(_, value)| value.as_str()),
    })
}

fn placeholders(value: &str) -> Result<Vec<String>, String> {
    let mut output = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find(['{', '}']) {
        if rest.as_bytes()[open] == b'}' {
            return Err("delivery template contains an unmatched closing brace".to_owned());
        }
        let after = &rest[open + 1..];
        let close = after
            .find('}')
            .ok_or_else(|| "delivery template contains an unmatched opening brace".to_owned())?;
        let placeholder = &after[..close];
        if placeholder.is_empty() || placeholder.contains('{') {
            return Err("delivery template contains a malformed placeholder".to_owned());
        }
        output.push(placeholder.to_owned());
        rest = &after[close + 1..];
    }
    Ok(output)
}

fn replace<'a>(value: &'a str, mut resolve: impl FnMut(&str) -> Option<&'a str>) -> Option<String> {
    let names = placeholders(value).ok()?;
    let mut output = value.to_owned();
    for name in names {
        let replacement = resolve(&name)?;
        output = output.replace(&format!("{{{name}}}"), replacement);
    }
    Some(output)
}
