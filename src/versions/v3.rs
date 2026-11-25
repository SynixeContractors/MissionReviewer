use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::{description::name_summary_author, time::time},
};

pub fn check(
    dir: &Path,
    mission: (&Processed, &Config),
    config: (&Processed, &Config),
) -> Result<Vec<Annotation>, String> {
    let mut messages = vec![];
    // These files in templates should be untouched
    if !dir
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("TT"))
    {
        messages.append(&mut name_summary_author(dir, config));
        messages.append(&mut briefing(dir));
    }
    messages.append(&mut time(dir, mission, config));
    Ok(messages)
}

/// Discord messages are limited to 2000 characters, leave some room for formatting
const DISCORD_LIMIT: usize = 1960;

fn briefing(dir: &Path) -> Vec<Annotation> {
    let mut messages = vec![];
    let briefing_path = dir.join("edit_me").join("briefing");
    let Ok(files) = briefing_path.read_dir() else {
        messages.push(Annotation::new(
            None,
            briefing_path.display().to_string(),
            0..0,
            "Briefing folder is missing".to_string(),
            Level::Error,
        ));
        return messages;
    };
    let mut files = files
        .filter_map(Result::ok)
        .map(|f| f.path())
        .filter(|p| p.is_file())
        .collect::<Vec<_>>();
    files.sort();
    for path in files {
        let display = path.display().to_string();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            messages.push(Annotation::new(
                None,
                display,
                0..0,
                "Briefing file is not an HTML file".to_string(),
                Level::Error,
            ));
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                messages.push(Annotation::new(
                    None,
                    display,
                    0..0,
                    format!("Briefing file could not be read: {}", e),
                    Level::Error,
                ));
                continue;
            }
        };
        if let Some(pos) = content.find("INSERT") {
            messages.push(Annotation::new(
                None,
                display.clone(),
                pos..pos + "INSERT".len(),
                "Briefing file is not edited".to_string(),
                Level::Error,
            ));
        }
        let length = content.chars().count();
        if length > DISCORD_LIMIT {
            messages.push(Annotation::new(
                None,
                display.clone(),
                0..0,
                format!(
                    "Briefing {} is too long for Discord ({} characters, max {})",
                    path.display(),
                    length,
                    DISCORD_LIMIT
                ),
                Level::Error,
            ));
        }
        for pos in unescaped_ampersands(&content) {
            messages.push(Annotation::new(
                None,
                display.clone(),
                pos..pos + 1,
                "Briefing file contains an unescaped ampersand. Use &amp; instead.".to_string(),
                Level::Error,
            ));
        }
    }
    messages
}

/// Byte offsets of `&` that do not start an HTML entity like `&amp;` or `&#38;`
fn unescaped_ampersands(content: &str) -> Vec<usize> {
    content
        .match_indices('&')
        .filter(|(pos, _)| {
            let rest = &content[pos + 1..];
            let Some(end) = rest.find(';') else {
                return true;
            };
            let entity = &rest[..end];
            let valid = if let Some(number) = entity.strip_prefix('#') {
                if let Some(hex) = number.strip_prefix(['x', 'X']) {
                    !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit())
                } else {
                    !number.is_empty() && number.chars().all(|c| c.is_ascii_digit())
                }
            } else {
                !entity.is_empty() && entity.chars().all(|c| c.is_ascii_alphanumeric())
            };
            !valid
        })
        .map(|(pos, _)| pos)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::unescaped_ampersands;

    #[test]
    fn ampersands() {
        assert_eq!(unescaped_ampersands("Rock & Roll"), vec![5]);
        assert_eq!(unescaped_ampersands("R&R"), vec![1]);
        assert_eq!(
            unescaped_ampersands("Rock &amp; Roll &#38; &#x26;"),
            Vec::<usize>::new()
        );
        assert_eq!(unescaped_ampersands("a & b; c"), vec![2]);
        assert_eq!(unescaped_ampersands("trailing &"), vec![9]);
    }
}
