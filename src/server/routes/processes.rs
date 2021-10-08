use chrono::Utc;
use sqlx::types::Json;
use tide::{http::url::Position, Body, Request, Response};
use url::Url;
use uuid::Uuid;

use crate::common::core::{Link, MediaType, Relation};
use crate::db::Db;
use crate::processes::{Execute, Process, ProcessList, ProcessSummary, Query, Results, StatusInfo};

pub async fn list_processes(req: Request<Db>) -> tide::Result {
    let mut url = req.url().to_owned();

    let mut query: Query = req.query()?;

    let mut sql = vec!["SELECT summary FROM meta.processes".to_string()];

    let mut links = vec![Link::new(url.to_owned()).mime(MediaType::JSON)];

    // pagination
    if let Some(limit) = query.limit {
        sql.push("ORDER BY id".to_string());
        sql.push(format!("LIMIT {}", limit));

        let count = sqlx::query("SELECT id FROM meta.processes")
            .execute(&req.state().pool)
            .await?
            .rows_affected();

        if let Some(offset) = query.offset.or(Some(0)) {
            sql.push(format!("OFFSET {}", offset));

            if offset != 0 && offset >= limit {
                url.set_query(Some(&query.as_string_with_offset(offset - limit)));
                let previous = Link::new(url.to_owned())
                    .relation(Relation::Previous)
                    .mime(MediaType::JSON);
                links.push(previous);
            }

            if !(offset + limit) as u64 >= count {
                url.set_query(Some(&query.as_string_with_offset(offset + limit)));
                let next = Link::new(url.to_owned())
                    .relation(Relation::Next)
                    .mime(MediaType::JSON);
                links.push(next);
            }
        }
    }

    let summaries: Vec<Json<ProcessSummary>> = sqlx::query_scalar(&sql.join(" "))
        .fetch_all(&req.state().pool)
        .await?;

    let process_list = ProcessList {
        processes: summaries
            .into_iter()
            .map(|mut p| {
                p.0.links = Some(vec![Link::new(
                    Url::parse(&format!("{}/{}", &url[..Position::AfterPath], p.0.id)).unwrap(),
                )
                .mime(MediaType::JSON)
                .title("process description".to_string())]);
                p.0
            })
            .collect(),
        links,
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&process_list)?);
    Ok(res)
}

pub async fn retrieve_process(req: Request<Db>) -> tide::Result {
    let id: &str = req.param("id")?;

    let mut process: Process =
        sqlx::query_as("SELECT summary, inputs, outputs FROM meta.processes WHERE id = $id")
            .bind(id)
            .fetch_one(&req.state().pool)
            .await?;

    process.summary.links = Some(vec![Link::new(req.url().to_owned()).mime(MediaType::JSON)]);

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&process)?);
    Ok(res)
}

pub async fn execution(mut req: Request<Db>) -> tide::Result {
    let id = req.param("id")?.to_owned();

    let _prefer = req.header("Prefer");

    let _ececute: Execute = req.body_json().await?;

    let job = StatusInfo {
        job_id: Uuid::new_v4().to_string(),
        process_id: Some(id),
        created: Some(Utc::now()),
        ..Default::default()
    };

    sqlx::query(
        "INSERT INTO meta.jobs (job_id, process_id, status, created) VALUES ($1, $2, $3, $4)",
    )
    .bind(&job.job_id)
    .bind(&job.process_id)
    .bind(Json(&job.status))
    .bind(&job.created)
    .execute(&req.state().pool)
    .await?;

    // TODO: validation & execution

    let mut res = Response::new(201);
    res.insert_header("Location", format!("jobs/{}", job.job_id));
    res.set_body(Body::from_json(&job)?);
    Ok(res)
}

pub async fn job_status(req: Request<Db>) -> tide::Result {
    let id: &str = req.param("id")?;

    let mut status: StatusInfo = sqlx::query_as("SELECT * FROM meta.jobs WHERE job_id = $id")
        .bind(id)
        .fetch_one(&req.state().pool)
        .await?;

    status.links = Some(Json(vec![
        Link::new(req.url().to_owned()).mime(MediaType::JSON)
    ]));

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&status)?);
    Ok(res)
}

pub async fn delete_job(req: Request<Db>) -> tide::Result {
    let id: &str = req.param("id")?;

    sqlx::query!("DELETE FROM meta.jobs WHERE job_id = $1", id)
        .execute(&req.state().pool)
        .await?;

    // TODO: cancel execution

    Ok(Response::new(204))
}

pub async fn job_result(req: Request<Db>) -> tide::Result {
    let id: &str = req.param("id")?;

    let results: (Json<Results>,) =
        sqlx::query_as("SELECT results FROM meta.jobs WHERE job_id = $id")
            .bind(id)
            .fetch_one(&req.state().pool)
            .await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&results.0 .0)?);
    Ok(res)
}
