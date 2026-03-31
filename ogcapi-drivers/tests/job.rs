#[cfg(feature = "processes")]
mod postgres {
    use std::collections::HashMap;

    use ogcapi_drivers::{JobHandler, ProcessResult, postgres::Db};
    use ogcapi_types::processes::{
        ExecuteResult, InlineOrRefData, InputValueNoObject, Output, Response, StatusCode,
        StatusInfo,
    };

    #[sqlx::test]
    async fn job_handling(pool: sqlx::PgPool) -> () {
        let db = Db { pool };

        let job = StatusInfo {
            job_id: "test-job".to_string(),
            ..Default::default()
        };

        // register
        let job_id = db.register(&job, Response::default()).await.unwrap();

        assert_eq!(job_id, job.job_id);

        // status
        db.status(&job.job_id).await.unwrap();

        // dismiss
        let info = db.dismiss(&job.job_id).await.unwrap();

        assert_eq!(info.unwrap().status, StatusCode::Dismissed)
    }

    #[sqlx::test]
    async fn job_result(pool: sqlx::PgPool) -> () {
        let db = Db { pool };

        let job = StatusInfo {
            job_id: "test-job".to_string(),
            ..Default::default()
        };

        matches!(
            db.results(&job.job_id).await.unwrap(),
            ProcessResult::NoSuchJob
        );

        assert_eq!(
            db.register(&job, Response::Document).await.unwrap(),
            job.job_id
        );

        matches!(
            db.results(&job.job_id).await.unwrap(),
            ProcessResult::NotReady
        );

        db.finish(
            &job.job_id,
            &StatusCode::Successful,
            Some("it is ready".to_string()),
            vec![],
            Some(HashMap::from([(
                "key1".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        "foobar".into(),
                    )),
                },
            )])),
        )
        .await
        .unwrap();

        matches!(
            db.results(&job.job_id).await.unwrap(),
            ProcessResult::Results {
                results: _,
                response_mode: Response::Document,
            }
        );
    }

    #[sqlx::test]
    async fn job_status_list(pool: sqlx::PgPool) -> () {
        use ogcapi_types::common::Link;

        let db = Db { pool };

        let job = StatusInfo {
            job_id: "test-job-status-list".to_string(),
            status: StatusCode::Running,
            links: Vec::<Link>::new(),
            ..Default::default()
        };

        // register the job with running status and empty links
        assert_eq!(
            db.register(&job, Response::default()).await.unwrap(),
            job.job_id
        );

        // query the status list
        let list = db.status_list(0, 10).await.unwrap();

        // find our job in the returned list
        let found = list
            .into_iter()
            .find(|s| s.job_id == "test-job-status-list");

        assert!(found.is_some());
        let info = found.unwrap();
        assert_eq!(info.status, StatusCode::Running);
        assert!(info.links.is_empty());
    }

    #[sqlx::test]
    async fn job_result_failed(pool: sqlx::PgPool) -> () {
        let db = Db { pool };

        let job = StatusInfo {
            job_id: "test-job".to_string(),
            ..Default::default()
        };

        let _ = db.register(&job, Response::Document).await.unwrap();

        db.finish(&job.job_id, &StatusCode::Failed, None, vec![], None)
            .await
            .unwrap();

        matches!(
            db.results(&job.job_id).await.unwrap(),
            ProcessResult::Results {
                results: _,
                response_mode: Response::Document,
            }
        );
    }
}
