# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1(-bc)] - 2025-04-29

### Added

- `builder` arg for generating a builder struct


## [0.3.0-bc] - 2025-03-24

### Breaking

- create older-rustc-version-compatible versions of the crate behind a `-bc` pre-release version


## [0.2.0] - 2025-03-23

### Breaking

- new visibility attribute arg (`#[generic_array_struct(pub(crate))]`) means `.0` field is private by default if attribute is used without any args (`#[generic_array_struct]`)

### Added

- `*_mut(&mut self) -> &mut T` methods


## [0.1.0] - 2025-03-22

Initial release
