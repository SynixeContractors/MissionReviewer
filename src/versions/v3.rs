use std::path::Path;

use hemtt_common::reporting::Processed;
use hemtt_config::Config;

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
    if !dir.file_name().unwrap().to_str().unwrap().starts_with("TT") {
        messages.append(&mut name_summary_author(dir, config));
        messages.append(&mut briefing(dir));
    }
    messages.append(&mut time(dir, mission, config));
    Ok(messages)
}

fn briefing(dir: &Path) -> Vec<Annotation> {
    let mut messages = vec![];
    let briefing_path = dir.join("edit_me").join("briefing");
    let mut combined_briefing_length = 0;
    for file in briefing_path.read_dir().unwrap() {
        let file = file.unwrap();
        if !file.file_type().unwrap().is_file() {
            continue;
        }
        let path = file.path();
        if let Some(ext) = path.extension() {
            if ext != "html" {
                messages.push(Annotation::new(
                    None,
                    path.display().to_string(),
                    0..0,
                    "Briefing file is not an HTML file".to_string(),
                    Level::Error,
                ));
            }
            let content = std::fs::read_to_string(&path).unwrap();
            if content.contains("INSERT") {
                messages.push(Annotation::new(
                    None,
                    path.display().to_string(),
                    0..0,
                    "Briefing file is not edited".to_string(),
                    Level::Error,
                ));
            }
        }
        match path.file_name().unwrap().to_str().unwrap() {
            "mission.html" | "objectives.html" => {
                let content = std::fs::read_to_string(&path).unwrap();
                combined_briefing_length += content.len();
            }
            "situation.html" => {
                let content = std::fs::read_to_string(&path).unwrap();
                if content.len() > 1900 {
                    messages.push(Annotation::new(
                        None,
                        path.display().to_string(),
                        0..0,
                        "Situation briefing is too long for Discord".to_string(),
                        Level::Error,
                    ));
                }
            }
            _ => {}
        }
    }
    if combined_briefing_length > 1900 {
        messages.push(Annotation::new(
            None,
            briefing_path.display().to_string(),
            0..0,
            "Combined briefing (mission & objectives) is too long for Discord".to_string(),
            Level::Error,
        ));
    }
    messages
}
