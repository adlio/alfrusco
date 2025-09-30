# Changelog

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

## [0.1.9]

### Fixed
- Clipboard test contention issues by serializing test runs
- Various clippy warnings and code cleanup
- Improved clipboard test reliability

## [0.1.8]

### Changed
- Replaced `rust-clipboard` dependency with `arboard` for better clipboard support
- Improved clipboard functionality reliability

## [0.1.7]

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

## [0.1.2]

### Added
- Initial stable release with core Alfred workflow functionality
- Basic Item and URLItem support
- Response caching and rerun capabilities
- Background job execution
- Clipboard integration
- Fuzzy filtering and sorting
- Comprehensive error handling