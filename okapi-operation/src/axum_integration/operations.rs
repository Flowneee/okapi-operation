use std::collections::HashMap;

use axum::http::Method;

use super::method_router::MethodRouterOperations;
use crate::OperationGenerator;

#[derive(Clone, Default)]
pub struct RoutesOperations(pub(super) HashMap<String, HashMap<Method, OperationGenerator>>);

impl RoutesOperations {
    pub(super) fn new(routes_operations: HashMap<String, MethodRouterOperations>) -> Self {
        Self(
            routes_operations
                .into_iter()
                .filter_map(|(path, operations)| {
                    let op_map: HashMap<Method, OperationGenerator> = operations.into_map();
                    if op_map.is_empty() {
                        None
                    } else {
                        Some((path, op_map))
                    }
                })
                .collect(),
        )
    }

    pub fn get(&self, path: &str, method: &Method) -> Option<&OperationGenerator> {
        self.0.get(path).and_then(|x| x.get(method))
    }

    pub fn get_path(&self, path: &str) -> Option<&HashMap<Method, OperationGenerator>> {
        self.0.get(path)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn openapi_operation_generators(&self) -> HashMap<(String, Method), OperationGenerator> {
        self.0
            .iter()
            .flat_map(|(path, methods)| {
                let path = path.clone();
                methods
                    .iter()
                    .map(move |(method, op)| ((path.clone(), method.clone()), *op))
            })
            .collect()
    }
}
