use tracing::error;
use url::Url;

/// `url` with `segments` added to its path, each a segment of its own (an id
/// holding `/`, `?` or `#` addresses no other resource), without its query.
pub(crate) fn url_plus_segments(mut url: Url, segments: &[&str]) -> anyhow::Result<Url> {
    url.set_query(None);
    url.set_fragment(None);
    let Ok(mut path) = url.path_segments_mut() else {
        anyhow::bail!("Cannot modify path segments of a URL without a base");
    };
    path.pop_if_empty().extend(segments);
    drop(path);
    Ok(url)
}

/// `url` with its last `remove` segments replaced by `segments`, as
/// [`url_plus_segments`] adds them.
#[cfg(any(feature = "edr", feature = "processes"))]
pub(crate) fn url_replace_segments(
    mut url: Url,
    remove: usize,
    segments: &[&str],
) -> anyhow::Result<Url> {
    let Ok(mut path) = url.path_segments_mut() else {
        anyhow::bail!("Cannot modify path segments of a URL without a base");
    };
    path.pop_if_empty();
    for _ in 0..remove {
        path.pop();
    }
    drop(path);
    url_plus_segments(url, segments)
}

/// Helper function to read-lock a `RwLock`, recovering from poisoning if necessary.
pub fn read_lock<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("RwLock was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to write-lock a `RwLock`, recovering from poisoning if necessary.
pub fn write_lock<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("RwLock was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}
