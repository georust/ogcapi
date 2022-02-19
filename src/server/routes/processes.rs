use chrono::Utc;
use sqlx::types::Json;
use tide::Server;
use tide::{http::url::Position, Body, Request, Response};
use uuid::Uuid;

use crate::common::core::{Link, LinkRel, MediaType};
use crate::processes::{Execute, Process, ProcessList, ProcessSummary, Query, Results, StatusInfo};
use crate::server::State;

const CONFORMANCE: [&str; 5] = [
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss",
];

async fn processes(req: Request<State>) -> tide::Result {
    let mut url = req.url().to_owned();

    let mut query: Query = req.query()?;

    let mut sql = vec!["SELECT summary FROM meta.processes".to_string()];

    let mut links = vec![Link::new(url.as_str()).mime(MediaType::JSON)];

    // pagination
    if let Some(limit) = query.limit {
        sql.push("ORDER BY id".to_string());
        sql.push(format!("LIMIT {}", limit));

        let count = sqlx::query("SELECT id FROM meta.processes")
            .execute(&req.state().db.pool)
            .await?
            .rows_affected();

        if let Some(offset) = query.offset.or(Some(0)) {
            sql.push(format!("OFFSET {}", offset));

            if offset != 0 && offset >= limit {
                query.offset = Some(offset - limit);
                url.set_query(Some(&query.to_string()));
                let previous = Link::new(url.as_str())
                    .relation(LinkRel::Prev)
                    .mime(MediaType::JSON);
                links.push(previous);
            }

            if !(offset + limit) as u64 >= count {
                query.offset = Some(offset + limit);
                url.set_query(Some(&query.to_string()));
                let next = Link::new(url.as_str())
                    .relation(LinkRel::Next)
                    .mime(MediaType::JSON);
                links.push(next);
            }
        }
    }

    let summaries: Vec<Json<ProcessSummary>> = sqlx::query_scalar(&sql.join(" "))
        .fetch_all(&req.state().db.pool)
        .await?;

    let process_list = ProcessList {
        processes: summaries
            .into_iter()
            .map(|mut p| {
                p.0.links = Some(vec![Link::new(&format!(
                    "{}/{p.0.id}",
                    &url[..Position::AfterPath]
                ))
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

async fn process(req: Request<State>) -> tide::Result {
    let id: &str = req.param("id")?;

    let mut process: Process =
        sqlx::query_as("SELECT summary, inputs, outputs FROM meta.processes WHERE id = $id")
            .bind(id)
            .fetch_one(&req.state().db.pool)
            .await?;

    process.summary.links = Some(vec![Link::new(req.url().as_str()).mime(MediaType::JSON)]);

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&process)?);
    Ok(res)
}

async fn execution(mut req: Request<State>) -> tide::Result {
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
    .execute(&req.state().db.pool)
    .await?;

    // TODO: validation & execution

    Ok(Response::builder(201)
        .header("Location", format!("jobs/{}", job.job_id))
        .body(Body::from_json(&job)?)
        .build())
}

async fn jobs(_req: Request<State>) -> tide::Result {
    Ok(Response::builder(200).build())
}

async fn status(req: Request<State>) -> tide::Result {
    let id: &str = req.param("id")?;

    let mut status: StatusInfo = sqlx::query_as("SELECT * FROM meta.jobs WHERE job_id = $id")
        .bind(id)
        .fetch_one(&req.state().db.pool)
        .await?;

    status.links = Some(Json(vec![
        Link::new(req.url().as_str()).mime(MediaType::JSON)
    ]));

    Ok(Response::builder(200)
        .body(Body::from_json(&status)?)
        .build())
}

async fn delete(req: Request<State>) -> tide::Result {
    let id: &str = req.param("id")?;

    sqlx::query("DELETE FROM meta.jobs WHERE job_id = $1")
        .bind(id)
        .execute(&req.state().db.pool)
        .await?;

    // TODO: cancel execution

    Ok(Response::new(204))
}

async fn results(req: Request<State>) -> tide::Result {
    let id: &str = req.param("id")?;

    let results: (Json<Results>,) =
        sqlx::query_as("SELECT results FROM meta.jobs WHERE job_id = $id")
            .bind(id)
            .fetch_one(&req.state().db.pool)
            .await?;

    Ok(Response::builder(200)
        .body(Body::from_json(&results.0 .0)?)
        .build())
}

pub(crate) async fn register(app: &mut Server<State>) {
    app.state().root.write().await.links.append(&mut vec![
        Link::new("http://ogcapi.rs/processes")
            .relation(LinkRel::Processes)
            .mime(MediaType::JSON)
            .title("Metadata about the processes".to_string()),
        Link::new("http://ogcapi.rs/jobs")
            .relation(LinkRel::JobList)
            .mime(MediaType::JSON)
            .title("The endpoint for job monitoring".to_string()),
    ]);
    app.state()
        .conformance
        .write()
        .await
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    app.at("/processes").get(processes);
    app.at("/processes/:id").get(process);
    app.at("/processes/:id/execution").post(execution);
    app.at("/jobs").get(jobs);
    app.at("/jobs/:id").get(status).delete(delete);
    app.at("/jobs/:id/results").get(results);
}
