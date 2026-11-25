use std::{collections::HashMap, path::Path, sync::OnceLock};

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
    // parse times.txt once
    static TIMES: OnceLock<HashMap<String, (i32, i32)>> = OnceLock::new();
    let times = TIMES.get_or_init(|| {
        let mut times = HashMap::new();
        // In the working directory, or the missions repository the mission is in
        let times_path = std::iter::once(Path::new(""))
            .chain(dir.ancestors().skip(1))
            .map(|d| d.join("starts.txt"))
            .find(|p| p.is_file());
        if let Some(lines) = times_path.and_then(|p| std::fs::read_to_string(p).ok()) {
            for line in lines.lines() {
                // example: Altis|12:30
                let mut parts = line.split('|');
                if let (Some(map), Some(time)) = (parts.next(), parts.next()) {
                    let mut char = ':';
                    if time.contains('.') {
                        char = '.';
                    }
                    let mut time_parts = time.split(char);
                    if let (Some(hour), Some(minute)) = (time_parts.next(), time_parts.next()) {
                        if let (Ok(hour), Ok(minute)) = (hour.parse::<i32>(), minute.parse::<i32>())
                        {
                            times.insert(map.to_lowercase(), (hour, minute));
                        }
                    }
                }
            }
        }
        times
    });

    let ext_path = dir.to_path_buf().join("edit_me").join("description.ext");
    let sqm_path = dir.to_path_buf().join("mission.sqm");
    let mut messages = Vec::new();
    let Some((synixe_start_time, synixe_start_time_span)) =
        get_number(config.1, "synixe_start_time")
    else {
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
            synixe_start_time_span,
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
    let (mission_hour, mission_hour_span) =
        if let Some((mission_hour, mission_hour_span)) = get_number(mission_intel, "hour") {
            (mission_hour, mission_hour_span)
        } else {
            let Some(map) = dir.extension().and_then(|ext| ext.to_str()) else {
                messages.push(Annotation::new(
                    Some(mission.0),
                    sqm_path.display().to_string(),
                    0..0,
                    "Mission filename does not contain a map name".to_string(),
                    Level::Error,
                ));
                return messages;
            };
            let Some((mission_hour, _)) = times.get(&map.to_lowercase()) else {
                messages.push(Annotation::new(
                    Some(mission.0),
                    sqm_path.display().to_string(),
                    0..0,
                    "Mission >> Intel >> hour is missing".to_string(),
                    Level::Error,
                ));
                return messages;
            };
            (*mission_hour, 0..0)
        };
    if !(mission_hour + 1 == synixe_start_time || (mission_hour == 23 && synixe_start_time == 0)) {
        messages.push(Annotation::new(
            Some(mission.0),
            sqm_path.display().to_string(),
            mission_hour_span,
            format!("Editor hour needs to be 1 hour before synixe_start_time. Editor: {}, Description: {}", mission_hour, synixe_start_time),
            Level::Error,
        ));
    }

    let (mission_minutes, mission_minutes_span) = if let Some((
        mission_minutes,
        mission_minutes_span,
    )) = get_number(mission_intel, "minute")
    {
        (mission_minutes, mission_minutes_span)
    } else {
        let Some(map) = dir.extension().and_then(|ext| ext.to_str()) else {
            messages.push(Annotation::new(
                Some(mission.0),
                sqm_path.display().to_string(),
                0..0,
                "Mission filename does not contain a map name".to_string(),
                Level::Error,
            ));
            return messages;
        };
        let Some((_, mission_minutes)) = times.get(&map.to_lowercase()) else {
            println!("Missing time for map {} in starts.txt", map);
            messages.push(Annotation::new(
                Some(mission.0),
                sqm_path.display().to_string(),
                0..0,
                "Mission >> Intel >> minute is missing".to_string(),
                Level::Error,
            ));
            return messages;
        };
        (*mission_minutes, 0..0)
    };

    if mission_minutes != 0 {
        messages.push(Annotation::new(
            Some(mission.0),
            sqm_path.display().to_string(),
            mission_minutes_span,
            "Editor minutes needs to be 0".to_string(),
            Level::Error,
        ));
    }

    messages
}
