use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use url::Position;

use ogcapi_types::{
    common::{
        link_rel::{NEXT, PREV, PROCESSES, SELF},
        media_type::JSON,
        query::LimitOffsetPagination,
        Link,
    },
    processes::{
        Execute, InlineOrRefData, JobList, Process, ProcessList, ProcessSummary, Results,
        ResultsQuery,
    },
};

use crate::{extractors::RemoteUrl, processes::ProcessResponse, AppState, Error, Result};

const CONFORMANCE: [&str; 4] = [
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss",
];

async fn processes(
    State(state): State<AppState>,
    RemoteUrl(mut url): RemoteUrl,
    Query(mut query): Query<LimitOffsetPagination>,
) -> Result<Json<ProcessList>> {
    let limit = query
        .limit
        .unwrap_or_else(|| state.processors.read().unwrap().len());
    let offset = query.offset.unwrap_or(0);

    let mut summaries: Vec<ProcessSummary> = state
        .processors
        .read()
        .unwrap()
        .clone()
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|(_id, p)| p.process().unwrap().summary)
        .collect();

    let mut links = vec![Link::new(&url, SELF).mediatype(JSON)];

    if query.limit.is_some() {
        if offset != 0 && offset >= limit {
            query.offset = Some(offset - limit);
            let query_string = serde_qs::to_string(&query)?;
            url.set_query(Some(&query_string));
            let previous = Link::new(&url, PREV).mediatype(JSON);
            links.push(previous);
        }

        if summaries.len() == limit {
            query.offset = Some(offset + limit);
            let query_string = serde_qs::to_string(&query)?;
            url.set_query(Some(&query_string));
            let next = Link::new(&url, NEXT).mediatype(JSON);
            links.push(next);
        }
    }

    summaries.iter_mut().for_each(|p| {
        p.links = vec![
            Link::new(format!("{}/{}", &url[..Position::AfterPath], p.id), SELF)
                .mediatype(JSON)
                .title("process description"),
        ];
    });

    let process_list = ProcessList {
        processes: summaries,
        links,
    };

    Ok(Json(process_list))
}

async fn process(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path(process_id): Path<String>,
) -> Result<Json<Process>> {
    match state.processors.read().unwrap().get(&process_id) {
        Some(processor) => {
            let mut process = processor.process().unwrap();

            process.summary.links = vec![Link::new(url, SELF).mediatype(JSON)];

            Ok(Json(process))
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", process_id),
        )),
    }
}

async fn execution(
    State(state): State<AppState>,
    Path(process_id): Path<String>,
    Json(execute): Json<Execute>,
) -> Result<impl IntoResponse> {
    let processors = state.processors.read().unwrap().clone();
    let processor = processors.get(&process_id);
    match processor {
        Some(processor) => match processor.execute(execute).await {
            Ok(body) => Ok(ProcessResponse(body)),
            Err(e) => Err(Error::Anyhow(anyhow::anyhow!(e))),
        },
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", process_id),
        )),
    }
}

async fn jobs(
    State(_state): State<AppState>,
    RemoteUrl(mut _url): RemoteUrl,
) -> Result<Json<JobList>> {
    todo!()
}

async fn status(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path(job_id): Path<String>,
) -> Result<Response> {
    let status = state.drivers.jobs.status(&job_id).await?;

    match status {
        Some(mut info) => {
            info.links = vec![Link::new(url, SELF).mediatype(JSON)];

            Ok(Json(info).into_response())
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", job_id),
        )),
    }
}

async fn delete(State(state): State<AppState>, Path(job_id): Path<String>) -> Result<Response> {
    let status = state.drivers.jobs.dismiss(&job_id).await?;

    // TODO: cancel execution

    match status {
        Some(info) => Ok(Json(info).into_response()),
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", job_id),
        )),
    }
}

async fn results(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<ResultsQuery>,
) -> Result<Response> {
    let results = state.drivers.jobs.results(&job_id).await?;

    // TODO: check if job is finished

    match results {
        Some(results) => {
            if let Some(outputs) = query.outputs {
                let results: HashMap<String, InlineOrRefData> = outputs
                    .iter()
                    .filter_map(|output| {
                        results
                            .get(output)
                            .map(|result| (output.to_string(), result.to_owned()))
                    })
                    .collect();

                Ok(Json(Results { results }).into_response())
            } else {
                Ok(Json(results).into_response())
            }
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", job_id),
        )),
    }
}

async fn result(
    State(state): State<AppState>,
    Path((job_id, output_id)): Path<(String, String)>,
) -> Result<Response> {
    let results = state.drivers.jobs.results(&job_id).await?;

    // TODO: check if job is finished

    match results {
        Some(results) => {
            let result = results.get(&output_id);
            Ok(Json(result).into_response())
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", job_id),
        )),
    }
}

pub(crate) fn router(state: &AppState) -> Router<AppState> {
    let mut root = state.root.write().unwrap();
    root.links.append(&mut vec![
        Link::new("processes", PROCESSES)
            .mediatype(JSON)
            .title("Metadata about the processes"),
        // Link::new("jobs", JOB_LIST)
        //     .mediatype(JSON)
        //     .title("The endpoint for job monitoring"),
    ]);

    state.conformance.write().unwrap().extend(&CONFORMANCE);

    Router::new()
        .route("/processes", get(processes))
        .route("/processes/{process_id}", get(process))
        .route("/processes/{process_id}/execution", post(execution))
        .route("/jobs", get(jobs))
        .route("/jobs/{job_id}", get(status).delete(delete))
        .route("/jobs/{job_id}/results", get(results))
        .route("/jobs/{job_id}/results/{output_id}", get(result))
}
