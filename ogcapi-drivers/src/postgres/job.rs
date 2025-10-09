use ogcapi_types::processes::{Results, StatusCode, StatusInfo};

use crate::JobHandler;

use super::Db;

#[async_trait::async_trait]
impl JobHandler for Db {
    async fn register(&self, job: &StatusInfo) -> anyhow::Result<String> {
        let (id,): (String,) = sqlx::query_as(
            r#"
            INSERT INTO meta.jobs(
                job_id, process_id, status, created, updated, links
            )
            VALUES (
                $1 ->> 'jobID', $1 ->> 'processID', $1 -> 'status', NOW(), NOW(), $1 -> 'links'
            )
            RETURNING job_id
            "#,
        )
        .bind(sqlx::types::Json(job))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn status_list(&self, offset: usize, limit: usize) -> anyhow::Result<Vec<StatusInfo>> {
        let status_list: Vec<sqlx::types::Json<StatusInfo>> = sqlx::query_scalar(
            r#"
            SELECT row_to_json(jobs) as "status_info!" 
            FROM meta.jobs
            ORDER BY created DESC
            OFFSET $1
            LIMIT $2
            "#,
        )
        .bind(offset as i64)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(status_list.into_iter().map(|s| s.0).collect())
    }

    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let status: Option<sqlx::types::Json<StatusInfo>> = sqlx::query_scalar(
            r#"
            SELECT row_to_json(jobs) as "status_info!" 
            FROM meta.jobs WHERE job_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(status.map(|s| s.0))
    }

    async fn dismiss(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let status: Option<sqlx::types::Json<StatusInfo>> = sqlx::query_scalar(
            r#"
            UPDATE meta.jobs
            SET status = $2,
                message = 'Job dismissed'
            WHERE job_id = $1 AND status <@ '["accepted", "running"]'::jsonb
            RETURNING row_to_json(jobs) as "status_info!"
            "#,
        )
        .bind(id)
        .bind(sqlx::types::Json(StatusCode::Dismissed))
        .fetch_optional(&self.pool)
        .await?;

        Ok(status.map(|s| s.0))
    }

    async fn results(&self, id: &str) -> anyhow::Result<Option<Results>> {
        let results: Option<sqlx::types::Json<Results>> = sqlx::query_scalar(
            r#"
            SELECT results as "results!"
            FROM meta.jobs
            WHERE job_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(results.map(|r| r.0))
    }
}
