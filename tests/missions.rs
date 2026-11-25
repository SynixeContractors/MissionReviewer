use std::path::{Path, PathBuf};

use missionreviewer::{
    annotation::Level,
    get_class, get_number,
    mission::{check, check_prefix, read_description, read_mission},
};

const GRANDIOSE: &str = "tests/CO30_Brett_GrandioseTeachings.Tanoa";
const IVORY: &str = "tests/CO30_Connect_LeaningTowerOfIvory.Altis";
const LEAP: &str = "tests/CO30_Matias_Leap.zargabad";
const IRENE: &str = "tests/CO30_Tobias_Irene.NorthTakistan";

/// (line, message) of every annotation
fn review(dir: &Path) -> Vec<(usize, String)> {
    let mut messages = check(dir)
        .unwrap()
        .into_iter()
        .map(|a| (a.start_line, a.message))
        .collect::<Vec<_>>();
    messages.sort();
    messages
}

fn messages(dir: &Path) -> Vec<String> {
    review(dir).into_iter().map(|(_, m)| m).collect()
}

/// Copy a fixture into a fresh folder so it can be broken on purpose
fn copy_fixture(fixture: &str, name: &str) -> PathBuf {
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let fixture = Path::new(fixture);
    let dir = std::env::temp_dir()
        .join(format!("missionreviewer-{}-{}", std::process::id(), name))
        .join(fixture.file_name().unwrap());
    let _ = std::fs::remove_dir_all(&dir);
    copy(fixture, &dir);
    dir
}

fn replace_in(path: &Path, from: &str, to: &str, count: usize) {
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.matches(from).count() >= count,
        "{from:?} not found {count} times in {}",
        path.display()
    );
    std::fs::write(path, content.replacen(from, to, count)).unwrap();
}

#[test]
fn clean_missions() {
    for dir in [IVORY, LEAP, IRENE] {
        assert_eq!(messages(Path::new(dir)), Vec::<String>::new(), "{dir}");
    }
}

#[test]
fn grandiose_start_time() {
    assert_eq!(
        review(Path::new(GRANDIOSE)),
        vec![(
            789,
            "Editor hour needs to be 1 hour before synixe_start_time. Editor: 12, Description: 12"
                .to_string()
        )]
    );
}

#[test]
fn reads_time() {
    let dir = PathBuf::from(IRENE);
    let description = read_description(&dir).unwrap();
    assert_eq!(description.version, 3);
    assert!(description.skip_lobby);
    assert_eq!(
        get_number(description.config.config(), "synixe_start_time").map(|(a, _)| a),
        Some(8)
    );
    let (_, mission) = read_mission(&dir).unwrap();
    let intel = get_class(mission.config(), "Mission.Intel").unwrap();
    assert_eq!(get_number(intel, "hour").map(|(a, _)| a), Some(7));
}

#[test]
fn prefixes() {
    assert!(check_prefix(Path::new("contracts/CO30_A.Altis"), false).is_empty());
    assert!(check_prefix(Path::new("contracts/SCO10_A.Altis"), false).is_empty());
    assert!(check_prefix(Path::new("specials/SPC30_A.Altis"), false).is_empty());
    assert!(check_prefix(Path::new("theatres/x/CCO10_A.Altis"), true).is_empty());
    assert!(check_prefix(Path::new("theatres/x/TT_A.Altis"), true).is_empty());
    assert_eq!(
        check_prefix(Path::new("contracts/XX30_A.Altis"), false).len(),
        1
    );
    assert_eq!(
        check_prefix(Path::new("theatres/x/CO30_A.Altis"), true).len(),
        1
    );
}

#[test]
fn briefing_non_html_does_not_crash() {
    let dir = copy_fixture(IRENE, "gitkeep");
    std::fs::write(dir.join("edit_me/briefing/.gitkeep"), "").unwrap();
    assert_eq!(
        messages(&dir),
        vec!["Briefing file is not an HTML file".to_string()]
    );
}

#[test]
fn briefing_ampersand() {
    let dir = copy_fixture(IRENE, "ampersand");
    let path = dir.join("edit_me/briefing/mission.html");
    let content = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("R&R &amp; more\n{content}")).unwrap();
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1);
    assert_eq!(
        annotations[0].message,
        "Briefing file contains an unescaped ampersand. Use &amp; instead."
    );
    assert_eq!(
        (annotations[0].start_line, annotations[0].start_column),
        (1, 2)
    );
}

#[test]
fn player_problems_are_reported_once() {
    let dir = copy_fixture(IRENE, "players");
    let sqm = dir.join("mission.sqm");
    replace_in(
        &sqm,
        "type=\"synixe_factions_synixe_Contractor\"",
        "type=\"B_Soldier_F\"",
        3,
    );
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert_eq!(
        annotations[0].message,
        "All players should be 'synixe_factions_synixe_Contractor' (3 players are not)"
    );
    assert_eq!(annotations[0].level, Level::Error);
    assert!(annotations[0].start_line > 1);
}

/// Attributes of trigger 603, the trigger in Irene that is linked to waypoints
const IRENE_LINKED_TRIGGER: &str =
    "\t\t\t\t\t\tisServerOnly=1;\n\t\t\t\t\t\ttriggerInterval=2;\n\t\t\t\t\t};\n\t\t\t\t\tid=603;";

