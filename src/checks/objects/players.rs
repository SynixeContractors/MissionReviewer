use std::path::Path;

use hemtt_common::reporting::Processed;
use hemtt_config::Config;

use crate::{
    annotation::{Annotation, Level},
    get_class, get_number, get_string,
};

use super::ObjectCheck;

pub struct PlayerCheck {
    count: usize,
    expected: usize,
    require_contractors: bool,
    messages: Vec<Annotation>,
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
        }
    }
}

impl ObjectCheck for PlayerCheck {
    fn observe(
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

        let Some((description, description_span)) = get_string(attributes, "description") else {
            return;
        };
        if description != "Contractor" {
            self.messages.push(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                description_span,
                "Player description should be 'Contractor'".to_string(),
                Level::Error,
            ));
        }

        let Some((class, class_span)) = get_string(class, "type") else {
            return;
        };
        if class != "synixe_contractors_Unit_I_Contractor" {
            self.messages.push(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                class_span,
                "Player class should be 'synixe_contractors_Unit_I_Contractor'".to_string(),
                Level::Error,
            ));
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = self.messages.clone();
        if self.count != self.expected {
            messages.push(Annotation::new(
                None,
                "mission.sqm".to_string(),
                0..0,
                format!("Expected {} players, found {}", self.expected, self.count),
                Level::Error,
            ));
        }
        messages
    }
}
