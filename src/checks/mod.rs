pub mod description;
pub mod objects;
pub mod time;

use std::path::Path;

use hemtt_config::{Class, Config, Property};
use hemtt_workspace::reporting::Processed;

use crate::{annotation::Annotation, get_class, get_string, GetChildren};

pub trait MissionCheck {
    fn object(
        &mut self,
        _mission: (&Processed, &Config),
        _dir: &Path,
        _class: &Class,
        _data_type: &str,
    ) {
    }
    fn link(&mut self, _mission: (&Processed, &Config), _dir: &Path, _class: &Class) {}
    fn done(&self, dir: &Path) -> Vec<Annotation>;
}

pub fn run_checks(
    dir: &Path,
    mut checks: Vec<Box<dyn MissionCheck>>,
    mission: (&Processed, &Config),
) -> Vec<Annotation> {
    let entities = get_class(mission.1, "Mission.Entities").unwrap();
    process_entities(mission, dir, &mut checks, entities);
    if let Some(links) = get_class(mission.1, "Mission.Connections.Links") {
        process_links(mission, dir, &mut checks, links);
    }
    checks.iter().flat_map(|c| c.done(dir)).collect()
}

fn process_entities(
    mission: (&Processed, &Config),
    dir: &Path,
    checks: &mut Vec<Box<dyn MissionCheck>>,
    parent: impl GetChildren,
) {
    for child in parent.get_children() {
        if let Property::Class(class) = child {
            let Some((data_type, _)) = get_string(&class, "dataType") else {
                continue;
            };
            for check in &mut *checks {
                check.object(mission, dir, &class, data_type.as_str());
            }
            if let "Group" | "Layer" = data_type.as_str() {
                let Some(entities) = get_class(&class, "Entities") else {
                    continue;
                };
                process_entities(mission, dir, checks, entities);
            }
        }
    }
}

fn process_links(
    mission: (&Processed, &Config),
    dir: &Path,
    checks: &mut Vec<Box<dyn MissionCheck>>,
    parent: impl GetChildren,
) {
    for child in parent.get_children() {
        if let Property::Class(class) = child {
            for check in &mut *checks {
                check.link(mission, dir, &class);
            }
        }
    }
}
