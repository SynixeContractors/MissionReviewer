use std::{path::Path, sync::Arc};

use hemtt_common::config::{PDriveOption, PreprocessorOptions};
use hemtt_config::ConfigReport;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{
    reporting::{Code, Processed, WorkspaceFiles},
    LayerType, Workspace, WorkspacePath,
};

use crate::{
    annotation::{Annotation, Level},
    checks::{
        objects::{
            cover_map::CoverMapCheck, cup_parking::CUPParking, debug_units::DebugUnits,
            hostiles::HostilesCheck, players::PlayerCheck, shops::ShopCheck,
            spawners::SpawnersCheck, spectator::RequireSpectator, trigger::TriggerChecks,
            zeus::ZeusModule,
        },
        run_checks,
        scripts::{self, SqmCode},
        MissionCheck,
    },
    get_class, get_number, versions,
};

pub fn check(dir: &Path) -> Result<Vec<Annotation>, String> {
    let mut messages = vec![];
    let (mission_processed, mission) = match read_mission(dir) {
        Ok(config) => config,
        Err(errors) => {
            return Ok(errors);
        }
    };
    let Description {
        version,
        skip_lobby,
        processed: config_processed,
        config,
    } = match read_description(dir) {
        Ok(description) => description,
        Err(errors) => {
            return Ok(errors);
        }
    };
    let edit_me_description = dir
        .join("edit_me")
        .join("description.ext")
        .display()
        .to_string();
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
                None,
                dir.join("description.ext").display().to_string(),
                0..0,
                format!("Unknown synixe_template {}", version),
                Level::Error,
            ));
        }
    }

    let mut checks: Vec<Box<dyn MissionCheck>> = vec![
        Box::new(TriggerChecks::new()),
        Box::new(CUPParking::new()),
        Box::new(ZeusModule::new()),
        Box::new(DebugUnits::new()),
        Box::new(CoverMapCheck::new()),
        Box::new(HostilesCheck::new(
            get_class(mission.config(), "Mission.Intel")
                .and_then(|intel| get_number(intel, "resistanceWest").map(|(v, _)| v))
                // Eden leaves out default values, and the default is Independent friendly to West
                .unwrap_or(1),
            scripts::spawns_units(dir),
        )),
        Box::new(SqmCode::new()),
    ];
    messages.extend(scripts::script_files_check(dir));
    // 0: Contract, 1: Sub-Contract, 2: Training, 3: Special
    match get_number(config.config(), "synixe_type") {
        Some((0..=2, _)) if version >= 3 && !skip_lobby => {
            messages.push(Annotation::new(
                None,
                dir.join("do_not_edit")
                    .join("description.ext")
                    .display()
                    .to_string(),
                0..0,
                "do_not_edit is out of date (missing skipLobby = 1). Copy do_not_edit from a recent mission"
                    .to_string(),
                Level::Error,
            ));
        }
        _ => {}
    }
    match get_number(config.config(), "synixe_type") {
        Some((0 | 1, _)) => {
            checks.push(Box::new(PlayerCheck::new(dir, true)));
            checks.push(Box::new(SpawnersCheck::new(
                true,
                version,
                get_number(config.config(), "synixe_no_vehicles")
                    .map(|(v, _)| v == 1)
                    .unwrap_or_default(),
            )));
            checks.push(Box::new(ShopCheck::new()));
            checks.push(Box::new(RequireSpectator::new()));
        }
        Some((2, _)) => {
            checks.push(Box::new(PlayerCheck::new(dir, true)));
            checks.push(Box::new(SpawnersCheck::new(false, version, false)));
            checks.push(Box::new(ShopCheck::new()));
        }
        Some((3, _)) => {
            checks.push(Box::new(PlayerCheck::new(dir, false)));
            checks.push(Box::new(SpawnersCheck::new(false, version, false)));
        }
        Some((synixe_type, span)) => {
            messages.push(Annotation::new(
                Some(&config_processed),
                edit_me_description,
                span,
                format!("Unknown synixe_type {}", synixe_type),
                Level::Error,
            ));
        }
        None => {
            messages.push(Annotation::new(
                Some(&config_processed),
                edit_me_description,
                0..0,
                "synixe_type is missing or not a number".to_string(),
                Level::Error,
            ));
        }
    }
    messages.extend(run_checks(
        dir,
        checks,
        (&mission_processed, mission.config()),
    ));
    Ok(messages)
}

