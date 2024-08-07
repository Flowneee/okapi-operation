# Changelog
All notable changes to this project will be documented in the changelog of the respective crates.
This project follows the [Semantic Versioning standard](https://semver.org/).


## [Unreleased] - 2024-07-21
### Added
 - Feature `axum` for enable axum-specific functionality;
 - Request body detection from function arguments for specific frameworks (i.e. axum);
 - `#[body]` attribute as replacement for `#[request_body]` (now considered deprecated);
 - Updates `syn` crate to version 2;
 - `crate` attribute to support renaming base crate, by default `okapi_operation`;
 - `#[openapi]` macro takes care of reimporting necessary types and traits from base crate.


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
