# Changelog
All notable changes to this project will be documented in the changelog of the respective crates.
This project follows the [Semantic Versioning standard](https://semver.org/).

## [Unreleased] - 2024-07-21
### Added
 - Feature `axum` as replacement for `axum-integration` (now considered deprecated);
 - Request body detection from function arguments for specific frameworks (i.e. axum);
 - `#[body]` attribute as replacement for `#[request_body]` (now considered deprecated).
 

## [0.3.0-rc2] - 2024-07-18
### Add
 - shorter version of `openapi_handler!` macro - `oh!`.

### Changed
 - `#[body]` attribute can be used without braces;
 - `openapi_handler` in axum integration now accept function with generic parameters;
 - switched to using `indexmap` in place of `hashmap` to make produced specs deterministic.

### Fixed
 - handler now accept `accept` header `*/*`.


## [0.3.0-rc1] - 2023-12-03
### Notable changes
 - `axum` integration updated to be used with axum 0.7. Also this makes library unusable with older versions of `axum`;
 - `OpenApiBuilder` rewritten, now providing more safe API to inner specification;
 - Simplified usage of `axum-integration::Router` - it is now unnecessary to provide `OpenApiBuilder`.
 
### Added
 - New methods for `OpenApiBuilder` for setting variuos fields in inner specification;
 - `OpenApiBuilder::build()` method for building specification (replaced `generate_spec()`);
 - `OpenApiBuilder` inside `axum-integration::Router`, which allow to omit explicit vreation of builder;
 - New methods in `axum-integration::Router`:
   - `set_openapi_builder_template` - replace `OpenApiBuilder` inside `Router`;
   - `update_openapi_builder_template` - update `OpenApiBuilder` inside `Router`;
   - `openapi_builder_template_mut` - get mutable reference to `OpenApiBuilder` from `Router`;
   - `generate_openapi_builder` - generate `OpenApiBuilder` from `Router`;
   - (!) `finish_openapi` - builder OpenAPI specification, mount it to path and return `axum::Router` (replaces `route_openapi_specification` method).

### Changed
 - (breaking) `axum` integration types updated to be used with axum 0.7.

### Removed
 - (breaking) `set_openapi_version`, because underlying library compatible only with OpenAPI `3.0.x` (`x` is 0 to 3, changes between versions minor). Now generated specification always have OpenAPI version `3.0.0`;
 - (breaking) Bunch of old methods from `OpenApiBuilder`;
 - (breaking) `axum-integration::Router::route_openapi_specification()` (replaced by `finish_openapi` method).

### Fixed
 - (breaking) `OpenApiBuilder::add_operations` now use passed paths as is. Previously it converted it from `axum` format to OpenAPI, which could mess up integration with another framework. This change does not affect `axum` integration;
 - (breaking) Feature `axum-integration` disabled by default, it was enabled by mistake previously;
 - Minor documentation fixes.
 
 
## [0.2.2] - 2023-12-03
### Fixed
- The `Accept` header parsing in the `axum` integration handler is more relaxed to allow content types such as `+json`, `+yaml`, `text/yaml`, etc.
- Align the behavior of `Router::route` in the `axum` integration to merge routes with same path, rather than overwriting them.


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
