use std::process::Command;

use chrono::{Datelike as _, Local};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};

// We have a crate version, as defined in the Cargo.toml, and a firmware version, as reported by
// the device e. g. in the admin app.  The firmware version is determined using these steps:
//
// 1. The crate version is either <major>.<minor>.<patch> or <major>.<minor>.<patch>-rc.<n>.
// 2. If the test feature is activated, the prerelease component for the firmware version is set to
//    test.<yyyy><mm><dd>.
// 3. If built from a tag in the CI, the tag name must be a semver version with the v prefix.  The
//    major, minor and patch components must match the crate version, the prerelease component must
//    match the crate version or the prerelease component computed in step (2), and the build
//    metadata must be empty.
// 4. Unless the firmware is built in the release CI, the build metadata for the firmware version
//    is set to git-<commit> or git-<commit>-dirty.
//
// The environment variable NK3_FIRMWARE_VERSION is set to the calculated firmware version.

const PATTERN_PRE: &str = r"rc\.\d+";

fn main() {
    let mut version = crate_version();
    let test_prerelease = test_prerelease();

    if let Some(tag_version) = tag_version() {
        assert_eq!(
            tag_version.major, version.major,
            "bad major component in tag"
        );
        assert_eq!(
            tag_version.minor, version.minor,
            "bad minor component in tag"
        );
        assert_eq!(
            tag_version.patch, version.patch,
            "bad patch component in tag"
        );
        assert!(
            tag_version.pre == version.pre || tag_version.pre == test_prerelease,
            "prerelease component {} in tag must be one of: {}, {}",
            tag_version.pre,
            version.pre,
            test_prerelease,
        );
        assert_eq!(
            tag_version.build,
            BuildMetadata::EMPTY,
            "build metadata in tag must be empty"
        );
    }

    if cfg!(feature = "test") {
        // We intentionally overwrite the rc.<n> prerelease component if it is set
        version.pre = test_prerelease;
    }
    if !is_release() {
        if let Some(build_metadata) = build_metadata() {
            version.build = build_metadata;
        }
    }

    println!("cargo:rustc-env=NK3_FIRMWARE_VERSION={version}");
}

fn crate_version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse crate version");
    assert!(
        version.build.is_empty(),
        "crate version may not have build metadata: {version}"
    );
    if !version.pre.is_empty() {
        let r = Regex::new(PATTERN_PRE).unwrap();
        assert!(
            r.is_match(version.pre.as_str()),
            "unexpected pre version: {}",
            version.pre
        );
    }
    version
}

fn tag_version() -> Option<Version> {
    option_env!("CI_COMMIT_TAG")
        .map(|s| s.strip_prefix("v").expect("tag must start with v"))
        .map(|s| Version::parse(s).expect("failed to parse version from tag"))
}

fn test_prerelease() -> Prerelease {
    let now = Local::now();
    let pre = format!("test.{:04}{:02}{:02}", now.year(), now.month(), now.day());
    pre.parse().unwrap()
}

fn build_metadata() -> Option<BuildMetadata> {
    // We want to get the latest commit and whether the working tree is dirty.  Apparently, there
    // is no easy machine-readble way to do the latter so we use git-describe with --exclude * so
    // that we don’t get tag names, only commit shas.
    Command::new("git")
        .args(["describe", "--always", "--dirty", "--exclude", "*"])
        .output()
        .ok()
        .map(|output| String::from_utf8(output.stdout).expect("invalid output from git describe"))
        .map(|s| format!("git.{}", s.trim().replace('-', ".")))
        .map(|s| s.parse().unwrap())
}

fn is_release() -> bool {
    option_env!("CI_PIPELINE_SOURCE") == Some("web")
}
