# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-01-08

### Added
- Boost feature for controlling item ranking in filtered results
- `Item::boost(value)` method to adjust fuzzy match scores
- Boost constants for common priority levels:
  - `BOOST_SLIGHT` (25) - Subtle preference
  - `BOOST_LOW` (50) - Minor preference
  - `BOOST_MODERATE` (75) - Noticeable preference
  - `BOOST_HIGH` (100) - Strong preference
  - `BOOST_HIGHER` (150) - Very strong preference
  - `BOOST_HIGHEST` (200) - Effectively guarantees top ranking

## [0.2.1] - 2025-09-30

### Added
- `.var()` method support for URLItem - allows setting custom workflow variables on URL items

## [0.2.0] - 2025-09-30

### Added
- Custom `.arg()` support for URLItem - allows URLItems to be both web links and Alfred workflow navigation items
- Complete API consistency: `cache()` and `rerun()` methods now available directly on `Workflow`
- New example demonstrating URLItem with custom args for workflow navigation

### Changed
- Improved API consistency - all response methods now available on `Workflow` directly
- Enhanced documentation with unified API patterns

## [0.1.9] - 2025-06-10

### Fixed
- Clipboard test contention issues by serializing test runs
- Various clippy warnings and code cleanup
- Improved clipboard test reliability

## [0.1.8] - 2025-06-09

### Changed
- Replaced `rust-clipboard` dependency with `arboard` for better clipboard support
- Improved clipboard functionality reliability

## [0.1.7] - 2025-06-09

### Added
- Enhanced background job tracking with detailed status messages
- Background job failure tracking and automatic retry logic
- Workflow command system for internal operations
- Item sticky concept to prevent sorting of specific items
- Data and cache directory helper methods
- Ability to unset workflow variables
- URLItem subtitle customization support

### Changed
- Enhanced background job status reporting with timestamps and duration
- Improved error reporting for missing environment variables
- Better logging initialization error handling

### Fixed
- Background job status tracking and display
- Log initialization failures that were previously swallowed
- Compatibility with newer sysinfo versions
- Duration serialization in responses

## [0.1.2] - 2024-09-15

### Added
- Initial stable release with core Alfred workflow functionality
- Basic Item and URLItem support
- Response caching and rerun capabilities
- Background job execution
- Clipboard integration
- Fuzzy filtering and sorting
- Comprehensive error handling

[Unreleased]: https://github.com/adlio/alfrusco/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/adlio/alfrusco/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/adlio/alfrusco/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/adlio/alfrusco/compare/v0.1.9...v0.2.0
[0.1.9]: https://github.com/adlio/alfrusco/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/adlio/alfrusco/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/adlio/alfrusco/compare/v0.1.2...v0.1.7
[0.1.2]: https://github.com/adlio/alfrusco/releases/tag/v0.1.2
