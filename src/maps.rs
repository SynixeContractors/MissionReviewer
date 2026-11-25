//! Keeping `maps.txt` in the missions repository up to date with the maps missions use

use std::path::{Path, PathBuf};

use crate::discover::Mission;

/// File the CI mode writes the missing maps to, for the action to open a pull request
pub const MISSING_MAPS: &str = "missionreviewer.maps";

/// Find `maps.txt` in the working directory, or in the repository the missions are in
pub fn find_maps_txt(missions: &[Mission]) -> Option<PathBuf> {
    std::iter::once(Path::new(""))
        .chain(missions.iter().flat_map(|m| m.path.ancestors().skip(1)))
        .map(|d| d.join("maps.txt"))
        .find(|p| p.is_file())
}

/// The map of a mission, from its folder name (`CO30_Author_Name.map`)
pub fn mission_map(mission: &Path) -> Option<&str> {
    mission.extension().and_then(|e| e.to_str())
}

/// Maps used by missions that are not listed in `maps.txt`, in the case the mission uses.
/// Comments (`# CUP`) and blank lines in `maps.txt` are ignored, map names are case insensitive.
pub fn missing_maps(maps_txt: &str, missions: &[Mission]) -> Vec<String> {
    let known = maps_txt
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    let mut missing: Vec<String> = Vec::new();
    for map in missions.iter().filter_map(|m| mission_map(&m.path)) {
        let lower = map.to_lowercase();
        if !known.contains(&lower) && !missing.iter().any(|m| m.to_lowercase() == lower) {
            missing.push(map.to_string());
        }
    }
    missing.sort_by_key(|m| m.to_lowercase());
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_missing() {
        let maps = "# Vanilla\nAltis\r\nTanoa\n\n# CUP\nchernarus\n";
        let missions = [
            "contracts/CO30_A.Altis",
            "contracts/CO30_B.altis",
            "contracts/CO30_C.kunduz_valley",
            "contracts/CO30_D.Chernarus",
            "contracts/SCO30_E.drakovac",
            "contracts/SCO30_F.kunduz_valley",
            "contracts/no_map",
        ]
        .map(|p| Mission::new(PathBuf::from(p)));
        assert_eq!(
            missing_maps(maps, &missions),
            vec!["drakovac", "kunduz_valley"]
        );
    }
}
