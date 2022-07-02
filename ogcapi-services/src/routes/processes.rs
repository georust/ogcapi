use std::sync::Arc;

use axum::{
    extract::{Extension, Multipart, Path, Query},
    http::StatusCode,
    response::Response,
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
    processes::{Execute as ProcessExecute, Process, ProcessList, ProcessQuery, ProcessSummary},
};

use crate::{extractors::RemoteUrl, Error, Result, State};

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
    Query(mut query): Query<ProcessQuery>,
    RemoteUrl(mut url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ProcessList>> {
    let limit = query.limit.unwrap_or(state.processors.len());
    let offset = query.offset.unwrap_or(0);

    let mut summaries: Vec<ProcessSummary> = state
        .processors
        .values()
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|p| p.process().summary)
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
    Path(id): Path<String>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Process>> {
    match state.processors.get(&id) {
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
    Path(id): Path<String>,
    json: Option<Json<ProcessExecute>>,
    multipart: Option<Multipart>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<Arc<State>>,
) -> Result<Response> {
    let mut execute: Option<ProcessExecute> = None;

    if let Some(Json(e)) = json {
        execute = Some(e);
    }

    if let Some(mut multipart) = multipart {
        while let Some(field) = multipart.next_field().await.unwrap() {
            tracing::debug!("{:#?}", field);
            let data = field.bytes().await.expect("Get field data as bytes");
            if let Ok(e) = serde_json::from_slice(&data) {
                execute = Some(e);
                break;
            }
        }
    }

    if execute.is_none() {
        return Err(Error::Exception(
            StatusCode::BAD_REQUEST,
            "Unable to extract `ProcessExecute` from body".to_string(),
        ));
    }

    match state.processors.get(&id) {
        Some(processor) => processor.execute(execute.unwrap(), &state, &url).await,
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", id),
        )),
    }
}

// async fn jobs() {
//     todo!()
// }

// async fn status(
//     RemoteUrl(url): RemoteUrl,
//     Path(id): Path<String>,
//     Extension(state): Extension<Arc<State>>,
// ) -> Result<Json<StatusInfo>> {
//     let mut status = state.drivers.jobs.status(&id).await?;

//     status.links = vec![Link::new(url, SELF).mediatype(JSON)];

//     Ok(Json(status))
// }

// async fn delete(
//     Path(id): Path<String>,
//     Extension(state): Extension<Arc<State>>,
// ) -> Result<StatusCode> {
//     state.drivers.jobs.delete(&id).await?;

//     // TODO: cancel execution

//     Ok(StatusCode::NO_CONTENT)
// }

// async fn results(
//     Path(id): Path<String>,
//     Extension(state): Extension<Arc<State>>,
// ) -> Result<Json<Results>> {
//     let results = state.drivers.jobs.results(&id).await?;

//     Ok(Json(results))
// }

pub(crate) fn router(state: &State) -> Router {
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
        .route("/processes/:id", get(process))
        .route("/processes/:id/execution", post(execution))
    // .route("/jobs", get(jobs))
    // .route("/jobs/:id", get(status).delete(delete))
    // .route("/jobs/:id/results", get(results))
}
