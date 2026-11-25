//! Finding mission folders to review

use std::path::{Path, PathBuf};

/// Folders that hold missions directly
pub const FLAT_FOLDERS: [&str; 2] = ["contracts", "specials"];
/// Folders that hold folders of missions
pub const NESTED_FOLDERS: [&str; 1] = ["theatres"];

/// A mission folder to review
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mission {
    pub path: PathBuf,
    /// Inside a theatre, which allows different prefixes
    pub nested: bool,
}

impl Mission {
    pub fn new(path: PathBuf) -> Self {
        // Show ./contracts/... as contracts/...
        let path = match path.strip_prefix(".") {
            Ok(stripped) if !stripped.as_os_str().is_empty() => stripped.to_path_buf(),
            _ => path,
        };
        let nested = path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .is_some_and(|n| NESTED_FOLDERS.iter().any(|f| n == *f));
        Self { path, nested }
    }
}

/// Sorted subdirectories of a folder, empty if the folder does not exist
pub fn subdirectories(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut dirs = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

/// Whether a folder looks like a mission
pub fn is_mission(dir: &Path) -> bool {
    ["mission.sqm", "description.ext", "edit_me", "do_not_edit"]
        .iter()
        .any(|f| dir.join(f).exists())
}

/// Every mission in the `contracts`, `specials` and `theatres` folders of a missions repository
pub fn scan_repository(root: &Path) -> Vec<Mission> {
    let mut missions = Vec::new();
    for folder in FLAT_FOLDERS {
        missions.extend(
            subdirectories(&root.join(folder))
                .into_iter()
                .map(Mission::new),
        );
    }
    for folder in NESTED_FOLDERS {
        for theatre in subdirectories(&root.join(folder)) {
            missions.extend(subdirectories(&theatre).into_iter().map(Mission::new));
        }
    }
    missions
}

/// Missions for a path given on the command line: a mission, a folder of missions,
/// or a missions repository
pub fn discover(path: &Path) -> Vec<Mission> {
    fn find(path: &Path, depth: usize, out: &mut Vec<Mission>) {
        if is_mission(path) {
            out.push(Mission::new(path.to_path_buf()));
            return;
        }
        if FLAT_FOLDERS
            .iter()
            .chain(NESTED_FOLDERS.iter())
            .any(|f| path.join(f).is_dir())
        {
            out.extend(scan_repository(path));
            return;
        }
        if depth == 0 {
            return;
        }
        for dir in subdirectories(path) {
            find(&dir, depth - 1, out);
        }
    }
    let mut out = Vec::new();
    find(path, 2, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested() {
        assert!(Mission::new(PathBuf::from("theatres/t/CCO30_A.Altis")).nested);
        assert!(!Mission::new(PathBuf::from("contracts/CO30_A.Altis")).nested);
        assert!(!Mission::new(PathBuf::from("CO30_A.Altis")).nested);
        assert_eq!(
            Mission::new(PathBuf::from("./contracts/CO30_A.Altis")).path,
            PathBuf::from("contracts/CO30_A.Altis")
        );
    }

    #[test]
    fn discover_fixtures() {
        let one = discover(Path::new("tests/CO30_Tobias_Irene.NorthTakistan"));
        assert_eq!(one.len(), 1);
        assert_eq!(discover(Path::new("tests")).len(), 4);
    }
}
