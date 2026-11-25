use std::path::Path;

use hemtt_config::Config;
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::{
        objects::debug_units::{is_debug_unit, CONTRACTOR},
        MissionCheck,
    },
    class_span, get_class, get_number, get_string,
};

const SIDE: &str = "should be on side 'West'";
const DESCRIPTION: &str = "should have the description 'Contractor'";
const CLASS: &str = "should be 'synixe_factions_synixe_Contractor'";

/// A problem that is reported once, on the first unit that has it
struct Problem {
    message: &'static str,
    first: Option<Annotation>,
    count: usize,
}

impl Problem {
    const fn new(message: &'static str) -> Self {
        Self {
            message,
            first: None,
            count: 0,
        }
    }

    fn add(&mut self, processed: &Processed, dir: &Path, span: std::ops::Range<usize>) {
        if self.first.is_none() {
            self.first = Some(Annotation::new(
                Some(processed),
                dir.join("mission.sqm").display().to_string(),
                span,
                String::new(),
                Level::Error,
            ));
        }
        self.count += 1;
    }

    fn annotation(&self) -> Option<Annotation> {
        let mut annotation = self.first.clone()?;
        annotation.message = format!(
            "All players {} ({} {} not)",
            self.message,
            self.count,
            if self.count == 1 {
                "player is"
            } else {
                "players are"
            }
        );
        Some(annotation)
    }
}

pub struct PlayerCheck {
    count: usize,
    expected: usize,
    require_contractors: bool,
    wrong_side: Problem,
    wrong_description: Problem,
    wrong_class: Problem,
}

impl PlayerCheck {
    pub fn new(dir: &Path, require_contractors: bool) -> Self {
        Self {
            count: 0,
            expected: dir
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.split_once('_'))
                .and_then(|(prefix, _)| {
                    prefix
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .ok()
                })
                .unwrap_or(1),
            require_contractors,
            wrong_side: Problem::new(SIDE),
            wrong_description: Problem::new(DESCRIPTION),
            wrong_class: Problem::new(CLASS),
        }
    }
}

impl MissionCheck for PlayerCheck {
    fn object(
        &mut self,
        mission: (&Processed, &Config),
        dir: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        // Reported by the debug units check instead
        if data_type != "Object" || is_debug_unit(class) {
            return;
        }
        let Some(attributes) = get_class(class, "Attributes") else {
            return;
        };
        let is_player = get_number(attributes, "isPlayer")
            .map(|(value, _)| value)
            .unwrap_or(0);
        let is_playable = get_number(attributes, "isPlayable")
            .map(|(value, _)| value)
            .unwrap_or(0);
        if is_player != 1 && is_playable != 1 {
            return;
        }
        self.count += 1;

        if !self.require_contractors {
            return;
        }

        match get_string(class, "side") {
            Some(("West", _)) => {}
            Some((_, span)) => self.wrong_side.add(mission.0, dir, span.clone()),
            None => self.wrong_side.add(mission.0, dir, class_span(class)),
        }
        match get_string(attributes, "description") {
            Some(("Contractor", _)) => {}
            Some((_, span)) => self.wrong_description.add(mission.0, dir, span.clone()),
            None => self
                .wrong_description
                .add(mission.0, dir, class_span(attributes)),
        }
        match get_string(class, "type") {
            Some((t, _)) if t == CONTRACTOR => {}
            Some((_, span)) => self.wrong_class.add(mission.0, dir, span.clone()),
            None => self.wrong_class.add(mission.0, dir, class_span(class)),
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = [&self.wrong_side, &self.wrong_description, &self.wrong_class]
            .into_iter()
            .filter_map(Problem::annotation)
            .collect::<Vec<_>>();
        if self.count != self.expected {
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                format!("Expected {} players, found {}", self.expected, self.count),
                Level::Error,
            ));
        }
        messages
    }
}
