# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-09-30

### Added
- Custom `.arg()` support for URLItem - allows URLItems to be both web links and Alfred workflow navigation items
- Complete API consistency: `cache()` and `rerun()` methods now available directly on `Workflow`
- New example demonstrating URLItem with custom args for workflow navigation

### Changed
- **BREAKING**: Improved API consistency - all response methods now available on `Workflow` directly
- Removed emoji from features list in README for cleaner documentation
- Updated all examples to use consistent `workflow.method()` pattern instead of `workflow.response.method()`
- Enhanced documentation with unified API patterns

### Fixed
- API inconsistency where some methods required `workflow.response.` while others used `workflow.`

## [0.1.9] - 2024-12-XX

### Fixed
- Clipboard test contention issues by serializing test runs
- Various clippy warnings and code cleanup

### Changed
- Improved clipboard test reliability

## [0.1.8] - 2024-12-XX

### Changed
- **BREAKING**: Replaced `rust-clipboard` dependency with `arboard` for better clipboard support
- Improved clipboard functionality reliability

## [0.1.7] - 2024-12-XX

### Added
- Enhanced background job tracking with detailed status messages
- Background job failure tracking and automatic retry logic
- Workflow command system for internal operations
- Item sticky concept to prevent sorting of specific items
- Data and cache directory helper methods
- Ability to unset workflow variables
- URLItem subtitle customization support

### Changed
- Consolidated and improved test suite coverage
- Enhanced background job status reporting with timestamps and duration
- Improved error reporting for missing environment variables
- Better logging initialization error handling

### Fixed
- Background job status tracking and display
- Log initialization failures that were previously swallowed
- Compatibility with newer sysinfo versions
- Duration serialization in responses

### Removed
- Unnecessary configuration flags
- Unused dependencies

## [0.1.2] - 2024-XX-XX

### Added
- Initial stable release with core Alfred workflow functionality
- Basic Item and URLItem support
- Response caching and rerun capabilities
- Background job execution
- Clipboard integration
- Fuzzy filtering and sorting
- Comprehensive error handling
- Testing utilities and examples

[Unreleased]: https://github.com/adlio/alfrusco/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/adlio/alfrusco/compare/v0.1.9...v0.2.0
[0.1.9]: https://github.com/adlio/alfrusco/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/adlio/alfrusco/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/adlio/alfrusco/compare/v0.1.2...v0.1.7
[0.1.2]: https://github.com/adlio/alfrusco/releases/tag/v0.1.2