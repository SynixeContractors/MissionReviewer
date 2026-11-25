use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::Annotation,
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
    }
    messages.append(&mut time(dir, mission, config));
    Ok(messages)
}
