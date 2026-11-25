use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_string,
};

pub struct ZeusModule {
    seen: Option<Annotation>,
}

impl ZeusModule {
    pub fn new() -> Self {
        Self { seen: None }
    }
}

impl Default for ZeusModule {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionCheck for ZeusModule {
    fn object(
        &mut self,
        mission: (&Processed, &Config),
        dir: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Logic" {
            return;
        }
        if self.seen.is_some() {
            return;
        }
        let Some((class_type, class_type_span)) = get_string(class, "type") else {
            return;
        };
        if class_type == "ModuleCurator_F" {
            self.seen = Some(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                class_type_span.clone(),
                "Zeus modules should not be placed in missions. (You can use ACE interact for local testing)".to_string(),
                Level::Error,
            ));
        }
    }

    fn done(&self, _: &Path) -> Vec<Annotation> {
        self.seen.iter().cloned().collect()
    }
}
