//! Infer path parameters from a function signature.
//!
//! Currently only the axum-style `Path<...>` extractor is recognized, behind
//! the `axum` feature. Three binding shapes are supported:
//!
//! - `Path(name): Path<T>` — a single binding. Because the macro only sees the
//!   syntactic type `T` (it cannot tell a struct from a primitive at compile
//!   time), this is resolved at runtime via `Components::infer_path_parameters`:
//!   a struct `T` expands to one parameter per field, while a scalar `T`
//!   produces a single parameter named `name`.
//! - `Path((a, b, ...)): Path<(T1, T2, ...)>` — produces one parameter per
//!   tuple position; the name comes from the binding, the schema from the
//!   corresponding tuple element.
//!
//! Anything more complex (`_` bindings, references, etc.) is silently skipped —
//! callers fall back to declaring the parameters explicitly via
//! `parameters(path(...))`.

#![cfg(feature = "axum")]

use syn::{FnArg, GenericArgument, ItemFn, Pat, PathArguments, Type};

use super::path::Path;

/// A single-binding `Path(name): Path<T>` extractor whose expansion (struct →
/// fields, or scalar → single parameter) is deferred to runtime.
#[derive(Debug)]
pub(super) struct InferredSingle {
    /// Parameter name used when `T` is a scalar.
    pub fallback_name: String,
    /// The `T` in `Path<T>`, spliced into a turbofish at runtime.
    pub ty: syn::Path,
}

/// Inferred path parameters, split by when they can be resolved.
#[derive(Default, Debug)]
pub(super) struct InferredPathParameters {
    /// Tuple elements: names and schemas are known at macro time.
    pub tuple_params: Vec<Path>,
    /// Single bindings: resolved at runtime from `T`'s JSON schema.
    pub singles: Vec<InferredSingle>,
}

/// Walk the function signature and produce inferred path parameters.
pub(super) fn infer_path_parameters(item_fn: &ItemFn) -> InferredPathParameters {
    let mut result = InferredPathParameters::default();
    for arg in &item_fn.sig.inputs {
        let FnArg::Typed(pt) = arg else { continue };
        extract_from_arg(&pt.pat, &pt.ty, &mut result);
    }
    result
}

fn extract_from_arg(pat: &Pat, ty: &Type, out: &mut InferredPathParameters) {
    let Some(inner_ty) = unwrap_axum_path_type(ty) else {
        return;
    };
    let Some(names) = extract_names_from_path_pat(pat) else {
        return;
    };

    match inner_ty {
        Type::Tuple(tuple) if names.len() == tuple.elems.len() => {
            let mut params = Vec::with_capacity(names.len());
            for (name, elem_ty) in names.iter().zip(tuple.elems.iter()) {
                let Some(schema) = type_to_simple_path(elem_ty) else {
                    return;
                };
                params.push(Path::new_inferred(name.clone(), schema));
            }
            out.tuple_params.extend(params);
        }
        // Tuples with mismatched arity vs binding — skip rather than guess.
        Type::Tuple(_) => {}
        // Single non-tuple type with a single binding name. Resolved at runtime.
        single if names.len() == 1 => {
            if let Some(ty) = type_to_simple_path(single) {
                out.singles.push(InferredSingle {
                    fallback_name: names.into_iter().next().unwrap(),
                    ty,
                });
            }
        }
        _ => {}
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
