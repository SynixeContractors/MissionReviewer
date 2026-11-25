use std::{
    io::{IsTerminal, Write},
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::ExitCode,
};

use missionreviewer::{
    annotation::{Annotation, Level},
    discover::{discover, scan_repository, Mission},
    maps::{find_maps_txt, missing_maps, MISSING_MAPS},
    report::{plural, render, summary, Sources, Style},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const LOG: &str = "missionreviewer.log";

const HELP: &str = "\
Review Synixe missions for common mistakes

Usage:
  missionreviewer            Review every mission in contracts/, specials/ and theatres/
                             and write the results to missionreviewer.log (used by CI)
  missionreviewer <PATH>...  Review missions and print the results. A path can be a mission,
                             a folder of missions, or a missions repository

Options:
  -h, --help     Show this help
  -V, --version  Show the version

Exits with 1 when a reviewed mission has errors (only when paths are given).
Set NO_COLOR to turn off colors.";

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("missionreviewer {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if let Some(flag) = args.iter().find(|a| a.starts_with('-')) {
        eprintln!("Unknown option {flag}\n\n{HELP}");
        return ExitCode::from(2);
    }
    if args.is_empty() {
        ci();
        ExitCode::SUCCESS
    } else {
        cli(&args)
    }
}

/// Review the whole repository and write the annotations for the action
fn ci() {
    // An empty root keeps paths relative (contracts/...), the action matches them to changed files
    let missions = scan_repository(Path::new(""));
    let mut messages = missions
        .par_iter()
        .flat_map_iter(|mission| {
            println!("Checking {}", mission.path.display());
            review(mission)
        })
        .collect::<Vec<_>>();
    sort(&mut messages);

    let mut out = std::fs::File::create(LOG).unwrap();
    for message in &messages {
        out.write_all(message.line().as_bytes()).unwrap();
    }
    println!("Wrote {} messages to {LOG}", messages.len());

    // The action opens a pull request adding these to maps.txt
    let _ = std::fs::remove_file(MISSING_MAPS);
    if let Some(maps) = maps_missing_from_list(&missions) {
        if !maps.is_empty() {
            println!("Maps missing from maps.txt: {}", maps.join(", "));
            std::fs::write(MISSING_MAPS, maps.join("\n") + "\n").unwrap();
        }
    }
}

/// Maps the missions use that aren't in maps.txt, `None` if there is no maps.txt
fn maps_missing_from_list(missions: &[Mission]) -> Option<Vec<String>> {
    let content = std::fs::read_to_string(find_maps_txt(missions)?).ok()?;
    Some(missing_maps(&content, missions))
}

/// Review the given paths and print the results
fn cli(args: &[String]) -> ExitCode {
    let mut missions = Vec::new();
    for arg in args {
        let path = PathBuf::from(arg);
        if !path.is_dir() {
            eprintln!("{arg} is not a folder");
            return ExitCode::from(2);
        }
        let found = discover(&path);
        if found.is_empty() {
            eprintln!("No missions found in {arg}");
            return ExitCode::from(2);
        }
        missions.extend(found);
    }
    missions.sort();
    missions.dedup();

    let results = missions
        .par_iter()
        .map(|mission| {
            let mut messages = review(mission);
            sort(&mut messages);
            (mission, messages)
        })
        .collect::<Vec<_>>();

    let style =
        Style::new(std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none());
    let mut sources = Sources::default();
    let mut stdout = std::io::stdout().lock();
    let (mut total_errors, mut total_warnings, mut failed) = (0, 0, 0);
    for (mission, messages) in &results {
        let (errors, warnings) = summary(messages);
        total_errors += errors;
        total_warnings += warnings;
        if errors > 0 {
            failed += 1;
        }
        if messages.is_empty() {
            let _ = writeln!(stdout, "{}: ok", mission.path.display());
            continue;
        }
        let _ = writeln!(
            stdout,
            "{}: {}, {}\n",
            mission.path.display(),
            plural(errors, "error"),
            plural(warnings, "warning")
        );
        for message in messages {
            let _ = writeln!(stdout, "{}", render(message, &mut sources, &style));
        }
    }
    let _ = writeln!(
        stdout,
        "\nReviewed {}: {}, {}{}",
        plural(missions.len(), "mission"),
        plural(total_errors, "error"),
        plural(total_warnings, "warning"),
        if failed > 0 && missions.len() > 1 {
            format!(" ({} with errors)", plural(failed, "mission"))
        } else {
            String::new()
        }
    );
    if let Some(maps) = maps_missing_from_list(&missions) {
        if !maps.is_empty() {
            let _ = writeln!(
                stdout,
                "Maps missing from maps.txt (CI opens a pull request to add them): {}",
                maps.join(", ")
            );
        }
    }
    if total_errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// All annotations for one mission. A panic is reported on the mission instead of
/// stopping the others from being reviewed
fn review(mission: &Mission) -> Vec<Annotation> {
    let mut messages = missionreviewer::mission::check_prefix(&mission.path, mission.nested);
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        missionreviewer::mission::check(&mission.path)
    })) {
        Ok(Ok(m)) => messages.extend(m),
        Ok(Err(e)) => eprintln!("{}: {}", mission.path.display(), e),
        Err(panic) => {
            let reason = panic
                .downcast_ref::<&str>()
                .map(ToString::to_string)
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_string());
            eprintln!("{}: reviewer crashed: {}", mission.path.display(), reason);
            messages.push(Annotation::new(
                None,
                mission.path.join("mission.sqm").display().to_string(),
                0..0,
                format!("The mission reviewer crashed while checking this mission: {reason}"),
                Level::Error,
            ));
        }
    }
    messages
}

fn sort(messages: &mut Vec<Annotation>) {
    messages.sort_by(|a, b| {
        (&a.path, a.start_line, a.start_column, &a.message).cmp(&(
            &b.path,
            b.start_line,
            b.start_column,
            &b.message,
        ))
    });
    messages.dedup();
}
