use std::path::PathBuf;

use hemtt_common::{
    reporting::{Processed, WorkspaceFiles},
    workspace::{LayerType, Workspace},
};
use hemtt_config::{ConfigReport, Number, Property, Value};
use hemtt_preprocessor::Processor;

use crate::{
    annotation::{Annotation, Level},
    checks::objects::{
        players::PlayerCheck, run_over_entities, shops::ShopCheck, spawners::SpawnersCheck,
        spectator::RequireSpectator,
    },
    get_number, versions,
};

pub fn check(dir: &PathBuf) -> Result<Vec<Annotation>, String> {
    let mut messages = vec![];
    println!("Checking {}", dir.display());
    let (mission_processed, mission) = match read_mission(dir) {
        Ok(config) => config,
        Err(errors) => {
            return Err(errors.join("\n"));
        }
    };
    let (version, config_processed, config) = match read_description(dir) {
        Ok(config) => config,
        Err(errors) => {
            return Err(errors.join("\n"));
        }
    };
    match version {
        2 => {
            messages.append(&mut versions::v2::check(
                dir,
                (&mission_processed, mission.config()),
                (&config_processed, config.config()),
            )?);
        }
        3 => {
            messages.append(&mut versions::v3::check(
                dir,
                (&mission_processed, mission.config()),
                (&config_processed, config.config()),
            )?);
        }
        _ => {
            messages.push(Annotation::new(
                Some(&config_processed),
                dir.join("do_not_edit")
                    .join("description.ext")
                    .display()
                    .to_string(),
                0..0,
                format!("Unknown synixe_template {}", version),
                Level::Error,
            ));
        }
    }
    let checks = run_over_entities(
        dir,
        {
            let (synixe_type, synixe_type_span) =
                get_number(config.config(), "synixe_type").unwrap_or_default();
            // 0: Contract, 1: Sub-Contract, 2: Training, 3: Special
            match synixe_type {
                0 | 1 => {
                    vec![
                        Box::new(PlayerCheck::new(dir, true)),
                        Box::new(ShopCheck::new()),
                        Box::new(RequireSpectator::new()),
                        Box::new(SpawnersCheck::new(
                            true,
                            version,
                            get_number(config.config(), "synixe_no_vehicles")
                                .map(|(v, _)| v == 1)
                                .unwrap_or_default(),
                        )),
                    ]
                }
                2 => vec![
                    Box::new(PlayerCheck::new(dir, true)),
                    Box::new(ShopCheck::new()),
                    Box::new(SpawnersCheck::new(false, version, false)),
                ],
                3 => vec![
                    Box::new(SpawnersCheck::new(false, version, false)),
                    Box::new(PlayerCheck::new(dir, false)),
                ],
                _ => {
                    messages.push(Annotation::new(
                        Some(&config_processed),
                        dir.join("edit_me")
                            .join("description.ext")
                            .display()
                            .to_string(),
                        synixe_type_span,
                        format!("Unknown synixe_type {}", synixe_type),
                        Level::Error,
                    ));
                    vec![]
                }
            }
        },
        (&mission_processed, mission.config()),
    );
    messages.extend(checks);
    Ok(messages)
}

