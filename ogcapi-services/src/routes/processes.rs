use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::OnceCell;
use url::Position;

use ogcapi_types::{
    common::{
        link_rel::{JOB_LIST, NEXT, PREV, PROCESSES, SELF},
        media_type::JSON,
        Link,
    },
    processes::{
        Execute as ProcessExecute, Process, ProcessList, ProcessQuery, ProcessSummary, Results,
        StatusInfo,
    },
};

use crate::{extractors::RemoteUrl, Error, Processor, Result, State};

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

static PROCESSORS: OnceCell<BTreeMap<String, Box<dyn Processor>>> = OnceCell::new();

async fn processes(
    Query(mut query): Query<ProcessQuery>,
    RemoteUrl(mut url): RemoteUrl,
) -> Result<Json<ProcessList>> {
    let processors = PROCESSORS.get().unwrap();

    let limit = query.limit.unwrap_or(processors.len());
    let offset = query.offset.unwrap_or(0);

    let mut summaries: Vec<ProcessSummary> = processors
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

async fn process(Path(id): Path<String>, RemoteUrl(url): RemoteUrl) -> Result<Json<Process>> {
    let processors = PROCESSORS.get().unwrap();

    if !processors.contains_key(&id) {
        return Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", id),
        ));
    }

    let mut process = processors.get(&id).unwrap().process();

    process.summary.links = vec![Link::new(&url, SELF).mediatype(JSON)];

    Ok(Json(process))
}

async fn execution(
    Path(id): Path<String>,
    Json(execute): Json<ProcessExecute>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Response> {
    let processors = PROCESSORS.get().unwrap();

    if !processors.contains_key(&id) {
        return Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{}`", id),
        ));
    }

    // TODO: validation & async execution
    processors.get(&id).unwrap().execute(&execute, &state).await
}

async fn jobs() {
    todo!()
}

async fn status(
    RemoteUrl(url): RemoteUrl,
    Path(id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<StatusInfo>> {
    let mut status = state.drivers.jobs.status(&id).await?;

    status.links = vec![Link::new(url, SELF).mediatype(JSON)];

    Ok(Json(status))
}

async fn delete(
    Path(id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<StatusCode> {
    state.drivers.jobs.delete(&id).await?;

    // TODO: cancel execution

    Ok(StatusCode::NO_CONTENT)
}

async fn results(
    Path(id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Results>> {
    let results = state.drivers.jobs.results(&id).await?;

    Ok(Json(results))
}

pub(crate) fn router(state: &State, processors: Vec<Box<dyn Processor>>) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.append(&mut vec![
        Link::new(format!("{}/processes", &state.remote), PROCESSES)
            .mediatype(JSON)
            .title("Metadata about the processes"),
        Link::new(format!("{}/jobs", &state.remote), JOB_LIST)
            .mediatype(JSON)
            .title("The endpoint for job monitoring"),
    ]);

    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    // Register processors
    let mut processor_map: BTreeMap<String, Box<dyn Processor>> = BTreeMap::new();
    for p in processors {
        processor_map.insert(p.id(), p);
    }
    let _ = PROCESSORS.set(processor_map);

    Router::new()
        .route("/processes", get(processes))
        .route("/processes/:id", get(process))
        .route("/processes/:id/execution", post(execution))
        .route("/jobs", get(jobs))
        .route("/jobs/:id", get(status).delete(delete))
        .route("/jobs/:id/results", get(results))
}
