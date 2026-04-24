use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::{objects::hostiles, MissionCheck},
    get_class, get_number, get_string,
};

pub struct HostilesCheck {
    east: u32,
    independent: u32,
    independent_is_hostile: bool,
}

impl HostilesCheck {
    pub fn new(resistance_west: i32) -> Self {
        Self {
            east: 0,
            independent: 0,
            independent_is_hostile: resistance_west == 0,
        }
    }
}

impl MissionCheck for HostilesCheck {
    fn object(
        &mut self,
        mission: (&Processed, &Config),
        dir: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Object" {
            return;
        }
        let Some(attributes) = get_class(class, "Attributes") else {
            return;
        };
        let is_player = get_number(&attributes, "isPlayer")
            .map(|(value, _)| value)
            .unwrap_or(0);
        let is_playable = get_number(&attributes, "isPlayable")
            .map(|(value, _)| value)
            .unwrap_or(0);
        if is_player == 1 || is_playable == 1 {
            return;
        }

        let Some((side, side_range)) = get_string(class, "side") else {
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
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                format!(
                    "No hostiles found in mission (East: {}, Independent: {})",
                    self.east, self.independent
                ),
                Level::Error,
            ));
        }
        messages
    }
}
