use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_string,
};

pub struct SpawnersCheck {
    count_land: u32,
    count_air: u32,
    count_sea: u32,
    count_thing: u32,
    data_type: String,
    expected: bool,
    acknowledged_land: bool,
}

impl SpawnersCheck {
    pub fn new(expected: bool, version: u8, acknowledged_land: bool) -> Self {
        Self {
            count_land: 0,
            count_air: 0,
            count_sea: 0,
            count_thing: 0,
            data_type: if version == 3 {
                "Object".to_string()
            } else {
                "Marker".to_string()
            },
            expected,
            acknowledged_land,
        }
    }
}

impl MissionCheck for SpawnersCheck {
    fn object(
        &mut self,
        _: (&Processed, &Config),
        _: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != self.data_type {
            return;
        }
        match data_type {
            "Object" => {
                let Some((type_, _)) = get_string(class, "type") else {
                    return;
                };
                // TODO check distance between spawners
                match type_.as_str() {
                    "crate_client_garage_land_large"
                    | "crate_client_garage_land_medium"
                    | "crate_client_garage_land_small" => self.count_land += 1,
                    "crate_client_garage_air_large"
                    | "crate_client_garage_air_medium"
                    | "crate_client_garage_air_small" => self.count_air += 1,
                    "crate_client_garage_sea_large"
                    | "crate_client_garage_sea_medium"
                    | "crate_client_garage_sea_small" => self.count_sea += 1,
                    "crate_client_garage_thing_large"
                    | "crate_client_garage_thing_medium"
                    | "crate_client_garage_thing_small" => self.count_thing += 1,
                    _ => {}
                }
            }
            "Marker" => {
                let Some((name, _)) = get_string(class, "name") else {
                    return;
                };
                match name.as_str() {
                    "spawn_land" => self.count_land += 1,
                    "spawn_air" => self.count_air += 1,
                    "spawn_sea" => self.count_sea += 1,
                    "spawn_thing" => self.count_thing += 1,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = vec![];
        if self.expected {
            if self.count_land == 0 && !self.acknowledged_land {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "No land spawner found".to_string(),
                    Level::Error,
                ));
            }
            if self.count_thing == 0 && !self.acknowledged_land {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "No thing spawner found".to_string(),
                    Level::Error,
                ));
            }
        } else {
            if self.count_land != 0 {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Land spawner found, but spawners are not allowed on this mission type"
                        .to_string(),
                    Level::Error,
                ));
            }
            if self.count_air != 0 {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Air spawner found, but spawners are not allowed on this mission type"
                        .to_string(),
                    Level::Error,
                ));
            }
            if self.count_sea != 0 {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Sea spawner found, but spawners are not allowed on this mission type"
                        .to_string(),
                    Level::Error,
                ));
            }
            if self.count_thing != 0 {
                messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Thing spawner found, but spawners are not allowed on this mission type"
                        .to_string(),
                    Level::Error,
                ));
            }
        }
        messages
    }
}
