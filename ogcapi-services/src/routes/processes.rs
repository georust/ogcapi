use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::TryFutureExt;
use hyper::HeaderMap;
use ogcapi_drivers::ProcessResult;
use tokio::spawn;
use tracing::error;
use url::Position;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Exception, Link,
        link_rel::{NEXT, PREV, PROCESSES, RESULTS, SELF, STATUS},
        media_type::JSON,
        query::LimitOffsetPagination,
    },
    processes::{
        Execute, JobControlOptions, JobList, Process, ProcessList, ProcessSummary, Results,
        ResultsQuery, StatusInfo,
    },
};

use crate::{
    AppState, Error, Result,
    extractors::RemoteUrl,
    processes::{ProcessExecuteResponse, ProcessResultsResponse, ValidParams},
};

const CONFORMANCE: [&str; 5] = [
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30",
    "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback",
    // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss",
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
    let processors = read_lock(&state.processors);
    let limit = query.limit.unwrap_or_else(|| processors.len());
    let offset = query.offset.unwrap_or(0);

    let mut summaries: Vec<ProcessSummary> = processors
        .iter()
        .skip(offset)
        .take(limit)
        .filter_map(|(_id, p)| {
            p.process()
                .map(|p| p.summary)
                .inspect_err(|e| error!("Error when accessing process: {e}"))
                .ok()
        })
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
    match read_lock(&state.processors)
        .get(&process_id)
        .and_then(|processor| processor.process().ok())
    {
        Some(mut process) => {
            let self_link = Link::new(url.clone(), SELF).mediatype(JSON);
            if let Some(link) = process.summary.links.iter_mut().find(|l| l.rel == SELF) {
                *link = Link::new(url.clone(), SELF).mediatype(JSON);
            } else {
                process.summary.links.insert(0, self_link);
            }

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
    RemoteUrl(url): RemoteUrl,
    Path(process_id): Path<String>,
    headers: HeaderMap,
    ValidParams(Json(execute)): ValidParams<Json<Execute>>,
) -> Result<ProcessExecuteResponse> {
    let Some(processor) = read_lock(&state.processors).get(&process_id).cloned() else {
        return Err(Error::Exception(
            StatusCode::NOT_FOUND,
            format!("No process with id `{process_id}`"),
        ));
    };

    let process_description = processor.process()?;

    let response_mode = execute.response.clone();
    let negotiated_execution_mode =
        negotiate_execution_mode(&headers, &process_description.summary.job_control_options);

    if negotiated_execution_mode.is_sync() {
        let results = processor.execute(execute).await?;
        return Ok(ProcessExecuteResponse::Synchronous {
            results: ProcessResultsResponse {
                results,
                response_mode,
            },
            was_preferred_execution_mode: negotiated_execution_mode.was_preferred(),
        });
    }

    let base_url = url[..url::Position::BeforePath].to_string();

    let mut status_info = StatusInfo {
        process_id: Some(process_id),
        status: ogcapi_types::processes::StatusCode::Accepted,
        ..Default::default()
    };

    let job_id = state
        .drivers
        .jobs
        .register(&status_info, response_mode)
        .await?;

    status_info.job_id = job_id;
    status_info.links.push(
        Link::new(format!("{base_url}/jobs/{}", status_info.job_id), STATUS)
            .title("Job status")
            .mediatype(JSON),
    );

    {
        let mut status_info = status_info.clone();
        spawn(async move {
            status_info.status = ogcapi_types::processes::StatusCode::Running;

            let result = state
                .drivers
                .jobs
                .update(&status_info)
                .and_then(|_| processor.execute(execute))
                .await;
            let mut results = None;

            match result {
                Ok(res) => {
                    status_info.status = ogcapi_types::processes::StatusCode::Successful;
                    status_info.message = None;
                    status_info.progress = Some(100);

                    if let Ok(results_link) =
                        url.join(&format!("/jobs/{}/results", status_info.job_id))
                    {
                        status_info
                            .links
                            .push(Link::new(results_link, RESULTS).title("Job result"));
                    }

                    results = Some(res);
                }
                Err(e) => {
                    status_info.status = ogcapi_types::processes::StatusCode::Failed;
                    status_info.message = e.to_string().into();
                }
            };

            let _ = state
                .drivers
                .jobs
                .finish(
                    &status_info.job_id,
                    &status_info.status,
                    status_info.message.clone(),
                    status_info.links.clone(),
                    results,
                )
                .await;
        });
    }

    Ok(ProcessExecuteResponse::Asynchronous {
        status_info,
        was_preferred_execution_mode: negotiated_execution_mode.was_preferred(),
        base_url,
    })
}

/// Determine whether the client prefers synchronous execution
/// by inspecting the "Prefer" header.
fn client_execute_preference(headers: &HeaderMap) -> ClientExecutionModePreference {
    let prefer = headers
        .get("Prefer")
        .and_then(|s| s.to_str().ok())
        .unwrap_or_default();

    if prefer.contains("respond-sync") {
        ClientExecutionModePreference::Sync
    } else if prefer.contains("respond-async") {
        ClientExecutionModePreference::Async
    } else {
        ClientExecutionModePreference::None
    }
}

enum ClientExecutionModePreference {
    Sync,
    Async,
    None,
}

enum NegotiatedExecutionMode {
    Sync { was_preferred: bool },
    Async { was_preferred: bool },
}

impl NegotiatedExecutionMode {
    fn is_sync(&self) -> bool {
        matches!(self, NegotiatedExecutionMode::Sync { .. })
    }

    fn was_preferred(&self) -> bool {
        match self {
            NegotiatedExecutionMode::Sync { was_preferred } => *was_preferred,
            NegotiatedExecutionMode::Async { was_preferred } => *was_preferred,
        }
    }
}

/// Determine whether the execution should be synchronous or asynchronous.
///
/// Requirements:
/// - `req/core/process-execute-default-execution-mode`: If the execute request is not accompanied with a preference:
///     - If the process supports only synchronous execution, execute synchronously.
///     - If the process supports only asynchronous execution, execute asynchronously.
///     - If the process supports both synchronous and asynchronous execution, execute synchronously.
/// - `/req/core/process-execute-auto-execution-mode`: If the execute request is accompanied with the preference `respond-async`:
///     - If the process supports only asynchronous execution, execute asynchronously.
///     - If the process supports only synchronous execution, execute synchronously.
///     - If the process supports both synchronous and asynchronous execution, execute asynchronously (or synchronously).
/// - `/rec/core/process-execute-preference-applied`: If the execute request is executed as preferred by the client, indicate this in the response (`Preference-Applied`).
///
fn negotiate_execution_mode(
    headers: &HeaderMap,
    job_control_options: &[JobControlOptions],
) -> NegotiatedExecutionMode {
    let client_preference = client_execute_preference(headers);
    let (can_be_executed_sync, can_be_executed_async) =
        job_control_options
            .iter()
            .fold((false, false), |(sync, async_), option| match option {
                JobControlOptions::SyncExecute => (true, async_),
                JobControlOptions::AsyncExecute => (sync, true),
                _ => (sync, async_),
            });
    match client_preference {
        ClientExecutionModePreference::Sync if can_be_executed_sync => {
            NegotiatedExecutionMode::Sync {
                was_preferred: true,
            }
        }
        ClientExecutionModePreference::Async if can_be_executed_async => {
            NegotiatedExecutionMode::Async {
                was_preferred: true,
            }
        }
        _ if can_be_executed_sync => NegotiatedExecutionMode::Sync {
            was_preferred: false,
        },
        _ => NegotiatedExecutionMode::Async {
            was_preferred: false,
        },
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
    State(state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Query(query): Query<LimitOffsetPagination>,
) -> Result<Json<JobList>> {
    const DEFAULT_LIMIT: usize = 10;
    const MAX_LIMIT: usize = 100;

    let offset = query.offset.unwrap_or_default();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).max(MAX_LIMIT);

    let jobs = state.drivers.jobs.status_list(offset, limit).await?;

    let mut links = vec![Link::new(&url, SELF).mediatype(JSON)];

    if jobs.len() >= limit {
        let mut next_url = url.clone();
        let next_offset = offset + limit;
        next_url.set_query(Some(&format!("limit={}&offset={}", limit, next_offset)));
        links.push(Link::new(&next_url, NEXT).mediatype(JSON));
    }

    Ok(Json(JobList { jobs, links }))
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
async fn status(State(state): State<AppState>, Path(job_id): Path<String>) -> Result<Response> {
    let status = state.drivers.jobs.status(&job_id).await?;

    let Some(info) = status else {
        return Err(Error::OgcApiException(
            Exception::new(
                "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-job",
            )
            .status(StatusCode::NOT_FOUND.as_u16())
            .title("NoSuchJob")
            .detail(format!("No job with id `{job_id}`")),
        ));
    };

    Ok(Json(info).into_response())
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
/// For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).
///
// On success, cf. `/req/core/job-results`
// On failure, cf. `/req/core/job-results-failed`.
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
    Query(_query): Query<ResultsQuery>,
) -> Result<ProcessResultsResponse> {
    let results = state.drivers.jobs.results(&job_id).await?;

    // TODO: use pagination, etc. from `_query`

    match results {
        ProcessResult::NoSuchJob => {
            // `/req/core/job-results-exception/no-such-job`
            Err(Error::OgcApiException(
                Exception::new(
                    "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-job",
                )
                .status(404)
                .title("NoSuchJob")
                .detail(format!("No job with id `{job_id}`")),
            ))
        }
        ProcessResult::NotReady => {
            // `/req/core/job-results-exception/results-not-ready`
            Err(Error::OgcApiException(
                Exception::new(
                    "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/result-not-ready",
                )
                .status(404)
                .title("NotReady")
                .detail(format!("Results for job `{job_id}` are not ready yet")),
            ))
        }
        ProcessResult::Results {
            results,
            response_mode,
        } => Ok(ProcessResultsResponse {
            results,
            response_mode,
        }),
    }
}

/// Helper function to read-lock a RwLock, recovering from poisoning if necessary.
fn read_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match mutex.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to write-lock a RwLock, recovering from poisoning if necessary.
fn write_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match mutex.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    let mut root = write_lock(&state.root);
    root.links.append(&mut vec![
        Link::new("processes", PROCESSES)
            .mediatype(JSON)
            .title("Metadata about the processes"),
        // Link::new("jobs", JOB_LIST)
        //     .mediatype(JSON)
        //     .title("The endpoint for job monitoring"),
    ]);

    write_lock(&state.conformance).extend(&CONFORMANCE);

    OpenApiRouter::new()
        .routes(routes!(processes))
        .routes(routes!(process))
        .routes(routes!(execution))
        .routes(routes!(jobs))
        .routes(routes!(status, delete))
        .routes(routes!(results))
}
