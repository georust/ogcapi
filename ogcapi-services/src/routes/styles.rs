use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::Value;

use ogcapi_types::styles::Styles;

use crate::{AppState, Error, Result};

async fn styles(State(state): State<AppState>) -> Result<Json<Styles>> {
    let styles = state.drivers.styles.list_styles().await?;
    Ok(Json(styles))
}

async fn read_style(Path(id): Path<String>, State(state): State<AppState>) -> Result<Json<Value>> {
    let style = state.drivers.styles.read_style(&id).await?;

    style.map(Json).ok_or(Error::NotFound)
}

pub(crate) fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/styles", get(styles))
        .route("/styles/:id", get(read_style))
}
