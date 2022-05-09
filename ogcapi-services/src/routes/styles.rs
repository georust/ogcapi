use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    routing::get,
    Json, Router,
};
use serde_json::Value;

use ogcapi_types::styles::Styles;

use crate::{Result, State};

async fn styles(Extension(state): Extension<Arc<State>>) -> Result<Json<Styles>> {
    let styles = state.drivers.styles.list_styles().await?;
    Ok(Json(styles))
}

async fn read_style(
    Path(id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Value>> {
    let style = state.drivers.styles.read_style(&id).await?;

    Ok(Json(style))
}

pub(crate) fn router(_state: &State) -> Router {
    Router::new()
        .route("/styles", get(styles))
        .route("/styles/:id", get(read_style))
}
