use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_string,
};

pub struct ZeusModule {
    seen: bool,
}

impl ZeusModule {
    pub fn new() -> Self {
        Self { seen: false }
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
        _: (&Processed, &Config),
        _: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Logic" {
            return;
        }
        if self.seen {
            return;
        }
        let Some((class, _)) = get_string(class, "type") else {
            return;
        };
        if class == "ModuleCurator_F" {
            self.seen = true;
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        if self.seen {
            vec![Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "Zeus modules should not be placed in missions. (You can use ACE interact for local testing)".to_string(),
                Level::Error,
            )]
        } else {
            vec![]
        }
    }
}
