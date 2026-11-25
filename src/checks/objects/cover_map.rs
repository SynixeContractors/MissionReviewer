use std::path::Path;

use hemtt_config::{Config, Item};
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    class_span, extract_number, get_array, get_number, get_string,
};

pub struct CoverMapCheck {
    messages: Vec<Annotation>,
}

impl CoverMapCheck {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    fn get_area_size(class: &hemtt_config::Class) -> Option<(f32, f32)> {
        let (dimensions, _) = get_array(class, "areaSize")?;
        if dimensions.len() != 3 {
            return None;
        }
        let Item::Number(width) = &dimensions[0] else {
            return None;
        };
        let Item::Number(length) = &dimensions[2] else {
            return None;
        };

        Some((extract_number(width)?.abs(), extract_number(length)?.abs()))
    }
}

/// Eden stores sizes as floats, so allow for rounding (e.g. 500.00003)
fn is_multiple_of_100(value: f32) -> bool {
    let remainder = value.rem_euclid(100.0);
    remainder < 0.01 || remainder > 99.99
}

impl Default for CoverMapCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionCheck for CoverMapCheck {
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

        let Some((class_type, _)) = get_string(class, "type") else {
            return;
        };

        if class_type != "ModuleCoverMap_F" {
            return;
        }

        // Check if it's a rectangle
        let is_rectangle = get_number(class, "areaIsRectangle").map_or(0, |(value, _)| value);
        if is_rectangle != 1 {
            self.messages.push(Annotation::new(
                Some(mission.0),
                dir.join("mission.sqm").display().to_string(),
                class_span(class),
                "Cover maps must be rectangles (set areaIsRectangle=1)".to_string(),
                Level::Error,
            ));
            return;
        }

        // Check if width and length are multiples of 100
        if let Some((width, length)) = Self::get_area_size(class) {
            if !is_multiple_of_100(width) || !is_multiple_of_100(length) {
                self.messages.push(Annotation::new(
                    Some(mission.0),
                    dir.join("mission.sqm").display().to_string(),
                    class_span(class),
                    format!(
                        "Cover map dimensions must be a multiple of 100 (current: {:.2} × {:.2})",
                        width, length
                    ),
                    Level::Error,
                ));
            }
        }
    }

    fn done(&self, _: &Path) -> Vec<Annotation> {
        self.messages.clone()
    }
}
