//! Checks on the SQF code of a mission: script files, and code in `mission.sqm` attributes

use std::path::{Path, PathBuf};

use hemtt_config::{Class, Config};
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_class, get_string,
};

/// Commands that give players information they couldn't know: hints, chat and tasks
const MAGIC_INFORMATION: &[&str] = &[
    "hint",
    "hintC",
    "hintSilent",
    "systemChat",
    "createSimpleTask",
    "createTask",
    "BIS_fnc_taskCreate",
    "BIS_fnc_taskSetState",
    "BIS_fnc_setTask",
];

/// Commands and functions that create units
const SPAWNS_UNITS: &[&str] = &[
    "createUnit",
    "createVehicleCrew",
    "createAgent",
    "BIS_fnc_spawnGroup",
    "BIS_fnc_spawnVehicle",
];

/// Attributes in `mission.sqm` that hold code
const CODE_ATTRIBUTES: &[&str] = &[
    "init",
    "condition",
    "onActivation",
    "onDeactivation",
    "expCond",
    "expActiv",
];

/// Identifiers in SQF code with their byte offset, skipping comments and strings
pub fn identifiers(code: &str) -> Vec<(usize, &str)> {
    let bytes = code.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = code[i..].find('\n').map_or(bytes.len(), |n| i + n);
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = code[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |n| i + 2 + n + 2);
            }
            quote @ (b'"' | b'\'') => {
                // Strings end at the next quote, a doubled quote is an escaped quote
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == quote {
                        if bytes.get(i + 1) == Some(&quote) {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
                i += 1;
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                out.push((start, &code[start..i]));
            }
            c if c.is_ascii_digit() => {
                // Skip numbers, so `0x1F` or `1e5` aren't read as identifiers
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// Uses of any of the given commands (SQF is case insensitive)
fn find<'a>(code: &'a str, commands: &[&str]) -> Vec<(usize, &'a str)> {
    identifiers(code)
        .into_iter()
        .filter(|(_, ident)| commands.iter().any(|c| c.eq_ignore_ascii_case(ident)))
        .collect()
}

/// The mission's own script files, skipping `do_not_edit`
fn script_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                if entry.file_name() != "do_not_edit" {
                    walk(&path, out);
                }
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("sqf"))
            {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn magic_information_message(command: &str) -> String {
    format!(
        "`{command}` gives players information they couldn't know. Players learn things from the briefing, the world, and characters who speak to them (diag_log is fine for debugging)"
    )
}

/// Errors for hints, chat messages and tasks in the mission's script files
pub fn script_files_check(dir: &Path) -> Vec<Annotation> {
    let mut messages = Vec::new();
    for path in script_files(dir) {
        let Ok(code) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (pos, command) in find(&code, MAGIC_INFORMATION) {
            messages.push(Annotation::new(
                None,
                path.display().to_string(),
                pos..pos + command.len(),
                magic_information_message(command),
                Level::Error,
            ));
        }
    }
    messages
}

/// Whether any of the mission's scripts create units
pub fn spawns_units(dir: &Path) -> bool {
    script_files(dir).iter().any(|path| {
        std::fs::read_to_string(path).is_ok_and(|code| !find(&code, SPAWNS_UNITS).is_empty())
    })
}

/// Errors for hints, chat messages and tasks in code inside `mission.sqm`
#[derive(Default)]
pub struct SqmCode {
    messages: Vec<Annotation>,
}

impl SqmCode {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MissionCheck for SqmCode {
    fn object(&mut self, mission: (&Processed, &Config), dir: &Path, class: &Class, _: &str) {
        // Waypoints keep their code on the class, everything else in Attributes
        let attributes = get_class(class, "Attributes");
        for name in CODE_ATTRIBUTES {
            let Some((code, span)) = attributes
                .and_then(|a| get_string(a, name))
                .or_else(|| get_string(class, name))
            else {
                continue;
            };
            for (_, command) in find(code, MAGIC_INFORMATION) {
                self.messages.push(Annotation::new(
                    Some(mission.0),
                    dir.join("mission.sqm").display().to_string(),
                    span.clone(),
                    magic_information_message(command),
                    Level::Error,
                ));
            }
        }
    }

    fn done(&self, _: &Path) -> Vec<Annotation> {
        self.messages.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_comments_and_strings() {
        let code =
            "// hint \"a\"\n/* systemChat */ private _x = \"hint \"\"q\"\" \"; HINT 'x'; 0x1F;";
        let found = find(code, MAGIC_INFORMATION);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, "HINT");
        assert_eq!(&code[found[0].0..found[0].0 + 4], "HINT");
    }

    #[test]
    fn whole_words_only() {
        assert!(find(
            "_hints = 1; hintText = 2; mission_hint = 3;",
            MAGIC_INFORMATION
        )
        .is_empty());
        assert_eq!(
            find("[] call BIS_fnc_taskCreate", MAGIC_INFORMATION).len(),
            1
        );
    }
}
