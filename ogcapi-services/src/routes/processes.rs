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
        Link,
    },
    processes::{Execute, Process, ProcessList, ProcessQuery, ProcessSummary},
};

use crate::{extractors::RemoteUrl, AppState, Error, Result};

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
    Query(mut query): Query<ProcessQuery>,
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
        .map(|(_id, p)| p.process().summary)
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

    summaries.iter_mut().for_each(|mut p| {
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
    Path(id): Path<String>,
) -> Result<Json<Process>> {
    match state.processors.read().unwrap().get(&id) {
        Some(processor) => {
            let mut process = processor.process();

            process.summary.links = vec![Link::new(&url, SELF).mediatype(JSON)];

            Ok(Json(process))
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", id),
        )),
    }
}

async fn execution(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path(id): Path<String>,
    Json(execute): Json<Execute>,
) -> Result<Response> {
    let processors = state.processors.read().unwrap().clone();
    let processor = processors.get(&id);
    match processor {
        Some(processor) => processor.execute(execute, &state, &url).await,
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", id),
        )),
    }
}

async fn jobs() {
    todo!()
}

async fn status(
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path(id): Path<String>,
) -> Result<Response> {
    let status = state.drivers.jobs.status(&id).await?;

    match status {
        Some(mut info) => {
            info.links = vec![Link::new(url, SELF).mediatype(JSON)];

            Ok(Json(info).into_response())
        }
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", id),
        )),
    }
}

async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> Result<Response> {
    let status = state.drivers.jobs.dismiss(&id).await?;

    // TODO: cancel execution

    match status {
        Some(info) => Ok(Json(info).into_response()),
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", id),
        )),
    }
}

async fn results(State(state): State<AppState>, Path(id): Path<String>) -> Result<Response> {
    let results = state.drivers.jobs.results(&id).await?;

    // TODO: check if job is finished

    match results {
        Some(results) => Ok(Json(results).into_response()),
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{}`", id),
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

    Router::with_state(state.clone())
        .route("/processes", get(processes))
        .route("/processes/:id", get(process))
        .route("/processes/:id/execution", post(execution))
        .route("/jobs", get(jobs))
        .route("/jobs/:id", get(status).delete(delete))
        .route("/jobs/:id/results", get(results))
}
