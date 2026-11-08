use chrono::Utc;
use futures::{StreamExt as _, TryStreamExt as _};
use object_store::{
    Error, ObjectStoreExt as _, PutMode, UpdateVersion,
    path::{Path, PathPart},
};
use uuid::Uuid;

use ogcapi_types::{
    common::{Exception, Link},
    processes::{ExecuteResults, Response, StatusCode, StatusInfo},
};

use crate::{JobHandler, ProcessResult};

use super::{ObjectDriver, invalid_id, segment};

/// The path of a job's `file`; `None` where the id can't be a segment of one.
fn job_path(id: &str, file: &'static str) -> Option<Path> {
    Some(Path::from_iter([
        PathPart::from("jobs"),
        segment(id)?,
        PathPart::from(file),
    ]))
}

fn no_such_job() -> anyhow::Error {
    Exception::new_from_status(404).detail("no such job").into()
}

#[async_trait::async_trait]
impl JobHandler for ObjectDriver {
    async fn register(&self, job: &StatusInfo, response_mode: Response) -> anyhow::Result<String> {
        let mut job = job.clone();
        if job.job_id.is_empty() {
            job.job_id = Uuid::new_v4().to_string();
        }
        let now = Utc::now();
        job.created = Some(now);
        job.updated = Some(now);

        // the response mode first, a job's results answered as soon as its
        // status is
        let (Some(response), Some(status)) = (
            job_path(&job.job_id, "response.json"),
            job_path(&job.job_id, "status.json"),
        ) else {
            return Err(invalid_id(&job.job_id));
        };
        let exists = || -> anyhow::Error {
            Exception::new_from_status(409)
                .detail(format!("a job `{}` exists", job.job_id))
                .into()
        };
        let mode = serde_json::to_vec(&response_mode)?.into();
        if self
            .write(&response, mode, PutMode::Create)
            .await?
            .is_none()
        {
            return Err(exists());
        }
        let payload = serde_json::to_vec(&job)?.into();
        if self
            .write(&status, payload, PutMode::Create)
            .await?
            .is_none()
        {
            return Err(exists());
        }
        Ok(job.job_id)
    }

    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()> {
        let mut job = job.clone();
        job.updated = Some(Utc::now());
        self.write_status(&job).await
    }

    async fn status_list(&self, offset: usize, limit: usize) -> anyhow::Result<Vec<StatusInfo>> {
        let listed = self
            .store
            .list_with_delimiter(Some(&Path::from("jobs")))
            .await?;
        let read: Vec<Option<StatusInfo>> = futures::stream::iter(listed.common_prefixes)
            .map(|prefix| async move { self.read::<StatusInfo>(&prefix.join("status.json")).await })
            .buffered(16)
            .try_collect()
            .await?;
        // the latest first, as the PostgreSQL driver lists them
        let mut jobs: Vec<StatusInfo> = read.into_iter().flatten().collect();
        jobs.sort_by_key(|job| std::cmp::Reverse(job.created));
        Ok(jobs.into_iter().skip(offset).take(limit).collect())
    }

    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let Some(status) = job_path(id, "status.json") else {
            return Ok(None);
        };
        self.read(&status).await
    }

    async fn finish(
        &self,
        job_id: &str,
        status: &StatusCode,
        message: Option<String>,
        links: Vec<Link>,
        results: Option<ExecuteResults>,
    ) -> anyhow::Result<()> {
        let mut job = self.status(job_id).await?.ok_or_else(no_such_job)?;

        if let Some(results) = results {
            let location = job_path(job_id, "results.json").ok_or_else(no_such_job)?;
            self.store
                .put(&location, serde_json::to_vec(&results)?.into())
                .await?;
        }

        let now = Utc::now();
        job.status = status.clone();
        job.message = Some(message.unwrap_or_default());
        job.links = links;
        job.finished = Some(now);
        job.updated = Some(now);
        job.progress = Some(100);
        self.write_status(&job).await
    }

    async fn dismiss(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let Some(location) = job_path(id, "status.json") else {
            return Ok(None);
        };
        // a job accepted or running only, as the PostgreSQL driver dismisses,
        // its status written against the one read
        let _writes = self.writes.lock().await;
        for _ in 0..8 {
            let read = match self.store.get(&location).await {
                Ok(read) => read,
                Err(Error::NotFound { .. }) => return Ok(None),
                Err(error) => return Err(error.into()),
            };
            let version = UpdateVersion {
                e_tag: read.meta.e_tag.clone(),
                version: read.meta.version.clone(),
            };
            let mut job: StatusInfo = serde_json::from_slice(&read.bytes().await?)?;
            if !matches!(job.status, StatusCode::Accepted | StatusCode::Running) {
                return Ok(None);
            }
            job.status = StatusCode::Dismissed;
            job.message = Some("Job dismissed".to_string());
            let payload = serde_json::to_vec(&job)?.into();
            if self
                .write(&location, payload, PutMode::Update(version))
                .await?
                .is_some()
            {
                return Ok(Some(job));
            }
        }
        Err(Exception::new_from_status(503)
            .detail("the job changed under every dismissal")
            .into())
    }

    async fn results(&self, id: &str) -> anyhow::Result<ProcessResult> {
        let (Some(response), Some(results)) =
            (job_path(id, "response.json"), job_path(id, "results.json"))
        else {
            return Ok(ProcessResult::NoSuchJob);
        };
        let Some(response_mode) = self.read::<Response>(&response).await? else {
            return Ok(ProcessResult::NoSuchJob);
        };
        Ok(match self.read::<ExecuteResults>(&results).await? {
            None => ProcessResult::NotReady,
            Some(results) => ProcessResult::Results {
                results,
                response_mode,
            },
        })
    }
}

