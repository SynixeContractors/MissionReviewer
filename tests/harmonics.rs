use std::path::PathBuf;

use missionreviewer::{
    checks::run_checks,
    get_class, get_number,
    mission::{read_description, read_mission},
};

#[test]
fn get_time() {
    let (version, _, description) =
        read_description(&PathBuf::from("tests/CO30_Brett_Harmonics.pja308")).unwrap();
    assert_eq!(version, 2);
    assert_eq!(
        get_number(description.config(), "synixe_start_time").map(|(a, _)| a),
        Some(10)
    );
    let (_, mission) = read_mission(&PathBuf::from("tests/CO30_Brett_Harmonics.pja308")).unwrap();
    let mission_intel = get_class(mission.config(), "Mission.Intel").unwrap();
    assert_eq!(get_number(mission_intel, "hour").map(|(a, _)| a), Some(9));
}

#[test]
fn observe_specator() {
    let mission = read_mission(&PathBuf::from("tests/CO30_Brett_Harmonics.pja308")).unwrap();
    let annotations = run_checks(
        &PathBuf::from("tests/CO30_Brett_Harmonics.pja308"),
        vec![Box::new(
            missionreviewer::checks::objects::spectator::RequireSpectator::new(),
        )],
        (&mission.0, mission.1.config()),
    );
    assert_eq!(annotations.len(), 0);
}

#[test]
fn observe_shops() {
    let mission = read_mission(&PathBuf::from("tests/CO30_Brett_Harmonics.pja308")).unwrap();
    let annotations = run_checks(
        &PathBuf::from("tests/CO30_Brett_Harmonics.pja308"),
        vec![Box::new(
            missionreviewer::checks::objects::shops::ShopCheck::new(),
        )],
        (&mission.0, mission.1.config()),
    );
    assert_eq!(annotations.len(), 0);
}
