use axum::extract::{Extension, Path};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::common::core::Links;
use crate::server::{Result, State};
use crate::styles::{Style, Styles, Stylesheet};

async fn styles(Extension(state): Extension<State>) -> Result<Json<Styles>> {
    let styles = sqlx::query_as!(
        Style,
        r#"SELECT id, title, links as "links: sqlx::types::Json<Links>" FROM meta.styles"#
    )
    .fetch_all(&state.db.pool)
    .await?;
    Ok(Json(Styles { styles }))
}

async fn read_style(
    Path(id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Value>> {
    let style = sqlx::query_as!(
        Stylesheet,
        r#"SELECT id, value as "value: sqlx::types::Json<Value>" FROM meta.styles WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(style.value.0))
}

pub(crate) fn router(_state: &State) -> Router {
    Router::new()
        .route("/styles", get(styles))
        .route("/styles/:id", get(read_style))
}
