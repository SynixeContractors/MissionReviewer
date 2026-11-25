use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_class, get_number, get_string,
};

pub struct HostilesCheck {
    east: u32,
    independent: u32,
    independent_is_hostile: bool,
    scripts_spawn_units: bool,
}

impl HostilesCheck {
    pub fn new(resistance_west: i32, scripts_spawn_units: bool) -> Self {
        Self {
            east: 0,
            independent: 0,
            independent_is_hostile: resistance_west == 0,
            scripts_spawn_units,
        }
    }
}

impl MissionCheck for HostilesCheck {
    fn object(
        &mut self,
        _: (&Processed, &Config),
        _: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Object" {
            return;
        }
        let Some(attributes) = get_class(class, "Attributes") else {
            return;
        };
        let is_player = get_number(attributes, "isPlayer")
            .map(|(value, _)| value)
            .unwrap_or(0);
        let is_playable = get_number(attributes, "isPlayable")
            .map(|(value, _)| value)
            .unwrap_or(0);
        if is_player == 1 || is_playable == 1 {
            return;
        }

        let Some((side, _)) = get_string(class, "side") else {
            return;
        };
        if side == "East" {
            self.east += 1;
        } else if side == "Independent" {
            self.independent += 1;
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = Vec::new();
        let hostiles = self.east
            + if self.independent_is_hostile {
                self.independent
            } else {
                0
            };
        if hostiles == 0 {
            let (message, level) = if self.scripts_spawn_units {
                (
                    "No hostiles placed in the editor, assuming they are spawned by script",
                    Level::Warning,
                )
            } else if self.independent > 0 {
                // Zeus can change diplomacy during the mission
                (
                    "Independent units are friendly to West in the editor, fine if Zeus makes them hostile",
                    Level::Warning,
                )
            } else {
                ("No hostiles found in mission", Level::Error)
            };
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                format!(
                    "{} (East: {}, Independent: {}{})",
                    message,
                    self.east,
                    self.independent,
                    if self.independent_is_hostile || self.independent == 0 {
                        ""
                    } else {
                        ", friendly to West"
                    }
                ),
                level,
            ));
        }
        messages
    }
}
