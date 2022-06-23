use ogcapi_types::styles::{Style, Styles, Stylesheet};

use crate::StyleTransactions;

use super::Db;

#[async_trait::async_trait]
impl StyleTransactions for Db {
    async fn list_styles(&self) -> anyhow::Result<Styles> {
        let styles = sqlx::query_scalar!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t))) as "styles: sqlx::types::Json<Vec<Style>>"
            FROM (
                SELECT id, title, links FROM meta.styles
            ) t
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let styles = styles.map(|s| s.0).unwrap_or_default();
        Ok(Styles { styles })
    }

    async fn read_style(&self, id: &str) -> anyhow::Result<Option<serde_json::Value>> {
        let style = sqlx::query_scalar!(
            r#"
            SELECT row_to_json(t) as "stylesheet!: sqlx::types::Json<Stylesheet>"
            FROM (
                SELECT id, value FROM meta.styles WHERE id = $1
            ) t
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(style.map(|s| s.0.value))
    }
}
