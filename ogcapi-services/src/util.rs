use tracing::error;

/// Helper function to read-lock a RwLock, recovering from poisoning if necessary.
pub(crate) fn read_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match mutex.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to write-lock a RwLock, recovering from poisoning if necessary.
pub(crate) fn write_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match mutex.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// The original `routes!` macro from `utoipa_axum` does not support generics well.
/// This is a workaround to allow using it with our `OgcApiState` generic parameter.
#[macro_export]
macro_rules! routes2 {
    ( $handler:path $(, $tail:path)* $(,)? ) => {
        {
            use utoipa_axum::PathItemExt;
            let mut paths = utoipa::openapi::path::Paths::new();
            let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            let (path, item, types) = utoipa_axum::routes!(@resolve_types $handler : schemas);
            #[allow(unused_mut)]
            let mut method_router = types.iter().by_ref().fold(axum::routing::MethodRouter::new(), |router, path_type| {
                router.on(path_type.to_method_filter(), utoipa_axum::paste! { $handler :: <S> })
            });
            paths.add_path_operation(&path, types, item);
            // $( method_router = utoipa_axum::routes!( schemas: method_router: paths: $tail ); )*
            (schemas, paths, method_router)
        }
    };
}
