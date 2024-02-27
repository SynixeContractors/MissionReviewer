use std::fmt::Display;

use hemtt_common::reporting::Processed;

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
        let content = std::fs::read_to_string(&path).unwrap();
        let mut start_line = 1;
        let mut start_column = 1;
        let mut end_line = 1;
        let mut end_column = 1;
        let mut line_counter = 1;
        let mut column_counter = 1;

        for (i, c) in content.chars().enumerate() {
            if i == start {
                start_line = line_counter;
                start_column = column_counter;
            }
            if i == end {
                end_line = line_counter;
                end_column = column_counter;
            }
            if c == '\n' {
                line_counter += 1;
                column_counter = 1;
            } else {
                column_counter += 1;
            }
        }
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
    pub fn line(&self) -> String {
        format!(
            "{}||{}||{}||{}||{}||{}||{}||{}\n",
            self.start_line,
            self.end_line,
            self.start_column,
            self.end_column,
            self.level,
            self.title,
            self.message,
            self.path,
        )
    }
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
