use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use url::Position;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Exception, Link,
        link_rel::{NEXT, PREV, PROCESSES, SELF},
        media_type::JSON,
        query::LimitOffsetPagination,
    },
    processes::{
        Execute, InlineOrRefData, JobList, Process, ProcessList, ProcessSummary, Results,
        ResultsQuery, StatusInfo,
    },
};

use crate::{AppState, Error, Result, extractors::RemoteUrl, processes::ProcessResponse};

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

/// Retrieve the list of available processes
///
/// The list of processes contains a summary of each process the OGC API - Processes
/// offers, including the link to a more detailed description of the process.
///
/// For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
#[utoipa::path(get, path = "/processes", tag = "Processes",
    responses(
        (
            status = 200,
            description = "Information about the available processe",
            body = ProcessList
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
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

/// Retrieve a processes description
///
/// The process description contains information about inputs and outputs and
/// a link to the execution-endpoint for the process. The Core does not
/// mandate the use of a specific process description to specify the interface
/// of a process. That said, the Core requirements class makes the following recommendation:
///
/// Implementations SHOULD consider supporting the OGC process description.
///
/// For more information, see Section 7.10.
#[utoipa::path(get, path = "/processes/{processID}", tag = "Processes",
    responses(
        (
            status = 200,
            description = "A process description",
            body = Process
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
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
            format!("No process with id `{process_id}`"),
        )),
    }
}

/// Execute a process
///
/// Create a new job.
///
/// For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).
#[utoipa::path(post, path = "/processes/{processID}/execution", tag = "Processes",
    request_body = Execute,
    responses(
        (
            status = 200,
            description = "Result of synchronous execution",
            body = Results
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
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
            format!("No process with id `{process_id}`"),
        )),
    }
}

/// Retrieve the list of jobs
///
/// For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
#[utoipa::path(get, path = "/jobs", tag = "Processes",
    responses(
        (
            status = 200,
            description = "A list of jobs for this process.",
            body = JobList
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
async fn jobs(
    State(_state): State<AppState>,
    RemoteUrl(mut _url): RemoteUrl,
) -> Result<Json<JobList>> {
    todo!()
}

/// Retrieve the status of a job
///
/// Shows the status of a job.
///
/// For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
#[utoipa::path(get, path = "/jobs/{jobId}", tag = "Processes",
    responses(
        (
            status = 200,
            description = "The status of a job",
            body = StatusInfo
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
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
            format!("No job with id `{job_id}`"),
        )),
    }
}

/// Cancel a job execution, remove finished job
///
/// Cancel a job execution and remove it from the jobs list.
///
/// For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
#[utoipa::path(delete, path = "/jobs/{jobId}", tag = "Processes",
    responses(
        (
            status = 200,
            description = "The status of a job",
            body = StatusInfo
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
async fn delete(State(state): State<AppState>, Path(job_id): Path<String>) -> Result<Response> {
    let status = state.drivers.jobs.dismiss(&job_id).await?;

    // TODO: cancel execution

    match status {
        Some(info) => Ok(Json(info).into_response()),
        None => Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No job with id `{job_id}`"),
        )),
    }
}

/// Retrieve the result(s) of a job
///
/// Lists available results of a job. In case of a failure, lists exceptions instead.
///
/// For more information, see [Section 7.13](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_job_results).
#[utoipa::path(get, path = "/jobs/{jobId}/results", tag = "Processes",
    responses(
        (
            status = 200,
            description = "The results of a job",
            body = Results
        ),
        (
            status = 404, description = "The requested URI was not found.", 
            body = Exception, example = json!(Exception::new_from_status(404))
        )
    )
)]
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
            format!("No job with id `{job_id}`"),
        )),
    }
}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
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

    OpenApiRouter::new()
        .routes(routes!(processes))
        .routes(routes!(process))
        .routes(routes!(execution))
        .routes(routes!(jobs))
        .routes(routes!(status, delete))
        .routes(routes!(results))
}