pub fn read_description(dir: &PathBuf) -> Result<(u8, Processed, ConfigReport), Vec<String>> {
    let description = dir.join("description.ext");
    if !description.is_file() {
        return Err(vec![format!(
            "{} is missing description.ext",
            dir.display()
        )]);
    }
    if std::fs::read_to_string(description).is_err() {
        return Err(vec![format!(
            "{} failed to read description.ext",
            dir.display()
        )]);
    };
    let workspace = Workspace::builder()
        .physical(dir, LayerType::Source)
        .finish(None, false)
        .expect("Failed to create workspace");
    let processed = match Processor::run(
        &workspace
            .join("description.ext")
            .expect("Failed to join path"),
    ) {
        Ok(processed) => processed,
        Err(hemtt_preprocessor::Error::Code(e)) => {
            return Err(vec![format!(
                "{} failed to process description.ext: {}",
                dir.display(),
                e.diagnostic()
                    .expect("diagnostic")
                    .to_string(&WorkspaceFiles::new())
            )])
        }
        Err(e) => {
            return Err(vec![format!(
                "{} failed to process description.ext: {}",
                dir.display(),
                e
            )])
        }
    };
    match hemtt_config::parse(None, &processed) {
        Ok(config) => {
            let version = match config.config().0.iter().find(|c| {
                if let Property::Entry {
                    name,
                    value: Value::Number(_),
                    ..
                } = c
                {
                    name.as_str() == "synixe_template"
                } else {
                    false
                }
            }) {
                Some(value) => {
                    let Property::Entry {
                        value: Value::Number(Number::Int32 { value, .. }),
                        ..
                    } = value
                    else {
                        panic!("Expected entry");
                    };
                    *value as u8
                }
                None => 2,
            };
            let processed = match Processor::run(
                &workspace
                    .join("edit_me")
                    .expect("Failed to join path")
                    .join("description.ext")
                    .expect("Failed to join path"),
            ) {
                Ok(processed) => processed,
                Err(hemtt_preprocessor::Error::Code(e)) => {
                    return Err(vec![format!(
                        "{} failed to process description.ext: {}",
                        dir.display(),
                        e.diagnostic()
                            .expect("diagnostic")
                            .to_string(&WorkspaceFiles::new())
                    )])
                }
                Err(e) => {
                    return Err(vec![format!(
                        "{} failed to process description.ext: {}",
                        dir.display(),
                        e
                    )])
                }
            };
            match hemtt_config::parse(None, &processed) {
                Ok(config) => Ok((version, processed, config)),
                Err(e) => Err(e
                    .iter()
                    .map(|e| {
                        format!(
                            "{}: {}",
                            dir.display(),
                            e.diagnostic()
                                .expect("diagnositc")
                                .to_string(&WorkspaceFiles::new())
                        )
                    })
                    .collect()),
            }
        }
        Err(e) => Err(e
            .iter()
            .map(|e| {
                format!(
                    "{}: {}",
                    dir.display(),
                    e.diagnostic()
                        .expect("diagnositc")
                        .to_string(&WorkspaceFiles::new())
                )
            })
            .collect()),
    }
}

pub fn read_mission(dir: &PathBuf) -> Result<(Processed, ConfigReport), Vec<String>> {
    let description = dir.join("mission.sqm");
    if !description.is_file() {
        return Err(vec![format!("{} is missing mission.sqm", dir.display())]);
    }
    if std::fs::read_to_string(description).is_err() {
        return Err(vec![format!(
            "{} failed to read mission.sqm",
            dir.display()
        )]);
    };
    let workspace = Workspace::builder()
        .physical(dir, LayerType::Source)
        .finish(None, false)
        .expect("Failed to create workspace");
    let processed =
        match Processor::run(&workspace.join("mission.sqm").expect("Failed to join path")) {
            Ok(processed) => processed,
            Err(hemtt_preprocessor::Error::Code(e)) => {
                return Err(vec![format!(
                    "{} failed to process mission.sqm: {}",
                    dir.display(),
                    e.diagnostic()
                        .expect("diagnostic")
                        .to_string(&WorkspaceFiles::new())
                )])
            }
            Err(e) => {
                return Err(vec![format!(
                    "{} failed to process mission.sqm: {}",
                    dir.display(),
                    e
                )])
            }
        };
    match hemtt_config::parse(None, &processed) {
        Ok(config) => Ok((processed, config)),
        Err(e) => Err(e
            .iter()
            .map(|e| {
                format!(
                    "{}: {}",
                    dir.display(),
                    e.diagnostic()
                        .expect("diagnositc")
                        .to_string(&WorkspaceFiles::new())
                )
            })
            .collect()),
    }
}
