# CHANGELOG for `filetest`
This file keeps track of notable changes to the `filetest`-crate.

This project uses [semantic versioning](https://semver.org). As such, breaking changes are indicated by **(BREAKING)**.


## v0.1.1 - 2026-02-05
### Fixed
- The system now also finds tests if you use `.` or `..` in paths before the last file.
- Now generating unique names for unit tests, always.


## v0.1.0 - 2026-02-04
Initial release!

### Added
- The `#[file_ab_test]` attribute macro to generate unit tests for A/B input/gold file pairs.
