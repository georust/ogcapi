use std::collections::HashMap;

use ogcapi_types::{
    common::Link,
    processes::{ExecuteResult, Response, StatusCode, StatusInfo},
};
use sqlx::types::Json;

use crate::{JobHandler, ProcessResult};

use super::Db;

#[async_trait::async_trait]
impl JobHandler for Db {
    async fn register(&self, job: &StatusInfo, response_mode: Response) -> anyhow::Result<String> {
        let (id,): (String,) = sqlx::query_as(
            r#"
            INSERT INTO meta.jobs(
                job_id,
                process_id,
                status,
                created,
                updated,
                links,
                progress,
                message,
                response
            )
            VALUES (
                CASE WHEN(($1 ->> 'jobID') <> '') THEN $1 ->> 'jobID' ELSE gen_random_uuid()::text END,
                $1 ->> 'processID',
                $1 -> 'status',
                NOW(),
                NOW(),
                $1 -> 'links',
                COALESCE(($1 ->> 'progress')::smallint, 0),
                COALESCE($1 ->> 'message', ''),
                ($2 #>> '{}')::response_type
            )
            RETURNING job_id
            "#,
        )
        .bind(sqlx::types::Json(job))
        .bind(sqlx::types::Json(response_mode))
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE meta.jobs
            SET status = $1 -> 'status',
                message = $1 -> 'message',
                finished = NOW(), -- TODO: only set if status is successful or failed
                updated = NOW(),
                progress = ($1 -> 'progress')::smallint,
                links = $1 -> 'links'
            WHERE job_id = $1 ->> 'jobID'
            "#,
        )
        .bind(sqlx::types::Json(job))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn finish(
        &self,
        job_id: &str,
        status: &StatusCode,
        message: Option<String>,
        links: Vec<Link>,
        results: Option<HashMap<String, ExecuteResult>>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE meta.jobs
            SET status = $2,
                message = COALESCE($3, ''),
                links = $4,
                results = $5,
                finished = NOW(),
                updated = NOW(),
                progress = 100
            WHERE job_id = $1
            "#,
        )
        .bind(job_id)
        .bind(sqlx::types::Json(status))
        .bind(message)
        .bind(sqlx::types::Json(links))
        .bind(sqlx::types::Json(results))
        .execute(&self.pool)
        .await?;
        Ok(())
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

    async fn results(&self, id: &str) -> anyhow::Result<ProcessResult> {
        let results: Option<(Json<Option<HashMap<String, ExecuteResult>>>, Json<Response>)> = dbg!(
            sqlx::query_as(
                r#"
            SELECT results, to_jsonb(response)
            FROM meta.jobs
            WHERE job_id = $1
            "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        )?;

        Ok(match results {
            None => ProcessResult::NoSuchJob,
            Some((Json(None), _)) => ProcessResult::NotReady,
            Some((Json(Some(results)), Json(response_mode))) => ProcessResult::Results {
                results,
                response_mode,
            },
        })
    }
}
