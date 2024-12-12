use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::RwLock,
};

use missionreviewer::annotation::{Annotation, Level};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const FLAT_FOLDERS: [&str; 2] = ["contracts", "specials"];
const NESTED_FOLDERS: [&str; 2] = ["campaigns", "theatres"];

fn main() {
    let messages = RwLock::new(Vec::new());

    let prefixes = std::env::args().skip(1).collect::<Vec<String>>();

    let mut missions = Vec::new();

    for folder in FLAT_FOLDERS {
        if !PathBuf::from(folder).exists() {
            continue;
        }
        for mission in std::fs::read_dir(folder).unwrap() {
            let mission = mission.unwrap();
            if !mission.path().is_dir() {
                continue;
            }
            if prefixes.is_empty()
                || prefixes
                    .iter()
                    .any(|prefix| mission.path().display().to_string().contains(prefix))
            {
                missions.push(mission.path());
            }
        }
    }
    for folder in NESTED_FOLDERS {
        if !PathBuf::from(folder).exists() {
            continue;
        }
        for subfolder in std::fs::read_dir(folder).unwrap() {
            let subfolder = subfolder.unwrap();
            if !subfolder.path().is_dir() {
                continue;
            }
            for mission in std::fs::read_dir(subfolder.path()).unwrap() {
                let mission = mission.unwrap();
                if !mission.path().is_dir() {
                    continue;
                }
                if prefixes.is_empty()
                    || prefixes
                        .iter()
                        .any(|prefix| mission.path().display().to_string().contains(prefix))
                {
                    missions.push(mission.path());
                }
            }
        }
    }

    missions.par_iter().for_each(|mission| {
        if !mission.is_dir() {
            return;
        }
        if let Err(e) = check_prefix(mission, false) {
            eprintln!("{}", e);
        }
        match missionreviewer::mission::check(mission) {
            Err(e) => {
                eprintln!("{}", e);
            }
            Ok(m) => {
                messages.write().unwrap().extend(m);
            }
        }
    });

    let messages = messages.read().unwrap();
    let mut out = std::fs::File::create("missionreviewer.log").unwrap();
    for message in messages.iter() {
        out.write_all(message.line().as_bytes()).unwrap();
    }
    println!("Wrote {} messages to missionreviewer.log", messages.len());
}

fn check_prefix(dir: &Path, nested: bool) -> Result<Vec<Annotation>, String> {
    let mut messages = vec![];
    if !dir.is_dir() {
        messages.push(Annotation::new(
            None,
            dir.to_path_buf().display().to_string(),
            0..0,
            "Not a directory".to_string(),
            Level::Error,
        ));
        return Ok(messages);
    }
    let allowed = match nested {
        true => {
            vec!["CCO", "TCO", "TT"]
        }
        false => {
            vec!["CO", "SCO", "TRA"]
        }
    };
    if !allowed.iter().any(|prefix| {
        dir.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(*prefix)
    }) {
        messages.push(Annotation::new(
            None,
            dir.to_path_buf().display().to_string(),
            0..0,
            format!("Invalid prefix for {}", dir.display()),
            Level::Error,
        ));
    }
    Ok(messages)
}
