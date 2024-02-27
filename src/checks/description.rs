use std::path::Path;

use hemtt_common::reporting::Processed;
use hemtt_config::{Config, Property, Value};

use crate::annotation::{Annotation, Level};

pub fn name_summary_author(dir: &Path, config: (&Processed, &Config)) -> Vec<Annotation> {
    let path = dir.to_path_buf().join("edit_me").join("description.ext");
    let mut messages = vec![];
    let mut found_name = false;
    let mut found_summary = false;
    let mut found_author = false;
    config.1 .0.iter().for_each(|node| {
        if let Property::Entry {
            name, value: entry, ..
        } = node
        {
            if name.as_str() == "OnLoadName" {
                found_name = true;
                if let Value::Str(value) = entry {
                    if value.value() == "MISSION NAME" {
                        messages.push(Annotation::new(
                            Some(config.0),
                            path.display().to_string(),
                            entry.span(),
                            "OnLoadName is not set".to_string(),
                            Level::Error,
                        ));
                    }
                } else {
                    messages.push(Annotation::new(
                        Some(config.0),
                        path.display().to_string(),
                        entry.span(),
                        "OnLoadName is not a string".to_string(),
                        Level::Error,
                    ));
                }
            }
            if name.as_str() == "OnLoadMission" {
                found_summary = true;
                if let Value::Str(summary) = entry {
                    if summary.value() == "MISSION SUMMARY" {
                        messages.push(Annotation::new(
                            Some(config.0),
                            path.display().to_string(),
                            entry.span(),
                            "OnLoadMission is not set".to_string(),
                            Level::Error,
                        ));
                    }
                    if summary.value().ends_with('.') {
                        messages.push(Annotation::new(
                            Some(config.0),
                            path.display().to_string(),
                            entry.span(),
                            "OnLoadMission ends with a period".to_string(),
                            Level::Warning,
                        ));
                    } else if summary.value().contains(". ") {
                        messages.push(Annotation::new(
                            Some(config.0),
                            path.display().to_string(),
                            entry.span(),
                            "OnLoadMission should be a single sentence".to_string(),
                            Level::Warning,
                        ));
                    }
                } else {
                    messages.push(Annotation::new(
                        Some(config.0),
                        path.display().to_string(),
                        entry.span(),
                        "OnLoadMission is not a string".to_string(),
                        Level::Error,
                    ));
                }
            }
            if name.as_str() == "author" {
                found_author = true;
                if let Value::Str(author) = entry {
                    if author.value().starts_with("YOUR NAME") {
                        messages.push(Annotation::new(
                            Some(config.0),
                            path.display().to_string(),
                            entry.span(),
                            "author is not set".to_string(),
                            Level::Error,
                        ));
                    }
                } else {
                    messages.push(Annotation::new(
                        Some(config.0),
                        path.display().to_string(),
                        entry.span(),
                        "author is not a string".to_string(),
                        Level::Error,
                    ));
                }
            }
        }
    });
    if !found_name {
        messages.push(Annotation::new(
            Some(config.0),
            path.display().to_string(),
            0..0,
            "OnLoadName is missing".to_string(),
            Level::Error,
        ));
    }
    if !found_summary {
        messages.push(Annotation::new(
            Some(config.0),
            path.display().to_string(),
            0..0,
            "OnLoadMission is missing".to_string(),
            Level::Error,
        ));
    }
    if !found_author {
        messages.push(Annotation::new(
            Some(config.0),
            path.display().to_string(),
            0..0,
            "author is missing".to_string(),
            Level::Error,
        ));
    }
    messages
}
