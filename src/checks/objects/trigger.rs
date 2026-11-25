use std::path::Path;

use hemtt_config::{Class, Config};
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    class_span, get_class, get_float, get_number, get_string,
};

pub struct TriggerChecks {
    messages: Vec<Annotation>,
    triggers: Vec<Class>,
    waypoints: Vec<Class>,
    checked_triggers: Vec<i32>,
}

impl TriggerChecks {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            triggers: Vec::new(),
            waypoints: Vec::new(),
            checked_triggers: Vec::new(),
        }
    }
}

impl TriggerChecks {
    fn check_trigger(processed: &Processed, path: &str, trigger: &Class) -> Vec<Annotation> {
        let mut messages = Vec::new();
        let Some(attributes) = get_class(trigger, "Attributes") else {
            return messages;
        };
        if !matches!(get_number(attributes, "isServerOnly"), Some((1, _))) {
            messages.push(Annotation::new(
                Some(processed),
                path.to_string(),
                class_span(trigger),
                "Trigger not set to server only".to_string(),
                Level::Error,
            ));
        }
        // Eden leaves out the default interval of 0.5 seconds
        let (interval, span) =
            get_float(attributes, "triggerInterval").unwrap_or((0.5, class_span(trigger)));
        if interval < MIN_INTERVAL {
            messages.push(Annotation::new(
                Some(processed),
                path.to_string(),
                span,
                format!(
                    "Trigger interval is too low ({interval} seconds), it must be at least {MIN_INTERVAL} seconds"
                ),
                Level::Error,
            ));
        }
        messages
    }
}

/// Lowest allowed trigger interval, the Eden default is 0.5
const MIN_INTERVAL: f32 = 0.6;

impl Default for TriggerChecks {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionCheck for TriggerChecks {
    fn object(
        &mut self,
        mission: (&Processed, &Config),
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
                let Some((waypoint_type, waypoint_type_span)) = get_string(class, "type") else {
                    return;
                };
                if waypoint_type == "Guard" {
                    self.messages.push(Annotation::new(
                        Some(mission.0),
                        dir.join("mission.sqm").display().to_string(),
                        waypoint_type_span.clone(),
                        "Guard waypoint is not allowed".to_string(),
                        Level::Error,
                    ));
                }
            }
            _ => {}
        }
    }

    fn link(&mut self, mission: (&Processed, &Config), dir: &Path, class: &Class) {
        let path = dir.join("mission.sqm").display().to_string();
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
                if waypoint.is_none() {
                    waypoint = Some(item_1);
                } else {
                    // If both items are waypoints, it's probably fine
                    return;
                }
            } else if type_ == "Trigger" {
                trigger = Some(item_1);
            }
        }
        if let Some(trigger) = trigger {
            // Trigger class names (Item0, Item1, ...) repeat between layers, the id does not
            let id = get_number(trigger, "id").map(|(id, _)| id);
            if !id.is_some_and(|id| self.checked_triggers.contains(&id)) {
                self.checked_triggers.extend(id);
                self.messages
                    .extend(Self::check_trigger(mission.0, &path, trigger));
            }
        }
        let (Some(trigger), Some(waypoint)) = (trigger, waypoint) else {
            self.messages.push(Annotation::new(
                Some(mission.0),
                path,
                class_span(class),
                "WaypointActivation link does not connect to a trigger and waypoint".to_string(),
                Level::Error,
            ));
            return;
        };
        let Some((waypoint_type, waypoint_type_span)) = get_string(waypoint, "type") else {
            self.messages.push(Annotation::new(
                Some(mission.0),
                path,
                class_span(waypoint),
                "WaypointActivation link does not connect to a valid waypoint".to_string(),
                Level::Error,
            ));
            return;
        };
        let trigger_type = get_class(trigger, "Attributes")
            .and_then(|attributes| get_string(attributes, "type"))
            .map_or("NONE", |(s, _)| s);
        // A HOLD waypoint is only released by a trigger of type SWITCH (Skip Waypoint)
        if waypoint_type == "Hold" && trigger_type != "SWITCH" {
            self.messages.push(Annotation::new(
                Some(mission.0),
                path,
                waypoint_type_span.clone(),
                "HOLD waypoint is linked to a trigger that isn't SKIP WAYPOINT".to_string(),
                Level::Error,
            ));
        }
    }

    fn done(&self, _dir: &Path) -> Vec<Annotation> {
        self.messages.clone()
    }
}
