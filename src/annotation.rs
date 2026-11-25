use std::{fmt::Display, path::PathBuf};

use hemtt_workspace::reporting::Processed;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Annotation for a CI environment
///
/// <https://github.com/actions/toolkit/tree/main/packages/core#annotations>
pub struct Annotation {
    /// The path of the file to annotate
    pub path: String,
    /// The start line of the annotation
    pub start_line: usize,
    /// The end line of the annotation
    pub end_line: usize,
    /// The start column of the annotation
    pub start_column: usize,
    /// The end column of the annotation
    pub end_column: usize,
    /// The annotation level
    pub level: Level,
    /// The annotation message
    pub message: String,
    /// The annotation title
    pub title: String,
}

impl Annotation {
    pub fn new(
        processed: Option<&Processed>,
        path: String,
        range: std::ops::Range<usize>,
        message: String,
        level: Level,
    ) -> Self {
        let start = range.start;
        let end = range.end;
        if let Some(processed) = processed {
            if let Some(start) = processed.mapping(start) {
                if let Some(end) = processed.mapping(end) {
                    return Self {
                        path,
                        start_line: start.original().start().line(),
                        end_line: end.original().end().line(),
                        start_column: start.original().start().column(),
                        end_column: end.original().end().column(),
                        level,
                        message,
                        title: String::new(),
                    };
                }
            }
        }
        let content = if PathBuf::from(&path).is_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    println!("failed to read {}: {}", path, e);
                    return Self {
                        path,
                        start_line: 1,
                        end_line: 1,
                        start_column: 1,
                        end_column: 1,
                        level,
                        message,
                        title: String::new(),
                    };
                }
            }
        } else {
            return Self {
                path,
                start_line: 1,
                end_line: 1,
                start_column: 1,
                end_column: 1,
                level,
                message,
                title: String::new(),
            };
        };
        let (start_line, start_column) = line_column(&content, start);
        let (end_line, end_column) = line_column(&content, end);
        Self {
            path,
            start_line,
            end_line,
            start_column,
            end_column,
            level,
            message,
            title: String::new(),
        }
    }

    #[must_use]
    /// Generate a line for the CI annotation
    ///
    /// Fields are separated by `||`, so newlines and `||` inside fields are escaped
    /// (`\\n` is turned back into a newline by the action).
    pub fn line(&self) -> String {
        format!(
            "{}||{}||{}||{}||{}||{}||{}||{}\n",
            self.start_line,
            self.end_line,
            self.start_column,
            self.end_column,
            self.level,
            escape(&self.title),
            escape(&self.message),
            escape(&self.path),
        )
    }
}

fn escape(field: &str) -> String {
    field
        .replace('\r', "")
        .replace('\n', "\\n")
        .replace("||", "|")
}

/// Convert a byte offset into a 1-based (line, column), column counted in characters
fn line_column(content: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(content.len());
    while !content.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &content[..offset];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let column = before[line_start..].chars().count() + 1;
    (line, column)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Annotation level
pub enum Level {
    /// Annotate a notice
    Notice,
    /// Annotate a warning
    Warning,
    /// Annotate an error
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notice => write!(f, "notice"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_column_uses_byte_offsets() {
        let content = "é\nab€c\n";
        assert_eq!(line_column(content, 0), (1, 1));
        assert_eq!(line_column(content, 3), (2, 1));
        // byte 5 is the start of '€', which is the 3rd character on line 2
        assert_eq!(line_column(content, 5), (2, 3));
        // offset inside '€' snaps back to its start
        assert_eq!(line_column(content, 6), (2, 3));
        assert_eq!(line_column(content, 100), (3, 1));
    }

    #[test]
    fn line_escapes_separators() {
        let annotation = Annotation {
            path: "a||b".to_string(),
            start_line: 1,
            end_line: 1,
            start_column: 1,
            end_column: 1,
            level: Level::Error,
            message: "one\r\ntwo".to_string(),
            title: String::new(),
        };
        assert_eq!(annotation.line(), "1||1||1||1||error||||one\\ntwo||a|b\n");
    }
}
