//! Infer path parameters from a function signature.
//!
//! Currently only the axum-style `Path<...>` extractor is recognized, behind
//! the `axum` feature. Two binding shapes are supported:
//!
//! - `Path(name): Path<T>` — produces a single path parameter named `name`
//!   with schema `T`.
//! - `Path((a, b, ...)): Path<(T1, T2, ...)>` — produces one parameter per
//!   tuple position; the name comes from the binding, the schema from the
//!   corresponding tuple element.
//!
//! Anything more complex (struct extractors, `_` bindings, references, etc.)
//! is silently skipped — callers fall back to declaring the parameters
//! explicitly via `parameters(path(...))`.

#![cfg(feature = "axum")]

use syn::{FnArg, GenericArgument, ItemFn, Pat, PathArguments, Type};

use super::path::Path;

/// Walk the function signature and produce inferred path parameters.
pub(super) fn infer_path_parameters(item_fn: &ItemFn) -> Vec<Path> {
    let mut result = Vec::new();
    for arg in &item_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        let Some(params) = extract_from_arg(&pt.pat, &pt.ty) else {
            continue;
        };
        result.extend(params);
    }
    result
}

fn extract_from_arg(pat: &Pat, ty: &Type) -> Option<Vec<Path>> {
    let inner_ty = unwrap_axum_path_type(ty)?;
    let names = extract_names_from_path_pat(pat)?;

    match inner_ty {
        Type::Tuple(tuple) if names.len() == tuple.elems.len() => {
            let mut params = Vec::with_capacity(names.len());
            for (name, elem_ty) in names.into_iter().zip(tuple.elems.iter()) {
                let schema = type_to_simple_path(elem_ty)?;
                params.push(Path::new_inferred(name, schema));
            }
            Some(params)
        }
        // Tuples with mismatched arity vs binding — skip rather than guess.
        Type::Tuple(_) => None,
        // Single non-tuple type with a single binding name.
        single if names.len() == 1 => {
            let schema = type_to_simple_path(single)?;
            Some(vec![Path::new_inferred(
                names.into_iter().next().unwrap(),
                schema,
            )])
        }
        _ => None,
    }
}

/// If the type is `Path<T>` (last segment ident is `Path` with one generic
/// argument), return the inner type.
fn unwrap_axum_path_type(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else { return None };
    let last = tp.path.segments.last()?;
    if last.ident != "Path" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Pull binding names out of a `Path(...)` pattern. Recognizes:
///   `Path(ident)` → `[ident]`
///   `Path((a, b, ...))` → `[a, b, ...]`
/// Returns `None` for anything else, including patterns containing `_`.
fn extract_names_from_path_pat(pat: &Pat) -> Option<Vec<String>> {
    let Pat::TupleStruct(ts) = pat else {
        return None;
    };
    let last = ts.path.segments.last()?;
    if last.ident != "Path" {
        return None;
    }
    if ts.elems.len() != 1 {
        return None;
    }
    match ts.elems.first()? {
        Pat::Ident(pi) => Some(vec![pi.ident.to_string()]),
        Pat::Tuple(pt) => pt
            .elems
            .iter()
            .map(|e| match e {
                Pat::Ident(pi) => Some(pi.ident.to_string()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// Accept simple type-path schemas (`String`, `u32`, `my::Type`, …) and reject
/// everything else (references, slices, nested tuples). The downstream code
/// expects a `syn::Path` because it splices the type into a turbofish.
fn type_to_simple_path(ty: &Type) -> Option<syn::Path> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => Some(tp.path.clone()),
        _ => None,
    }
}
