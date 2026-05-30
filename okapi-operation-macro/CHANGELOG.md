# Changelog

All notable changes to this project will be documented in the changelog of the respective crates.
This project follows the [Semantic Versioning standard](https://semver.org/).

## [Unreleased]

### Added

- Path parameters are now inferred from the function signature when the `axum`
  feature is enabled. The axum `Path<...>` extractor is recognized in two
  forms: `Path(name): Path<T>` produces a single path parameter, and
  `Path((a, b, ...)): Path<(T1, T2, ...)>` produces one parameter per tuple
  position. Parameters declared explicitly via `parameters(path(...))` win
  over inferred ones with the same name, so existing code is unaffected.

### Fixed

- `parameters(cookie(...))` entries are now actually emitted into the
  generated operation. Previously cookie parameters were parsed but silently
  dropped, so declared cookies never appeared in the spec.
- The cookie style field is now wrapped in `Some(...)`, fixing a compile
  error that previously prevented `parameters(cookie(...))` from being used
  at all.
- The cookie parameter `location` is now `"cookie"` (lowercase) as required
  by OpenAPI 3.0.
- Support `content` field for `path`, `query`, `header`, and `cookie` parameters — allows specifying a parameter value via `ParameterValue::Content` (a media type map) instead of `ParameterValue::Schema`.

## [0.3.0] - 2025-06-15

Release `0.3.0` version.

## [0.3.0-rc4] - 2025-03-13

### Changed

- Use same version as `okapi-operation`.

## [0.2.0] - 2024-08-07

### Added

- Feature `axum` for enable axum-specific functionality;
- Request body detection from function arguments for specific frameworks (i.e. axum);
- `#[body]` attribute as replacement for `#[request_body]` (now considered deprecated);
- Updates `syn` crate to version 2;
- `crate` attribute to support renaming base crate, by default `okapi_operation`;
- `#[openapi]` macro takes care of reimporting necessary types and traits from base crate.

### Removed

- Support for multiple `#[openapi]` appearances above function.

## [0.1.4] - 2024-07-18

### Changed

- `#[request_body]` attribute can be used without braces.

## [0.1.3] - 2023-04-29

### Changed

- `axum` bumped to `0.6`.

## [0.1.2] - 2023-03-07

### Changed

- Used version 0.14.3 of `darling`.

## [0.1.1] - 2022-08-06

### Added

- Cookie parameters.

## [0.1.0] - 2022-07-10

Initial implementation.
