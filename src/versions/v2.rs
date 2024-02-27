use std::path::Path;

use hemtt_common::reporting::Processed;
use hemtt_config::Config;

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
    if !dir.file_name().unwrap().to_str().unwrap().starts_with("TT") {
        messages.append(&mut name_summary_author(dir, config));
    }
    messages.append(&mut time(dir, mission, config));
    Ok(messages)
}
