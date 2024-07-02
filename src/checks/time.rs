use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    get_class, get_number,
};

pub fn time(
    dir: &Path,
    mission: (&Processed, &Config),
    config: (&Processed, &Config),
) -> Vec<Annotation> {
    let ext_path = dir.to_path_buf().join("edit_me").join("description.ext");
    let sqm_path = dir.to_path_buf().join("mission.sqm");
    let mut messages = Vec::new();
    let Some((synixe_start_time, _)) = get_number(config.1, "synixe_start_time") else {
        messages.push(Annotation::new(
            Some(config.0),
            ext_path.display().to_string(),
            0..0,
            "synixe_start_time is missing".to_string(),
            Level::Error,
        ));
        return messages;
    };
    if !(0..=24).contains(&synixe_start_time) {
        messages.push(Annotation::new(
            Some(config.0),
            ext_path.display().to_string(),
            0..0,
            "synixe_start_time is not between 0 and 24".to_string(),
            Level::Error,
        ));
        return messages;
    }
    let Some(mission_intel) = get_class(mission.1, "Mission.Intel") else {
        messages.push(Annotation::new(
            Some(mission.0),
            sqm_path.display().to_string(),
            0..0,
            "Mission >> Intel is missing".to_string(),
            Level::Error,
        ));
        return messages;
    };
    let Some((mission_hour, mission_hour_span)) = get_number(mission_intel, "hour") else {
        messages.push(Annotation::new(
            Some(mission.0),
            sqm_path.display().to_string(),
            0..0,
            "Mission >> Intel >> hour is missing".to_string(),
            Level::Error,
        ));
        return messages;
    };
    if !(mission_hour + 1 == synixe_start_time || (mission_hour == 23 && synixe_start_time == 0)) {
        messages.push(Annotation::new(
            Some(mission.0),
            sqm_path.display().to_string(),
            mission_hour_span,
            "Mission >> Intel >> hour needs to be 1 hour before synixe_start_time".to_string(),
            Level::Error,
        ));
    }
    messages
}
