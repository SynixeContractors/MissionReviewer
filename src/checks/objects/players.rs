use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_class, get_number, get_string,
};

pub struct PlayerCheck {
    count: usize,
    expected: usize,
    require_contractors: bool,
    messages: Vec<Annotation>,
    did_log_player_description: bool,
}

impl PlayerCheck {
    pub fn new(dir: &Path, require_contractors: bool) -> Self {
        Self {
            count: 0,
            expected: {
                dir.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split_once('_')
                    .map(|(prefix, _)| prefix)
                    .and_then(|prefix| {
                        prefix
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse()
                            .ok()
                    })
                    .unwrap_or(1)
            },
            require_contractors,
            messages: vec![],
            did_log_player_description: false,
        }
    }
}

impl MissionCheck for PlayerCheck {
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
        let Some((playable, _)) = get_number(&attributes, "isPlayable") else {
            return;
        };
        if playable != 1 {
            return;
        }
        self.count += 1;

        if !self.require_contractors {
            return;
        }

        let Some((description, description_span)) = get_string(&attributes, "description") else {
            if !self.did_log_player_description {
                self.messages.push(Annotation::new(
                    Some(mission.0),
                    dir.join("mission.sqm").display().to_string(),
                    attributes.name().map(|n| n.span.clone()).unwrap_or(0..0),
                    "All player descriptions should be 'Contractor'".to_string(),
                    Level::Error,
                ));
                self.did_log_player_description = true;
            }
            return;
        };
        if description != "Contractor" && !self.did_log_player_description {
            self.messages.push(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                description_span,
                "All player descriptions should be 'Contractor'".to_string(),
                Level::Error,
            ));
            self.did_log_player_description = true;
        }

        let Some((class, class_span)) = get_string(class, "type") else {
            return;
        };
        if class != "synixe_factions_synixe_Contractor" {
            self.messages.push(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                class_span,
                "Player class should be 'synixe_factions_synixe_Contractor'".to_string(),
                Level::Error,
            ));
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = self.messages.clone();
        if self.count != self.expected {
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                format!("Expected {} players, found {}", self.expected, self.count),
                Level::Error,
            ));
        }
        messages
    }
}
