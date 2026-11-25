//! Human readable output for the command line

use std::{collections::HashMap, fmt::Write};

use crate::annotation::{Annotation, Level};

pub struct Style {
    color: bool,
}

impl Style {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn level(&self, level: &Level) -> String {
        match level {
            Level::Error => self.paint("1;31", "error"),
            Level::Warning => self.paint("1;33", "warning"),
            Level::Notice => self.paint("1;36", "notice"),
        }
    }

    fn gutter(&self, text: &str) -> String {
        self.paint("1;34", text)
    }
}

/// Source files, read once
#[derive(Default)]
pub struct Sources {
    files: HashMap<String, Option<Vec<String>>>,
}

impl Sources {
    fn line(&mut self, path: &str, line: usize) -> Option<&str> {
        self.files
            .entry(path.to_string())
            .or_insert_with(|| {
                std::fs::read_to_string(path).ok().map(|c| {
                    c.lines()
                        .map(|l| l.trim_end_matches('\r').to_string())
                        .collect()
                })
            })
            .as_ref()?
            .get(line.checked_sub(1)?)
            .map(String::as_str)
    }
}

/// Render one annotation, with the source line when it points somewhere in a file
pub fn render(annotation: &Annotation, sources: &mut Sources, style: &Style) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}: {}",
        style.level(&annotation.level),
        style.paint("1", &annotation.message)
    );
    // Annotations without a position point at the whole file
    let whole_file = annotation.start_line == 1
        && annotation.end_line == 1
        && annotation.start_column == 1
        && annotation.end_column == 1;
    if whole_file {
        let _ = writeln!(out, "  {} {}", style.gutter("-->"), annotation.path);
        return out;
    }
    let _ = writeln!(
        out,
        "  {} {}:{}:{}",
        style.gutter("-->"),
        annotation.path,
        annotation.start_line,
        annotation.start_column
    );
    let Some(source) = sources.line(&annotation.path, annotation.start_line) else {
        return out;
    };
    let number = annotation.start_line.to_string();
    let pad = " ".repeat(number.len());
    let chars = source.chars().collect::<Vec<_>>();
    let start = annotation.start_column.saturating_sub(1).min(chars.len());
    let end = if annotation.end_line == annotation.start_line {
        annotation.end_column.saturating_sub(1).min(chars.len())
    } else {
        chars.len()
    };
    let width = end.saturating_sub(start).max(1);
    // Keep tabs in the indent so the marker lines up with the source
    let indent = chars[..start]
        .iter()
        .map(|c| if *c == '\t' { '\t' } else { ' ' })
        .collect::<String>();
    let marker = match annotation.level {
        Level::Error => style.paint("1;31", &"^".repeat(width)),
        Level::Warning => style.paint("1;33", &"^".repeat(width)),
        Level::Notice => style.paint("1;36", &"^".repeat(width)),
    };
    let _ = writeln!(out, "{pad} {}", style.gutter("|"));
    let _ = writeln!(out, "{} {source}", style.gutter(&format!("{number} |")));
    let _ = writeln!(out, "{pad} {} {indent}{marker}", style.gutter("|"));
    out
}

/// Counts of errors and warnings
pub fn summary(annotations: &[Annotation]) -> (usize, usize) {
    let errors = annotations
        .iter()
        .filter(|a| a.level == Level::Error)
        .count();
    let warnings = annotations
        .iter()
        .filter(|a| a.level == Level::Warning)
        .count();
    (errors, warnings)
}

pub fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_snippet() {
        let path =
            std::env::temp_dir().join(format!("missionreviewer-report-{}.txt", std::process::id()));
        std::fs::write(&path, "first\r\n\thour=12;\r\n").unwrap();
        let annotation = Annotation {
            path: path.display().to_string(),
            start_line: 2,
            end_line: 2,
            start_column: 2,
            end_column: 9,
            level: Level::Error,
            message: "bad hour".to_string(),
            title: String::new(),
        };
        let out = render(&annotation, &mut Sources::default(), &Style::new(false));
        assert_eq!(
            out,
            format!(
                "error: bad hour\n  --> {}:2:2\n  |\n2 | \thour=12;\n  | \t^^^^^^^\n",
                path.display()
            )
        );
    }
}
