# Changelog
All notable changes to this project will be documented in the changelog of the respective crates.
This project follows the [Semantic Versioning standard](https://semver.org/).


## [0.2.1] - 2023-05-09
### Added
 - Serving spec in different formats in `axum` integration using `Accept` header (JSON supported by default, YAML behind `yaml` feature).


## [0.2.0] - 2023-04-29
### Notable changes
 - `axum` integration updated to be used with axum 0.6. Also this makes library unusable with older versions of `axum`.
 
### Changed
 - `axum` integration types updated to be used with axum 0.6.


## [0.1.3] - 2023-02-15
### Added
 - `ComponentesBuilder`. It allows to customize components storage (schemas/security/...), for exmple disable subschemas inlining which could help when you have multiple types with same name (otherwise they will override each other in generated spec);
 - Method `OpenApiBuilder::set_components` for customizing `Components`.


## [0.1.2] - 2022-08-06
### Added
 - Cookie parameters.

### Fixed
 - Macro `openapi_handler` now correctly handle paths.

### Deprecated
 - Macro `openapi_service`, now `openapi_handler` can handle both functions and services.


## [0.1.1] - 2022-07-11
### Fixed
 - docs.rs features.


## [0.1.0] - 2022-07-10
Initial implementation.
