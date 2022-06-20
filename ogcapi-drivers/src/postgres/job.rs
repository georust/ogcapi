use ogcapi_types::processes::{Results, StatusInfo};

use crate::JobHandler;

use super::Db;

#[async_trait::async_trait]
impl JobHandler for Db {
    async fn status(&self, id: &str) -> Result<StatusInfo, anyhow::Error> {
        let status = sqlx::query_scalar!(
            r#"
            SELECT row_to_json(jobs) as "status_info!: sqlx::types::Json<StatusInfo>" 
            FROM meta.jobs WHERE job_id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(status.0)
    }

    async fn delete(&self, id: &str) -> Result<(), anyhow::Error> {
        let _ = sqlx::query!("DELETE FROM meta.jobs WHERE job_id = $1", id)
            .execute(&self.pool)
            .await?;

        // TODO: cancel execution

        Ok(())
    }

    async fn results(&self, id: &str) -> Result<Results, anyhow::Error> {
        let results = sqlx::query_scalar!(
            r#"
            SELECT results as "results!: sqlx::types::Json<Results>"
            FROM meta.jobs
            WHERE job_id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(results.0)
    }
}
