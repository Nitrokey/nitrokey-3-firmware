# Maintenance Guide

This guide explains how to keep this repository up to date and how to make releases.

## Updating the Repository

### Upstream Changes

Regularly check the [solo2](https://github.com/solokeys/solo2) for relevant changes and merge them into this repository.  Before pushing the merge, make sure to run the build so that the `Cargo.lock` file is updated.  If it changed, add it to the merge commit.

### Dependencies

As we have a `Cargo.lock` file with fixed dependency versions, we don’t automatically pull in new dependency versions.  To update the version of a specific dependency, run `cargo update -p <name>`.  To update all dependencies, run `cargo update`.

## Releasing

To release a stable release or release candidate of the firmware, perform the following steps:
1. Update the version counter in `Cargo.toml` using the patterns `<major>.<minor>.<patch>` or `<major>.<minor>.<patch>-rc.<n>`.
2. Run the firmware build for the embedded runner and add the updated `Cargo.lock`.
3. Update the changelog.
4. Commit all changed files, create a PR and merge it once reviewed.
5. Create a signed tag with a `v` prefix and the version number, for example `v1.0.0` or `v1.5.0-rc.0`.

To release a test release, just create a signed tag with the version number using the pattern `v<major>.<minor>.<patch>-test.<yyyy><mm><dd>`, for example `v1.5.0-test.20231106`.

Refer to the internal documentation for more details on the release process.

## Forking Dependencies

If it is necessary to fork dependencies, please use the following guidelines:
- Create the fork in the Nitrokey namespace on Github and use its main branch.
- Try to create upstream PRs for all changes in the fork.
- Create a tracking issue that lists these upstream PRs ([example](https://github.com/Nitrokey/fido-authenticator/issues/5)).
- When patching `nitrokey-3-firmware` to use the fork, use a tag to specify the dependency version.  This tag should have the format `v0.1.0-nitrokey.1`, where `v0.1.0` is the latest upstream version and `1` can be incremented if more changes are added to the fork.