impl ObjectDriver {
    /// Write the status of a job registered, a refusal of one that isn't.
    async fn write_status(&self, job: &StatusInfo) -> anyhow::Result<()> {
        let status = job_path(&job.job_id, "status.json").ok_or_else(no_such_job)?;
        let _writes = self.writes.lock().await;
        if !self.exists(&status).await? {
            return Err(no_such_job());
        }
        self.store
            .put(&status, serde_json::to_vec(job)?.into())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ogcapi_types::processes::{Response, StatusCode, StatusInfo};

    use crate::{JobHandler, ProcessResult};

    use super::ObjectDriver;

    #[tokio::test]
    async fn a_job_is_registered_finished_and_answers_its_results() {
        let driver = ObjectDriver::default();

        let id = driver
            .register(&StatusInfo::new(""), Response::Raw)
            .await
            .unwrap();
        assert_ne!(id, "");
        assert!(matches!(
            driver.results(&id).await.unwrap(),
            ProcessResult::NotReady
        ));

        let mut job = driver.status(&id).await.unwrap().unwrap();
        job.status = StatusCode::Running;
        driver.update(&job).await.unwrap();
        assert_eq!(
            driver.status(&id).await.unwrap().unwrap().status,
            StatusCode::Running
        );

        driver
            .finish(
                &id,
                &StatusCode::Successful,
                None,
                vec![],
                Some(HashMap::new()),
            )
            .await
            .unwrap();
        let job = driver.status(&id).await.unwrap().unwrap();
        assert_eq!(job.status, StatusCode::Successful);
        assert_eq!(job.progress, Some(100));
        assert!(job.finished.is_some());
        assert!(matches!(
            driver.results(&id).await.unwrap(),
            ProcessResult::Results {
                response_mode: Response::Raw,
                ..
            }
        ));

        // a job finished is no longer dismissed
        assert!(driver.dismiss(&id).await.unwrap().is_none());
        assert!(matches!(
            driver.results("none").await.unwrap(),
            ProcessResult::NoSuchJob
        ));
    }

    #[tokio::test]
    async fn jobs_are_listed_the_latest_first() {
        let driver = ObjectDriver::default();
        for id in ["a", "b", "c"] {
            driver
                .register(&StatusInfo::new(id), Response::Document)
                .await
                .unwrap();
        }
        let dismissed = driver.dismiss("b").await.unwrap().unwrap();
        assert_eq!(dismissed.status, StatusCode::Dismissed);

        let jobs = driver.status_list(1, 1).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(driver.status_list(0, 10).await.unwrap().len(), 3);
    }
}
