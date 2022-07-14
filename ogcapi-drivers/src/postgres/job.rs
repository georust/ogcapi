use ogcapi_types::processes::{Results, StatusInfo};

use crate::JobHandler;

use super::Db;

#[async_trait::async_trait]
impl JobHandler for Db {
    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let status = sqlx::query_scalar!(
            r#"
            SELECT row_to_json(jobs) as "status_info!: sqlx::types::Json<StatusInfo>" 
            FROM meta.jobs WHERE job_id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(status.map(|s| s.0))
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let _ = sqlx::query!("DELETE FROM meta.jobs WHERE job_id = $1", id)
            .execute(&self.pool)
            .await?;

        // TODO: cancel execution

        Ok(())
    }

    async fn results(&self, id: &str) -> anyhow::Result<Option<Results>> {
        let results = sqlx::query_scalar!(
            r#"
            SELECT results as "results!: sqlx::types::Json<Results>"
            FROM meta.jobs
            WHERE job_id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(results.map(|r| r.0))
    }
}
