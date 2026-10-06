# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-05

### Added
- Initial stable release of ZipTransfer.
- Rewrote the whole backend with Rust.
- Used SQLite for the database (instead of MongoDB).
- Removed all SaaS telemetry, marketing and third-party tracking.
- Simplified the frontend with only the essential stuff for self-hosting and migrated the codebase from javascript to typescript.
- Unified configuration into a single `.env` file.
- Automated GitHub Actions for multi-platform Docker image builds (AMD and ARM) and release notes directly from Changelog.
- Created a selfhosting guide.
- Created a configurable way to connect to your own turn server for improving WebRTC. It can always fall back to Google's free STUN servers.

[Unreleased]: https://github.com/Mapaor/ziptransfer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Mapaor/ziptransfer/releases/tag/v0.1.0
