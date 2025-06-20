use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_string,
};

pub struct CUPParking {
    seen: bool,
}

impl CUPParking {
    pub fn new() -> Self {
        Self { seen: false }
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
        _: (&Processed, &Config),
        _: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Object" {
            return;
        }
        if self.seen {
            return;
        }
        let Some((class, _)) = get_string(class, "type") else {
            return;
        };
        if class == "CUP_sign_parking" {
            self.seen = true;
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        if self.seen {
            vec![Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "Use the vanilla \"Parking Lot\" (RoadSign_Livonia_parking) sign, the CUP sign floats above the ground".to_string(),
                Level::Error,
            )]
        } else {
            vec![]
        }
    }
}
