use axum::extract::{Extension, Path};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::{Result, State};
use ogcapi_types::styles::{Style, Styles, Stylesheet};

async fn styles(Extension(state): Extension<State>) -> Result<Json<Styles>> {
    let styles = sqlx::query_scalar!(
        r#"
        SELECT array_to_json(array_agg(row_to_json(t))) as "styles!: sqlx::types::Json<Vec<Style>>"
        FROM (
            SELECT id, title, links FROM meta.styles
        ) t
        "#
    )
    .fetch_one(&state.db.pool)
    .await?;
    Ok(Json(Styles { styles: styles.0 }))
}

async fn read_style(
    Path(id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Value>> {
    let style = sqlx::query_scalar!(
        r#"
        SELECT row_to_json(t) as "stylesheet!: sqlx::types::Json<Stylesheet>"
        FROM (
            SELECT id, value FROM meta.styles WHERE id = $1
        ) t
        "#,
        id
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(style.0.value))
}

pub(crate) fn router(_state: &State) -> Router {
    Router::new()
        .route("/styles", get(styles))
        .route("/styles/:id", get(read_style))
}
