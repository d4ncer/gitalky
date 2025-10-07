use crate::error::{GitError, GitResult};
use std::process::Command;

/// Minimum required git version
const MIN_GIT_VERSION: (u32, u32) = (2, 20);

/// Represents a git version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    /// Detect the installed git version
    pub fn detect() -> GitResult<Self> {
        let output = Command::new("git")
            .arg("--version")
            .output()
            .map_err(|e| GitError::GitVersionDetectionFailed(format!("Failed to execute git: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::GitVersionDetectionFailed(
                "git --version command failed".to_string(),
            ));
        }

        let version_string = String::from_utf8_lossy(&output.stdout);
        Self::parse(&version_string)
    }

    /// Parse git version from string like "git version 2.39.2"
    pub fn parse(version_str: &str) -> GitResult<Self> {
        // Expected format: "git version X.Y.Z" or "git version X.Y.Z.windows.1" etc.
        let parts: Vec<&str> = version_str.split_whitespace().collect();

        if parts.len() < 3 || parts[0] != "git" || parts[1] != "version" {
            return Err(GitError::ParseError(format!(
                "Unexpected git version format: {}",
                version_str
            )));
        }

        let version_nums = parts[2];
        let nums: Vec<&str> = version_nums.split('.').collect();

        if nums.len() < 2 {
            return Err(GitError::ParseError(format!(
                "Invalid version number format: {}",
                version_nums
            )));
        }

        let major = nums[0]
            .parse::<u32>()
            .map_err(|_| GitError::ParseError(format!("Invalid major version: {}", nums[0])))?;

        let minor = nums[1]
            .parse::<u32>()
            .map_err(|_| GitError::ParseError(format!("Invalid minor version: {}", nums[1])))?;

        let patch = if nums.len() >= 3 {
            nums[2]
                .parse::<u32>()
                .unwrap_or(0) // Allow patch version to have non-numeric suffixes
        } else {
            0
        };

        Ok(GitVersion {
            major,
            minor,
            patch,
        })
    }

    /// Check if this version meets minimum requirements
    pub fn is_supported(&self) -> bool {
        self.major > MIN_GIT_VERSION.0
            || (self.major == MIN_GIT_VERSION.0 && self.minor >= MIN_GIT_VERSION.1)
    }

    /// Validate that git version is sufficient
    pub fn validate() -> GitResult<Self> {
        let version = Self::detect()?;

        if !version.is_supported() {
            return Err(GitError::GitVersionTooOld(format!(
                "{}.{}.{}\n\nPlease upgrade git to version {}.{} or higher.\nVisit: https://git-scm.com/downloads",
                version.major,
                version.minor,
                version.patch,
                MIN_GIT_VERSION.0,
                MIN_GIT_VERSION.1
            )));
        }

        Ok(version)
    }
}

impl std::fmt::Display for GitVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_version() {
        let version = GitVersion::parse("git version 2.39.2").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 39);
        assert_eq!(version.patch, 2);
    }

    #[test]
    fn test_parse_version_with_suffix() {
        let version = GitVersion::parse("git version 2.39.2.windows.1").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 39);
        assert_eq!(version.patch, 2);
    }

    #[test]
    fn test_parse_version_no_patch() {
        let version = GitVersion::parse("git version 2.39").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 39);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(GitVersion::parse("version 2.39.2").is_err());
        assert!(GitVersion::parse("git 2.39.2").is_err());
        assert!(GitVersion::parse("random string").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = GitVersion { major: 2, minor: 20, patch: 0 };
        let v2 = GitVersion { major: 2, minor: 39, patch: 2 };
        let v3 = GitVersion { major: 3, minor: 0, patch: 0 };

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_is_supported() {
        assert!(GitVersion { major: 2, minor: 20, patch: 0 }.is_supported());
        assert!(GitVersion { major: 2, minor: 21, patch: 0 }.is_supported());
        assert!(GitVersion { major: 3, minor: 0, patch: 0 }.is_supported());

        assert!(!GitVersion { major: 2, minor: 19, patch: 9 }.is_supported());
        assert!(!GitVersion { major: 1, minor: 9, patch: 0 }.is_supported());
    }

    #[test]
    fn test_display() {
        let version = GitVersion { major: 2, minor: 39, patch: 2 };
        assert_eq!(format!("{}", version), "2.39.2");
    }
}
