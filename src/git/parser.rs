use crate::error::GitResult;

/// Parse git status --porcelain=v2 output
pub fn parse_status_porcelain_v2(output: &str) -> GitResult<Vec<StatusEntry>> {
    let mut entries = Vec::new();

    for line in output.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "1" | "2" => {
                // Tracked entry format: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                if parts.len() >= 9 {
                    let xy = parts[1];
                    let path = parts[8..].join(" ");

                    let status = match xy {
                        "M." => FileStatus::Modified,
                        ".M" => FileStatus::Modified,
                        "MM" => FileStatus::Modified,
                        "A." => FileStatus::Added,
                        ".A" => FileStatus::Added,
                        "D." => FileStatus::Deleted,
                        ".D" => FileStatus::Deleted,
                        _ => FileStatus::Unknown,
                    };

                    let staged = !xy.starts_with('.');
                    let unstaged = xy.chars().nth(1).is_some_and(|c| c != '.');

                    entries.push(StatusEntry {
                        status,
                        path,
                        staged,
                        unstaged,
                    });
                }
            }
            "?" => {
                // Untracked file: ? <path>
                if parts.len() >= 2 {
                    let path = parts[1..].join(" ");
                    entries.push(StatusEntry {
                        status: FileStatus::Untracked,
                        path,
                        staged: false,
                        unstaged: false,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(entries)
}

/// Parse git log output with format %H%x00%s
pub fn parse_log(output: &str) -> GitResult<Vec<CommitEntry>> {
    let mut commits = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() >= 2 {
            commits.push(CommitEntry {
                hash: parts[0].to_string(),
                message: parts[1].to_string(),
            });
        } else if parts.len() == 1 {
            // Handle case where there's no message
            commits.push(CommitEntry {
                hash: parts[0].to_string(),
                message: String::new(),
            });
        }
    }

    Ok(commits)
}

/// Parse git branch -vv output with format
pub fn parse_branch_list(output: &str) -> GitResult<Vec<BranchEntry>> {
    let mut branches = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        // Format: * main abc123 [origin/main: ahead 2, behind 1] Commit message
        // or:     feature-x def456 Feature work
        let is_current = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();

        branches.push(BranchEntry {
            name,
            is_current,
        });
    }

    Ok(branches)
}

/// Parse git stash list output with format %gd%x00%s
pub fn parse_stash_list(output: &str) -> GitResult<Vec<StashEntry>> {
    let mut stashes = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() >= 2 {
            stashes.push(StashEntry {
                index: parts[0].to_string(),
                message: parts[1].to_string(),
            });
        }
    }

    Ok(stashes)
}

/// Represents a file status entry from git status
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub status: FileStatus,
    pub path: String,
    pub staged: bool,
    pub unstaged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Unknown,
}

/// Represents a commit from git log
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitEntry {
    pub hash: String,
    pub message: String,
}

/// Represents a branch from git branch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEntry {
    pub name: String,
    pub is_current: bool,
}

/// Represents a stash entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StashEntry {
    pub index: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_modified_staged() {
        let output = "1 M. N... 100644 100644 100644 abc123 def456 README.md";
        let entries = parse_status_porcelain_v2(output).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "README.md");
        assert_eq!(entries[0].status, FileStatus::Modified);
        assert!(entries[0].staged);
        assert!(!entries[0].unstaged);
    }

    #[test]
    fn test_parse_status_modified_unstaged() {
        let output = "1 .M N... 100644 100644 100644 abc123 def456 src/main.rs";
        let entries = parse_status_porcelain_v2(output).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "src/main.rs");
        assert!(!entries[0].staged);
        assert!(entries[0].unstaged);
    }

    #[test]
    fn test_parse_status_untracked() {
        let output = "? untracked.txt";
        let entries = parse_status_porcelain_v2(output).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "untracked.txt");
        assert_eq!(entries[0].status, FileStatus::Untracked);
    }

    #[test]
    fn test_parse_log() {
        let output = "abc123\0Initial commit\ndef456\0Add README";
        let commits = parse_log(output).unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].message, "Initial commit");
        assert_eq!(commits[1].hash, "def456");
        assert_eq!(commits[1].message, "Add README");
    }

    #[test]
    fn test_parse_log_empty_message() {
        let output = "abc123\0";
        let commits = parse_log(output).unwrap();

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].message, "");
    }

    #[test]
    fn test_parse_branch_current() {
        let output = "* main\n  feature-x";
        let branches = parse_branch_list(output).unwrap();

        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_current);
        assert_eq!(branches[1].name, "feature-x");
        assert!(!branches[1].is_current);
    }

    #[test]
    fn test_parse_stash_list() {
        let output = "stash@{0}\0WIP on main: fix bug\nstash@{1}\0Experimental feature";
        let stashes = parse_stash_list(output).unwrap();

        assert_eq!(stashes.len(), 2);
        assert_eq!(stashes[0].index, "stash@{0}");
        assert_eq!(stashes[0].message, "WIP on main: fix bug");
        assert_eq!(stashes[1].index, "stash@{1}");
        assert_eq!(stashes[1].message, "Experimental feature");
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_status_porcelain_v2("").unwrap().len(), 0);
        assert_eq!(parse_log("").unwrap().len(), 0);
        assert_eq!(parse_branch_list("").unwrap().len(), 0);
        assert_eq!(parse_stash_list("").unwrap().len(), 0);
    }
}
