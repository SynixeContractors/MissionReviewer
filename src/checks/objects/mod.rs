pub mod players;
pub mod shops;
pub mod spawners;
pub mod spectator;

use std::path::Path;

use hemtt_config::{Class, Config, Property};
use hemtt_workspace::reporting::Processed;

use crate::{annotation::Annotation, get_class, get_string, GetChildren};

pub trait ObjectCheck {
    fn observe(
        &mut self,
        mission: (&Processed, &Config),
        dir: &Path,
        class: &Class,
        data_type: &str,
    );
    fn done(&self, dir: &Path) -> Vec<Annotation>;
}

pub fn run_over_entities(
    dir: &Path,
    mut checks: Vec<Box<dyn ObjectCheck>>,
    mission: (&Processed, &Config),
) -> Vec<Annotation> {
    let entities = get_class(mission.1, "Mission.Entities").unwrap();
    layer(mission, dir, &mut checks, entities);
    checks.iter().flat_map(|c| c.done(dir)).collect()
}

fn layer(
    mission: (&Processed, &Config),
    dir: &Path,
    checks: &mut Vec<Box<dyn ObjectCheck>>,
    parent: impl GetChildren,
) {
    for child in parent.get_children() {
        if let Property::Class(class) = child {
            let Some((data_type, _)) = get_string(&class, "dataType") else {
                continue;
            };
            for check in &mut *checks {
                check.observe(mission, dir, &class, data_type.as_str());
            }
            if let "Group" | "Layer" = data_type.as_str() {
                let Some(entities) = get_class(&class, "Entities") else {
                    continue;
                };
                layer(mission, dir, checks, entities);
            }
        }
    }
}