/// Check the mission folder name has an allowed prefix, nested is for missions in theatres
pub fn check_prefix(dir: &Path, nested: bool) -> Vec<Annotation> {
    let allowed: &[&str] = if nested {
        &["CCO", "TCO", "TT"]
    } else {
        &["CO", "SCO", "TRA", "SPC"]
    };
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if allowed.iter().any(|prefix| name.starts_with(prefix)) {
        return Vec::new();
    }
    vec![Annotation::new(
        None,
        dir.display().to_string(),
        0..0,
        format!(
            "Invalid prefix for {}, expected one of: {}",
            dir.display(),
            allowed.join(", ")
        ),
        Level::Error,
    )]
}

pub struct Description {
    /// `synixe_template`
    pub version: u8,
    /// Whether `skipLobby = 1` is set by the template
    pub skip_lobby: bool,
    /// `edit_me/description.ext`
    pub processed: Processed,
    /// `edit_me/description.ext`
    pub config: ConfigReport,
}

pub fn read_description(dir: &Path) -> Result<Description, Vec<Annotation>> {
    let description = dir.join("description.ext");
    let display = description.display().to_string();
    if !description.is_file() {
        return Err(vec![error(&display, "`description.ext` is missing")]);
    }
    if let Err(e) = std::fs::read_to_string(&description) {
        return Err(vec![error(
            &display,
            &format!("`description.ext` is invalid: {}", e),
        )]);
    };
    let workspace = workspace(dir, &display)?;
    let root = parse(
        &workspace,
        &["description.ext"],
        &display,
        "description.ext",
    )?;
    let version = match get_number(root.1.config(), "synixe_template") {
        Some((value, _)) => match u8::try_from(value) {
            Ok(value) => value,
            Err(_) => {
                return Err(vec![error(
                    &display,
                    &format!("Unknown synixe_template {}", value),
                )])
            }
        },
        None => 2,
    };
    let edit_me_display = dir
        .join("edit_me")
        .join("description.ext")
        .display()
        .to_string();
    let (processed, config) = parse(
        &workspace,
        &["edit_me", "description.ext"],
        &edit_me_display,
        "edit_me/description.ext",
    )?;
    Ok(Description {
        version,
        skip_lobby: matches!(get_number(root.1.config(), "skipLobby"), Some((1, _))),
        processed,
        config,
    })
}

pub fn read_mission(dir: &Path) -> Result<(Processed, ConfigReport), Vec<Annotation>> {
    let mission = dir.join("mission.sqm");
    let display = mission.display().to_string();
    if !mission.is_file() {
        return Err(vec![error(&display, "`mission.sqm` is missing")]);
    }
    if std::fs::read_to_string(&mission).is_err() {
        return Err(vec![error(
            &display,
            "`mission.sqm` is binarized or invalid",
        )]);
    };
    let workspace = workspace(dir, &display)?;
    parse(&workspace, &["mission.sqm"], &display, "mission.sqm")
}

fn error(path: &str, message: &str) -> Annotation {
    Annotation::new(
        None,
        path.to_string(),
        0..0,
        message.to_string(),
        Level::Error,
    )
}

fn workspace(dir: &Path, display: &str) -> Result<WorkspacePath, Vec<Annotation>> {
    Workspace::builder()
        .physical(&dir.to_path_buf(), LayerType::Source)
        .finish(None, false, &PDriveOption::Disallow)
        .map_err(|e| {
            vec![error(
                display,
                &format!("Failed to read mission folder: {}", e),
            )]
        })
}

fn describe(code: &Arc<dyn Code>) -> String {
    code.diagnostic()
        .map(|d| d.to_string(&WorkspaceFiles::new()))
        .unwrap_or_else(|| code.message())
}

/// Preprocess and parse a config file in the workspace
fn parse(
    workspace: &WorkspacePath,
    path: &[&str],
    display: &str,
    name: &str,
) -> Result<(Processed, ConfigReport), Vec<Annotation>> {
    let mut file = workspace.clone();
    for part in path {
        file = file
            .join(part)
            .map_err(|e| vec![error(display, &format!("`{}` is invalid: {}", name, e))])?;
    }
    let processed = match Processor::run(&file, &PreprocessorOptions::default()) {
        Ok(processed) => processed,
        Err((_, hemtt_preprocessor::Error::Code(e))) => {
            return Err(vec![error(
                display,
                &format!("`{}` failed to process: {}", name, describe(&e)),
            )]);
        }
        Err((_, e)) => {
            return Err(vec![error(
                display,
                &format!("`{}` failed to process: {}", name, e),
            )]);
        }
    };
    match hemtt_config::parse(None, &processed) {
        Ok(config) => Ok((processed, config)),
        Err(codes) => Err(codes
            .iter()
            .map(|e| {
                error(
                    display,
                    &format!("`{}` failed to process: {}", name, describe(e)),
                )
            })
            .collect()),
    }
}
