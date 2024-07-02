use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    get_string,
};

use super::ObjectCheck;

pub struct RequireSpectator {
    seen: bool,
}

impl RequireSpectator {
    pub fn new() -> Self {
        Self { seen: false }
    }
}

impl Default for RequireSpectator {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectCheck for RequireSpectator {
    fn observe(
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
        if class == "synixe_spectator_screen" {
            self.seen = true;
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        if !self.seen {
            vec![Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "No spectator screen found".to_string(),
                Level::Error,
            )]
        } else {
            vec![]
        }
    }
}