#[test]
fn trigger_not_server_only() {
    let dir = copy_fixture(IRENE, "trigger");
    replace_in(
        &dir.join("mission.sqm"),
        IRENE_LINKED_TRIGGER,
        &IRENE_LINKED_TRIGGER.replace("isServerOnly=1;", ""),
        1,
    );
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert_eq!(annotations[0].message, "Trigger not set to server only");
    assert!(annotations[0].start_line > 1);
}

#[test]
fn trigger_default_interval_is_too_low() {
    let dir = copy_fixture(IRENE, "interval");
    replace_in(
        &dir.join("mission.sqm"),
        IRENE_LINKED_TRIGGER,
        &IRENE_LINKED_TRIGGER.replace("triggerInterval=2;", ""),
        1,
    );
    assert_eq!(
        messages(&dir),
        vec![
            "Trigger interval is too low (0.5 seconds), it must be at least 0.6 seconds"
                .to_string()
        ]
    );
}

#[test]
fn hold_waypoint_needs_skip_trigger() {
    let dir = copy_fixture(IVORY, "hold");
    replace_in(&dir.join("mission.sqm"), "type=\"SWITCH\";", "", 5);
    let messages = messages(&dir);
    assert!(!messages.is_empty());
    assert!(messages
        .iter()
        .all(|m| m == "HOLD waypoint is linked to a trigger that isn't SKIP WAYPOINT"));
}

#[test]
fn leftover_debug_unit() {
    let dir = copy_fixture(IRENE, "debug");
    let sqm = dir.join("mission.sqm");
    // Play as the first East unit instead of the Contractor slot, like testing in Eden does
    let content = std::fs::read_to_string(&sqm).unwrap();
    let content = content.replacen("isPlayer=1;\n", "", 1);
    let east = content.find("side=\"East\";").unwrap();
    let attributes = east + content[east..].find("class Attributes").unwrap();
    let open = attributes + content[attributes..].find('{').unwrap() + 1;
    let content = format!("{}\nisPlayer=1;{}", &content[..open], &content[open..]);
    std::fs::write(&sqm, content).unwrap();

    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert!(
        annotations[0].message.starts_with("Leftover debug unit: ")
            && annotations[0].message.contains("(East)"),
        "{}",
        annotations[0].message
    );
    assert!(annotations[0].start_line > 1);
}

#[test]
fn magic_information_in_scripts() {
    let dir = copy_fixture(IRENE, "hint");
    std::fs::write(
        dir.join("initServer.sqf"),
        "// systemChat is fine in a comment\nprivate _a = \"hint\";\n  hint \"Go north\";\n",
    )
    .unwrap();
    std::fs::write(dir.join("do_not_edit/ignored.sqf"), "hint \"x\";").unwrap();
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert!(annotations[0].message.starts_with("`hint` gives players"));
    assert!(annotations[0].path.ends_with("initServer.sqf"));
    assert_eq!(
        (annotations[0].start_line, annotations[0].start_column),
        (3, 3)
    );
}

#[test]
fn magic_information_in_sqm() {
    let dir = copy_fixture(IRENE, "sqmchat");
    replace_in(
        &dir.join("mission.sqm"),
        "init=\"{ _x moveInCargo LB_1 } forEach units sof_1;\";",
        "init=\"systemChat \"\"hello\"\"; { _x moveInCargo LB_1 } forEach units sof_1;\";",
        1,
    );
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert!(annotations[0]
        .message
        .starts_with("`systemChat` gives players"));
    assert_eq!(annotations[0].start_line, 7771);
}

#[test]
fn outdated_do_not_edit() {
    let dir = copy_fixture(IRENE, "lobby");
    replace_in(
        &dir.join("do_not_edit/description.ext"),
        "skipLobby = 1;",
        "",
        1,
    );
    assert_eq!(
        messages(&dir),
        vec!["do_not_edit is out of date (missing skipLobby = 1). Copy do_not_edit from a recent mission".to_string()]
    );
}

#[test]
fn scripted_hostiles_are_a_warning() {
    use missionreviewer::checks::{objects::hostiles::HostilesCheck, MissionCheck};
    let dir = Path::new(IRENE);
    let error = HostilesCheck::new(1, false).done(dir);
    assert_eq!(error[0].level, Level::Error);
    let warning = HostilesCheck::new(1, true).done(dir);
    assert_eq!(warning[0].level, Level::Warning);
}

#[test]
fn friendly_independents_are_a_warning() {
    // Like the Christmas special: Independent enemies that Zeus makes hostile
    let dir = copy_fixture(LEAP, "zeus_diplomacy");
    let sqm = dir.join("mission.sqm");
    let content = std::fs::read_to_string(&sqm).unwrap();
    std::fs::write(
        &sqm,
        content.replace("side=\"East\";", "side=\"Independent\";"),
    )
    .unwrap();
    let annotations = check(&dir).unwrap();
    assert_eq!(annotations.len(), 1, "{annotations:?}");
    assert_eq!(annotations[0].level, Level::Warning);
    assert!(annotations[0]
        .message
        .starts_with("Independent units are friendly to West in the editor"));
}
