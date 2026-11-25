use std::path::Path;

use hemtt_config::{Class, Config};
use hemtt_workspace::reporting::Processed;

use crate::{
    annotation::{Annotation, Level},
    checks::MissionCheck,
    class_span, get_class, get_number, get_string,
};

pub const CONTRACTOR: &str = "synixe_factions_synixe_Contractor";

/// A unit left behind from testing in Eden: the unit the mission maker was playing as
/// when they saved, that isn't a player slot.
///
/// Eden saves real player slots with both `isPlayer=1` and `isPlayable=1`. A unit with only
/// `isPlayer=1` that isn't a Contractor is usually an enemy squad leader, a lone civilian, or
/// a random BLUFOR unit that was used to test the mission.
pub fn is_debug_unit(class: &Class) -> bool {
    let Some(attributes) = get_class(class, "Attributes") else {
        return false;
    };
    let flag = |name| matches!(get_number(attributes, name), Some((1, _)));
    flag("isPlayer")
        && !flag("isPlayable")
        && get_string(class, "type").is_some_and(|(t, _)| t != CONTRACTOR)
}

#[derive(Default)]
pub struct DebugUnits {
    messages: Vec<Annotation>,
}

impl DebugUnits {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MissionCheck for DebugUnits {
    fn object(
        &mut self,
        mission: (&Processed, &Config),
        dir: &Path,
        class: &Class,
        data_type: &str,
    ) {
        if data_type != "Object" || !is_debug_unit(class) {
            return;
        }
        let (unit_type, span) = get_string(class, "type")
            .map_or(("unknown", class_span(class)), |(t, s)| (t, s.clone()));
        let side = get_string(class, "side").map_or("unknown", |(s, _)| s);
        self.messages.push(Annotation::new(
            Some(mission.0),
            dir.join("mission.sqm").display().to_string(),
            span,
            format!(
                "Leftover debug unit: {unit_type} ({side}) is set as the player but isn't a player slot. Delete it if it was only for testing, or set a Contractor as the player"
            ),
            Level::Error,
        ));
    }

    fn done(&self, _: &Path) -> Vec<Annotation> {
        self.messages.clone()
    }
}
