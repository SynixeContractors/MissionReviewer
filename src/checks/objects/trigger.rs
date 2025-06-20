use std::path::Path;

use hemtt_config::{Class, Config};
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    get_class, get_float, get_number, get_string,
};

pub struct TriggerChecks {
    messages: Vec<Annotation>,
    triggers: Vec<Class>,
    waypoints: Vec<Class>,
}

impl TriggerChecks {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            triggers: Vec::new(),
            waypoints: Vec::new(),
        }
    }
}

impl Default for TriggerChecks {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionCheck for TriggerChecks {
    fn object(
        &mut self,
        _: (&Processed, &Config),
        dir: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        match data_type {
            "Trigger" => {
                self.triggers.push(class.clone());
            }
            "Waypoint" => {
                self.waypoints.push(class.clone());
                let Some((waypoint_type, _)) = get_string(class, "type") else {
                    return;
                };
                if waypoint_type == "Guard" {
                    self.messages.push(Annotation::new(
                        None,
                        dir.join("mission.sqm").display().to_string(),
                        0..0,
                        "Guard waypoint is not allowed".to_string(),
                        Level::Error,
                    ));
                }
            }
            _ => {}
        }
    }

    fn link(&mut self, _mission: (&Processed, &Config), dir: &Path, class: &Class) {
        let Some(custom_data) = get_class(class, "CustomData") else {
            return;
        };
        let Some((r#type, _)) = get_string(custom_data, "type") else {
            return;
        };
        if r#type != "WaypointActivation" {
            return;
        }
        let Some((item_0, _)) = get_number(class, "item0") else {
            return;
        };
        let Some((item_1, _)) = get_number(class, "item1") else {
            return;
        };
        let Some(item_0) = self
            .triggers
            .iter()
            .find(|c| matches!(get_number(*c, "id"), Some((id, _)) if id == item_0))
            .or_else(|| {
                self.waypoints
                    .iter()
                    .find(|c| matches!(get_number(*c, "id"), Some((id, _)) if id == item_0))
            })
        else {
            return;
        };
        let Some(item_1) = self
            .triggers
            .iter()
            .find(|c| matches!(get_number(*c, "id"), Some((id, _)) if id == item_1))
            .or_else(|| {
                self.waypoints
                    .iter()
                    .find(|c| matches!(get_number(*c, "id"), Some((id, _)) if id == item_1))
            })
        else {
            return;
        };
        let mut trigger = None;
        let mut waypoint = None;
        if let Some((type_, _)) = get_string(item_0, "dataType") {
            if type_ == "Waypoint" {
                waypoint = Some(item_0);
            } else if type_ == "Trigger" {
                trigger = Some(item_0);
            }
        }
        if let Some((type_, _)) = get_string(item_1, "dataType") {
            if type_ == "Waypoint" {
                waypoint = Some(item_1);
            } else if type_ == "Trigger" {
                trigger = Some(item_1);
            }
        }
        if let Some(trigger) = trigger {
            // Check for isServerOnly
            if let Some((is_server_only, _)) = get_number(trigger, "isServerOnly") {
                if is_server_only == 0 {
                    self.messages.push(Annotation::new(
                        None,
                        dir.join("mission.sqm").display().to_string(),
                        0..0,
                        "Trigger not set to server only".to_string(),
                        Level::Error,
                    ));
                }
            } else {
                self.messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Trigger not set to server only".to_string(),
                    Level::Error,
                ));
            }

            // Check interval
            if let Some((interval, _)) = get_float(trigger, "interval") {
                if interval < 0.6 {
                    self.messages.push(Annotation::new(
                        None,
                        dir.join("mission.sqm").display().to_string(),
                        0..0,
                        "Trigger interval is set too low (below 0.5 seconds)".to_string(),
                        Level::Error,
                    ));
                }
            } else {
                self.messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "Trigger interval is set too low (below 0.5 seconds)".to_string(),
                    Level::Error,
                ));
            }
        }
        if trigger.is_none() || waypoint.is_none() {
            self.messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "WaypointActivation link does not connect to a trigger and waypoint".to_string(),
                Level::Error,
            ));
            return;
        }
        let trigger = trigger.unwrap();
        let waypoint = waypoint.unwrap();
        let Some((waypoint_type, _)) = get_string(waypoint, "type") else {
            self.messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "WaypointActivation link does not connect to a valid waypoint".to_string(),
                Level::Error,
            ));
            return;
        };
        let trigger_type = {
            let Some(attributes) = get_class(trigger, "Attributes") else {
                return;
            };
            get_string(&attributes, "type")
                .map(|(s, _)| s)
                .unwrap_or_else(|| "ACTIVATE".to_string())
        };
        #[allow(clippy::single_match)]
        match (trigger_type.as_str(), waypoint_type.as_str()) {
            ("ACTIVATE", "Hold") => {
                self.messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    0..0,
                    "HOLD waypoint is linked to a trigger that isn't SKIP WAYPOINT".to_string(),
                    Level::Error,
                ));
            }
            _ => {}
        }
    }

    fn done(&self, _dir: &Path) -> Vec<Annotation> {
        self.messages.clone()
    }
}
