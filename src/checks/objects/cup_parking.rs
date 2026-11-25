use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_string,
};

pub struct CUPParking {
    seen: Option<Annotation>,
}

impl CUPParking {
    pub fn new() -> Self {
        Self { seen: None }
    }
}

impl Default for CUPParking {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionCheck for CUPParking {
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
        if self.seen.is_some() {
            return;
        }
        let Some((class_type, class_type_span)) = get_string(class, "type") else {
            return;
        };
        if class_type == "CUP_sign_parking" || class_type == "CUP_sign_parking_new" {
            self.seen = Some(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                class_type_span.clone(),
                "Use the vanilla \"Parking Lot\" (RoadSign_Livonia_parking) sign, the CUP sign floats above the ground".to_string(),
                Level::Error,
            ));
        }
    }

    fn done(&self, _: &Path) -> Vec<Annotation> {
        self.seen.iter().cloned().collect()
    }
}
