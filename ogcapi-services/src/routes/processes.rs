use anyhow::bail;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::TryFutureExt;
use hyper::HeaderMap;
use tracing::error;
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_drivers::ProcessResult;
use ogcapi_types::{
    common::{
        Exception, Link, Linked,
        link_rel::{EXECUTE, JOB_LIST, NEXT, PREV, PROCESSES, RESULTS, SELF, STATUS},
        media_type::JSON,
        query::LimitOffsetPagination,
    },
    processes::{
        Execute, JobControlOptions, JobList, Process, ProcessList, ProcessSummary, Results,
        ResultsQuery, StatusCode as JobStatusCode, StatusInfo,
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

    let mut links = vec![Link::new(url.clone(), SELF).mediatype(JSON)];

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

    summaries.iter_mut().for_each(|process| {
        let Ok(process_description_url) = url_plus_segments(url.clone(), &[&process.id])
             else {
                error!("Cannot create process description URL for process `{}`: cannot modify URL without a base", process.id);
                return;
             };
        process
            .links
            .insert_or_update(&[
                Link::new(process_description_url, SELF)
                    .mediatype(JSON)
                    .title("process description"), 
                Link::new(url_plus_segments(url.clone(), &[&process.id, "execution"]).unwrap(), EXECUTE)
                    .title("Execute endpoint")
            ]);
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
            process.summary.links.insert_or_update(&[
                Link::new(url.clone(), SELF)
                    .mediatype(JSON)
                    .title("Process description"),
                Link::new(url_plus_segments(url, &["execution"])?, EXECUTE)
                    .title("Execute endpoint"),
            ]);

            Ok(Json(process))
        }
        None => Err(Error::ApiException(
            (
                StatusCode::NOT_FOUND,
                format!("No process with id `{process_id}`"),
            )
                .into(),
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
        return Err(Error::ApiException(
            (
                StatusCode::NOT_FOUND,
                format!("No process with id `{process_id}`"),
            )
                .into(),
        ));
    };

    let process_description = processor.process()?;

    let response_mode = execute.response;
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

    let mut status_info = StatusInfo::new("");
    status_info.process_id = Some(process_id);
    status_info.job_id = state
        .drivers
        .jobs
        .register(&status_info, response_mode)
        .await?;

    {
        let mut status_info = status_info.clone();
        (state.spawn)(Box::pin(async move {
            status_info.status = JobStatusCode::Running;

            let result = state
                .drivers
                .jobs
                .update(&status_info)
                .and_then(|_| processor.execute(execute))
                .await;
            let mut results = None;

            match result {
                Ok(res) => {
                    status_info.status = JobStatusCode::Successful;
                    status_info.message = None;
                    status_info.progress = Some(100);
                    results = Some(res);
                }
                Err(e) => {
                    status_info.status = JobStatusCode::Failed;
                    status_info.message = e.to_string().into();
                }
            };

            let _ = state
                .drivers
                .jobs
                .finish(
                    &status_info.job_id,
                    status_info.status,
                    status_info.message.clone(),
                    status_info.links.clone(),
                    results,
                )
                .await;
        }));
    }

    let status_url = url_replace_segments(url, 3, &["jobs", &status_info.job_id])?;
    let status_url_string = status_url.to_string();

    status_info
        .links
        .insert_or_update(&[Link::new(status_url, STATUS)
            .title("Job status")
            .mediatype(JSON)]);

    Ok(ProcessExecuteResponse::Asynchronous {
        status_info,
        was_preferred_execution_mode: negotiated_execution_mode.was_preferred(),
        status_url: status_url_string,
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
    params(LimitOffsetPagination),
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
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    let jobs = state.drivers.jobs.status_list(offset, limit).await?;

    let mut links = vec![Link::new(url.clone(), SELF).mediatype(JSON)];

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
async fn status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    RemoteUrl(url): RemoteUrl,
) -> Result<Response> {
    let status = state.drivers.jobs.status(&job_id).await?;

    let Some(mut info) = status else {
        return Err(Error::ApiException(
            Exception::new(
                "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-job",
            )
            .status(StatusCode::NOT_FOUND.as_u16())
            .title("NoSuchJob")
            .detail(format!("No job with id `{job_id}`")),
        ));
    };

    info.links.insert_or_update(&[
        Link::new(url.clone(), SELF).mediatype(JSON),
        Link::new(url_plus_segments(url, &["results"])?, RESULTS)
            .mediatype(JSON)
            .title("Job results"),
    ]);

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
async fn delete(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    RemoteUrl(url): RemoteUrl,
) -> Result<Response> {
    let status = state.drivers.jobs.dismiss(&job_id).await?;

    // TODO: cancel execution

    let Some(mut status_info) = status else {
        return Err(Error::ApiException(
            (StatusCode::NOT_FOUND, format!("No job with id `{job_id}`")).into(),
        ));
    };

    status_info
        .links
        .insert_or_update(&[Link::new(url.clone(), SELF).mediatype(JSON)]);

    Ok(Json(status_info).into_response())
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
            Err(Error::ApiException(
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
            Err(Error::ApiException(
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
        Link::new("jobs", JOB_LIST)
            .mediatype(JSON)
            .title("The endpoint for job monitoring"),
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

/// Helper function to add a path segment to a URL, returning an error if the URL cannot be modified.
/// Example usage:
/// `url_plus_segments(Url::parse("http://example.com/foo")?, &["bar", "123"])` would return `http://example.com/foo/bar/123`.
fn url_plus_segments(mut url: Url, segments_to_add: &[&str]) -> anyhow::Result<Url> {
    let Ok(mut segments) = url.path_segments_mut() else {
        bail!("Cannot modify path segments of a URL without a base");
    };
    for segment in segments_to_add {
        segments.push(segment);
    }
    drop(segments);
    Ok(url)
}

/// Helper function to replace a path segment in a URL, returning an error if the URL cannot be modified.
/// Example usage:
/// `url_replace_segments(Url::parse("http://example.com/foo/bar")?, 1, &["baz"])` would return `http://example.com/foo/baz`.
fn url_replace_segments(
    mut url: Url,
    num_segments_to_remove: usize,
    segments_to_add: &[&str],
) -> anyhow::Result<Url> {
    let Ok(mut segments) = url.path_segments_mut() else {
        bail!("Cannot modify path segments of a URL without a base");
    };
    for _ in 0..num_segments_to_remove {
        segments.pop();
    }
    drop(segments);
    url_plus_segments(url, segments_to_add)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Drivers;
    use ogcapi_drivers::JobHandler;
    use ogcapi_processes::echo::Echo;
    use ogcapi_types::common::link_rel::EXECUTE;
    use tokio::task_local;

    /// Test that we can pass task-local context into spawned tasks.
    #[tokio::test]
    async fn it_allows_passing_scope_in_spawn() {
        task_local! {static FOO: String}

        /// A faux job handler that sends a message when update is called.
        struct FauxJobHandler {
            msg: tokio::sync::mpsc::Sender<String>,
        }
        #[async_trait::async_trait]
        impl JobHandler for FauxJobHandler {
            async fn register(
                &self,
                _job: &StatusInfo,
                _response_mode: ogcapi_types::processes::Response,
            ) -> anyhow::Result<String> {
                Ok(FOO.get().clone())
            }

            async fn update(&self, _job: &StatusInfo) -> anyhow::Result<()> {
                let foo = FOO.get().clone();
                self.msg.send(foo).await.unwrap();
                Ok(())
            }

            async fn status_list(
                &self,
                _offset: usize,
                _limit: usize,
            ) -> anyhow::Result<Vec<StatusInfo>> {
                unimplemented!()
            }

            async fn status(&self, _id: &str) -> anyhow::Result<Option<StatusInfo>> {
                unimplemented!()
            }

            async fn finish(
                &self,
                _job_id: &str,
                _status: ogcapi_types::processes::StatusCode,
                _message: Option<String>,
                _links: Vec<Link>,
                _results: Option<ogcapi_types::processes::ExecuteResults>,
            ) -> anyhow::Result<()> {
                unimplemented!()
            }

            async fn dismiss(&self, _id: &str) -> anyhow::Result<Option<StatusInfo>> {
                unimplemented!()
            }

            async fn results(&self, _id: &str) -> anyhow::Result<ProcessResult> {
                unimplemented!()
            }
        }

        crate::setup_env();

        // 1. Create drivers with our faux job handler.
        let mut drivers = Drivers::try_new_from_env().await.unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        drivers.jobs = Box::new(FauxJobHandler { msg: tx });

        let state = AppState::new(drivers)
            .await
            // 2. Add the Echo process.
            .processors(vec![Box::new(Echo)])
            // 3. Override the spawn function to pass task-local context.
            .with_spawn_fn(|fut| {
                let foo = FOO.scope(FOO.get(), fut);
                tokio::spawn(foo)
            });

        // 4. Execute a process asynchronously within a task-local scope.
        let mut headers = HeaderMap::new();
        headers.insert(
            "Prefer",
            hyper::header::HeaderValue::from_static("respond-async"),
        );
        let response = FOO
            .scope("bar".to_string(), async {
                execution(
                    State(state.clone()),
                    RemoteUrl(
                        "http://example.com/processes/echo/execution"
                            .parse()
                            .unwrap(),
                    ),
                    Path("echo".to_string()),
                    headers,
                    ValidParams(Json(
                        serde_json::from_value(serde_json::json!({
                          "inputs" : {
                            "stringInput" : "Value1",
                          },
                          "outputs" : {
                            "stringOutput" : {
                              "transmissionMode" : "value"
                            },
                          },
                          "response": "raw"
                        }))
                        .unwrap(),
                    )),
                )
                .await
            })
            .await
            .unwrap();

        // 5. Verify that the job was registered and the handler was called.
        let ProcessExecuteResponse::Asynchronous {
            status_info,
            was_preferred_execution_mode: _,
            status_url: _,
        } = response
        else {
            panic!("Expected asynchronous response");
        };

        // 6. Verify that the job ID is as expected.
        assert_eq!(status_info.job_id, "bar");

        // 7. Wait for the job handler to be called and verify the message.
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                panic!("Timeout waiting for job handler to be called");
            }
            foo = rx.recv() => {
                assert_eq!(foo.unwrap(), "bar");
            }
        }
    }

    #[tokio::test]
    async fn it_sets_the_correct_links() {
        // Minimal faux job handler to avoid DB dependencies for execution test
        struct FauxJobHandler;
        #[async_trait::async_trait]
        impl JobHandler for FauxJobHandler {
            async fn register(
                &self,
                _job: &StatusInfo,
                _response_mode: ogcapi_types::processes::Response,
            ) -> anyhow::Result<String> {
                Ok("job1".to_string())
            }

            async fn update(&self, _job: &StatusInfo) -> anyhow::Result<()> {
                Ok(())
            }

            async fn status_list(
                &self,
                _offset: usize,
                _limit: usize,
            ) -> anyhow::Result<Vec<StatusInfo>> {
                Ok(vec![])
            }

            async fn status(&self, _id: &str) -> anyhow::Result<Option<StatusInfo>> {
                Ok(Some(StatusInfo::new("job1")))
            }

            async fn finish(
                &self,
                _job_id: &str,
                _status: JobStatusCode,
                _message: Option<String>,
                _links: Vec<Link>,
                _results: Option<ogcapi_types::processes::ExecuteResults>,
            ) -> anyhow::Result<()> {
                Ok(())
            }

            async fn dismiss(&self, _id: &str) -> anyhow::Result<Option<StatusInfo>> {
                let mut info = StatusInfo::new("job1");
                info.status = JobStatusCode::Dismissed;
                Ok(Some(info))
            }

            async fn results(&self, _id: &str) -> anyhow::Result<ProcessResult> {
                Ok(ProcessResult::Results {
                    results: Default::default(),
                    response_mode: ogcapi_types::processes::Response::Document,
                })
            }
        }

        // Create drivers from env and replace jobs with our faux handler.
        crate::setup_env();
        let mut drivers = Drivers::try_new_from_env().await.unwrap();
        drivers.jobs = Box::new(FauxJobHandler);

        let state = AppState::new(drivers)
            .await
            .processors(vec![Box::new(Echo)]);

        // First: act like a relative request (RemoteUrl derived from Host/X-Forwarded-Proto)
        let base_url = Url::parse("http://example.org/subdir/").unwrap();

        // Call `processes` and assert the self link is the base URL
        let processes_response = processes(
            State(state.clone()),
            RemoteUrl(base_url.join("processes").unwrap()),
            Query(LimitOffsetPagination {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(
            processes_response.links,
            &[Link::new("http://example.org/subdir/processes", SELF).mediatype(JSON)]
        );

        assert_eq!(
            processes_response.processes[0].links,
            &[
                Link::new("http://example.org/subdir/processes/echo", SELF)
                    .mediatype(JSON)
                    .title("process description"),
                Link::new(
                    "http://example.org/subdir/processes/echo/execution",
                    EXECUTE
                )
                .title("Execute endpoint")
            ]
        );

        // Call `process` and assert the process description contains the self link
        let process_response = process(
            State(state.clone()),
            RemoteUrl(base_url.join("processes/echo").unwrap()),
            Path("echo".to_string()),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(
            process_response.summary.links,
            &[
                Link::new("http://example.org/subdir/processes/echo", SELF)
                    .mediatype(JSON)
                    .title("Process description"),
                Link::new(
                    "http://example.org/subdir/processes/echo/execution",
                    EXECUTE
                )
                .title("Execute endpoint")
            ]
        );

        // Call `jobs` and assert the self link
        let jobs_response = jobs(
            State(state.clone()),
            RemoteUrl(base_url.join("jobs?limit=10&offset=0").unwrap()),
            Query(LimitOffsetPagination {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(
            jobs_response.links,
            &[Link::new("http://example.org/subdir/jobs?limit=10&offset=0", SELF).mediatype(JSON)]
        );

        // Call `execution` (async) and assert the returned status link uses base_url
        let execution_response = execution(
            State(state.clone()),
            RemoteUrl(base_url.join("processes/echo/execution").unwrap()),
            Path("echo".to_string()),
            {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Prefer",
                    hyper::header::HeaderValue::from_static("respond-async"),
                );
                headers
            },
            ValidParams(Json(
                serde_json::from_value(serde_json::json!({
                    "inputs": {"stringInput": "Value1"},
                    "outputs": {"stringOutput": {"transmissionMode": "value"}},
                    "response": "raw"
                }))
                .unwrap(),
            )),
        )
        .await
        .unwrap();

        let ProcessExecuteResponse::Asynchronous {
            status_info,
            status_url,
            ..
        } = execution_response
        else {
            panic!("Expected asynchronous execution response");
        };

        assert_eq!(
            status_info.links,
            &[Link::new("http://example.org/subdir/jobs/job1", STATUS)
                .mediatype(JSON)
                .title("Job status")]
        );
        assert_eq!(
            status_url,
            "http://example.org/subdir/jobs/job1".to_string()
        );

        // call status route and assert
        let status_response = status(
            State(state.clone()),
            Path("job1".to_string()),
            RemoteUrl(base_url.join("jobs/job1").unwrap()),
        )
        .await
        .unwrap();
        let status_info: StatusInfo = serde_json::from_slice(
            &axum::body::to_bytes(status_response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            status_info.links,
            &[
                Link::new("http://example.org/subdir/jobs/job1", SELF).mediatype(JSON),
                Link::new("http://example.org/subdir/jobs/job1/results", RESULTS)
                    .mediatype(JSON)
                    .title("Job results")
            ]
        );

        // call delete (dismiss) route and assert
        let delete_response = delete(
            State(state.clone()),
            Path("job1".to_string()),
            RemoteUrl(base_url.join("jobs/job1").unwrap()),
        )
        .await
        .unwrap();
        let status_info: StatusInfo = serde_json::from_slice(
            &axum::body::to_bytes(delete_response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            status_info.links,
            &[Link::new("http://example.org/subdir/jobs/job1", SELF).mediatype(JSON)]
        );

        // call results and assert we get an empty results map
        let results_response = results(
            State(state.clone()),
            Path("job1".to_string()),
            Query(ResultsQuery {
                pagination: Default::default(),
                outputs: None,
            }),
        )
        .await
        .unwrap();

        assert!(results_response.results.is_empty());
    }
}
