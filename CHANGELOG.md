# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-01-21

### Fixed
- Fixed deserialization of `isDueCompleted` field in Card struct - API returns `isDueDateCompleted` instead of `isDueCompleted`
- Added `#[serde(rename = "isDueDateCompleted")]` attribute to correctly map the field from Planka API response
- Applied the same fix to CreateCardRequest struct for consistency

## [0.1.0] - Initial Release

### Added
- Initial implementation of Planka MCP server
- Support for projects, boards, lists, and cards operations
- Token and email/password authentication
- Docker support
- DISABLE_SSL option for self-signed certificates
