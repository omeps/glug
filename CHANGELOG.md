# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.1.1] - 2024-05-02

### Added

- README.md with a nice picture

- Fixes to examples in `glogger.rs` related to GLoggerOptions

- Fixed crashing on unknown terminal size -- unknown terminal size should panic as it's unusable

## [0.1.0] - 2024-05-02

### Added

- Basic logger with a spawned thread and colors, along with a unique sidebar displaying info

- structs `GLogger`, `GLoggerOptions`, enum `Ansi8`

- `GLogger` methods `setup()`, `setup_with_options()`, `end()`, along with `Log` trait

- `GLoggerOptions` with fields for `colors`, along with `Default` trait
