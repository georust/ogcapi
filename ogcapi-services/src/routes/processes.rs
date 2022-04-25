use anyhow::Context;
use axum::{
    extract::{Extension, Path, Query},
    http::{header::LOCATION, HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use url::{Position, Url};
use uuid::Uuid;

use ogcapi_types::common::{
    link_rel::{JOB_LIST, NEXT, PREV, PROCESSES, SELF},
    media_type::JSON,
    Link,
};
use ogcapi_types::processes::{
    Execute, Process, ProcessList, ProcessQuery, ProcessSummary, Results, StatusInfo,
};

use crate::extractors::RemoteUrl;
use crate::{Result, State};

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

async fn processes(
    Query(mut query): Query<ProcessQuery>,
    Extension(state): Extension<State>,
) -> Result<Json<ProcessList>> {
    let mut sql = vec!["SELECT summary FROM meta.processes".to_string()];

    let mut url: Url = format!("{}/processes", &state.remote).parse().unwrap();

    let mut links = vec![Link::new(&url, SELF).mime(JSON)];

    // pagination
    if let Some(limit) = query.limit {
        sql.push("ORDER BY id".to_string());
        sql.push(format!("LIMIT {}", limit));

        let count = sqlx::query("SELECT id FROM meta.processes")
            .execute(&state.db.pool)
            .await?
            .rows_affected();

        if let Some(offset) = query.offset.or(Some(0)) {
            sql.push(format!("OFFSET {}", offset));

            if offset != 0 && offset >= limit {
                query.offset = Some(offset - limit);
                let query_string =
                    serde_qs::to_string(&query).context("failed to serialize query")?;
                url.set_query(Some(&query_string));
                let previous = Link::new(&url, PREV).mime(JSON);
                links.push(previous);
            }

            if !(offset + limit) as u64 >= count {
                query.offset = Some(offset + limit);
                let query_string =
                    serde_qs::to_string(&query).context("failed to serialize query")?;
                url.set_query(Some(&query_string));
                let next = Link::new(&url, NEXT).mime(JSON);
                links.push(next);
            }
        }
    }

    let summaries: Vec<sqlx::types::Json<ProcessSummary>> = sqlx::query_scalar(&sql.join(" "))
        .fetch_all(&state.db.pool)
        .await?;

    let process_list = ProcessList {
        processes: summaries
            .into_iter()
            .map(|mut p| {
                p.0.links =
                    vec![
                        Link::new(format!("{}/{}", p.0.id, &url[..Position::AfterPath]), SELF)
                            .mime(JSON)
                            .title("process description"),
                    ];
                p.0
            })
            .collect(),
        links,
    };

    Ok(Json(process_list))
}

async fn process(
    Path(id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Process>> {
    let mut process = sqlx::query_scalar!(
        r#"
        SELECT row_to_json(t) as "process!: sqlx::types::Json<Process>"
        FROM (
            SELECT summary, inputs, outputs FROM meta.processes WHERE id = $1
        ) t
        "#,
        &id
    )
    .fetch_one(&state.db.pool)
    .await?;

    process.summary.links =
        vec![Link::new(format!("{}/processes/{}", &state.remote, &id), SELF).mime(JSON)];

    Ok(Json(process.0))
}

async fn execution(
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(_payload): Json<Execute>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, HeaderMap, Json<StatusInfo>)> {
    let _prefer = headers.get("Prefer");

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
    .bind(sqlx::types::Json(&job.status))
    .bind(&job.created)
    .execute(&state.db.pool)
    .await?;

    // TODO: validation & execution
    let location = format!("{}/jobs/{}", &state.remote, job.job_id)
        .parse()
        .context("Unable to parse `Location` header value")?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, location);

    Ok((StatusCode::CREATED, headers, Json(job)))
}

async fn jobs() {
    todo!()
}

async fn status(
    RemoteUrl(url): RemoteUrl,
    Path(id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<StatusInfo>> {
    let mut status = sqlx::query_scalar!(
        r#"
        SELECT row_to_json(jobs) as "status_info!: sqlx::types::Json<StatusInfo>" 
        FROM meta.jobs WHERE job_id = $1
        "#,
        id
    )
    .fetch_one(&state.db.pool)
    .await?;

    status.links = vec![Link::new(url, SELF).mime(JSON)];

    Ok(Json(status.0))
}

async fn delete(Path(id): Path<String>, Extension(state): Extension<State>) -> Result<StatusCode> {
    sqlx::query("DELETE FROM meta.jobs WHERE job_id = $1")
        .bind(id)
        .execute(&state.db.pool)
        .await?;

    // TODO: cancel execution

    Ok(StatusCode::NO_CONTENT)
}

async fn results(
    Path(id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Results>> {
    let results: (sqlx::types::Json<Results>,) =
        sqlx::query_as("SELECT results FROM meta.jobs WHERE job_id = $id")
            .bind(id)
            .fetch_one(&state.db.pool)
            .await?;

    Ok(Json(results.0 .0))
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.append(&mut vec![
        Link::new(format!("{}/processes", &state.remote), PROCESSES)
            .mime(JSON)
            .title("Metadata about the processes"),
        Link::new(format!("{}/jobs", &state.remote), JOB_LIST)
            .mime(JSON)
            .title("The endpoint for job monitoring"),
    ]);

    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new()
        .route("/processes", get(processes))
        .route("/processes/:id", get(process))
        .route("/processes/:id/execution", post(execution))
        .route("/jobs", get(jobs))
        .route("/jobs/:id", get(status).delete(delete))
        .route("/jobs/:id/results", get(results))
}
