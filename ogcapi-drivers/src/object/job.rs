use chrono::Utc;
use object_store::{Error, ObjectStoreExt, PutMode, PutOptions, path::Path};
use uuid::Uuid;

use ogcapi_types::{
    common::Link,
    processes::{ExecuteResults, Response, StatusCode, StatusInfo},
};

use crate::{JobHandler, ProcessResult};

use super::ObjectDriver;

#[async_trait::async_trait]
impl JobHandler for ObjectDriver {
    async fn register(&self, job: &StatusInfo, _response_mode: Response) -> anyhow::Result<String> {
        // set random id if missing
        let mut job_mut: StatusInfo;
        let job = if job.job_id.is_empty() {
            job_mut = job.to_owned();
            job_mut.job_id = Uuid::new_v4().to_string();
            &job_mut
        } else {
            job
        };
        let job_id = job.job_id.to_owned();

        let location = Path::from(format!("jobs/{job_id}/status_info.json"));

        let payload = serde_json::to_vec(job)?;

        let options = PutOptions {
            mode: PutMode::Create,
            ..Default::default()
        };
        self.store
            .put_opts(&location, payload.into(), options)
            .await?;

        Ok(job_id)
    }

    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()> {
        let job_id = &job.job_id;
        let location = Path::from(format!("jobs/{job_id}/status_info.json"));

        let payload = serde_json::to_vec(job)?;

        self.store.put(&location, payload.into()).await?;

        Ok(())
    }

    async fn finish(
        &self,
        job_id: &str,
        status_code: StatusCode,
        message: Option<String>,
        links: Vec<Link>,
        execute_results: Option<ExecuteResults>,
    ) -> anyhow::Result<()> {
        let Some(mut status_info) = self.status(job_id).await? else {
            anyhow::bail!("no such job registered")
        };

        // store execute results
        // TODO: store each one individual
        if let Some(execute_results) = execute_results.as_ref() {
            let location = Path::from(format!("jobs/{job_id}/execute_results.json"));

            let payload = serde_json::to_vec(execute_results)?;

            let options = PutOptions {
                mode: PutMode::Create,
                ..Default::default()
            };
            self.store
                .put_opts(&location, payload.into(), options)
                .await?;
        }

        // update status
        status_info.status = status_code;
        status_info.message = message;
        status_info.links.extend(links);
        status_info.finished = Some(Utc::now());
        status_info.progress = Some(100);

        self.update(&status_info).await?;

        Ok(())
    }

    async fn status_list(&self, _offset: usize, _limit: usize) -> anyhow::Result<Vec<StatusInfo>> {
        let prefix = Path::from("jobs");

        let list_result = self.store.list_with_delimiter(Some(&prefix)).await?;

        let mut jobs = Vec::with_capacity(list_result.common_prefixes.len());

        for path in list_result.common_prefixes {
            let last = path.parts().next_back().unwrap();
            let job_id = last.as_ref().trim_end_matches(".json");
            let status_info = self.status(job_id).await?;
            jobs.push(status_info.unwrap());
        }

        Ok(jobs)
    }

    async fn status(&self, job_id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let location = Path::from(format!("jobs/{job_id}/status_info.json"));

        match self.store.get(&location).await {
            Ok(r) => Ok(Some(serde_json::from_slice(&r.bytes().await?)?)),
            Err(e) => match e {
                Error::NotFound { path: _, source: _ } => return Ok(None),
                _ => return Err(anyhow::Error::new(e)),
            },
        }
    }

    async fn dismiss(&self, job_id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let Some(mut status_info) = self.status(job_id).await? else {
            return Ok(None);
        };

        status_info.status = StatusCode::Dismissed;

        let location = Path::from(format!("jobs/{job_id}/status_info.json"));

        let payload = serde_json::to_vec(&status_info)?;

        self.store.put(&location, payload.into()).await?;

        Ok(Some(status_info))
    }

    async fn results(&self, job_id: &str) -> anyhow::Result<ProcessResult> {
        let Some(job_info) = self.status(job_id).await? else {
            return Ok(ProcessResult::NoSuchJob);
        };

        match job_info.status {
            StatusCode::Accepted | StatusCode::Running => Ok(ProcessResult::NotReady),
            StatusCode::Successful => {
                let location = Path::from(format!("jobs/{job_id}/execute_results.json"));
                match self.store.get(&location).await {
                    Ok(result) => Ok(ProcessResult::Results {
                        results: serde_json::from_slice(&result.bytes().await?)?,
                        response_mode: Response::Document,
                    }),
                    Err(e) => match e {
                        Error::NotFound { path: _, source: _ } => Ok(ProcessResult::Results {
                            results: Default::default(),
                            response_mode: Response::Document,
                        }),
                        e => Err(e.into()),
                    },
                }
            }
            StatusCode::Failed => todo!(),
            StatusCode::Dismissed => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn job_lifecycle() {
        let driver = ObjectDriver::default();

        // register job
        let mut status_info = StatusInfo::new("");
        status_info.job_id = driver
            .register(&status_info, Response::Document)
            .await
            .unwrap();

        // get results
        let process_results = driver.results(&status_info.job_id).await.unwrap();
        assert_eq!(process_results, ProcessResult::NotReady);

        // update
        status_info.status = StatusCode::Running;
        driver.update(&status_info).await.unwrap();

        // get status
        let status_info_updated = driver.status(&status_info.job_id).await.unwrap();
        assert_eq!(status_info_updated.unwrap(), status_info);

        // finish job
        let execute_results = ExecuteResults::new();
        driver
            .finish(
                &status_info.job_id,
                StatusCode::Successful,
                None,
                vec![],
                Some(execute_results.clone()),
            )
            .await
            .unwrap();

        // get status
        let status_info_finish = driver.status(&status_info.job_id).await.unwrap().unwrap();
        assert!(status_info_finish.finished.is_some());
        assert_eq!(status_info_finish.progress, Some(100));
        assert_eq!(status_info_finish.status, StatusCode::Successful);

        // get results
        let results = driver.results(&status_info.job_id).await.unwrap();
        assert_eq!(
            results,
            ProcessResult::Results {
                results: execute_results,
                response_mode: Response::Document
            }
        )
    }
}
