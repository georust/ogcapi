#[cfg(feature = "processes")]
mod postgres {
    use ogcapi_drivers::{JobHandler, postgres::Db};
    use ogcapi_types::processes::{StatusCode, StatusInfo};

    #[sqlx::test]
    async fn job_handling(pool: sqlx::PgPool) -> () {
        let db = Db { pool };

        let job = StatusInfo {
            job_id: "test-job".to_string(),
            ..Default::default()
        };

        // register
        let job_id = db.register(&job).await.unwrap();

        assert_eq!(job_id, job.job_id);

        // status
        db.status(&job.job_id).await.unwrap();

        // dismiss
        let info = db.dismiss(&job.job_id).await.unwrap();

        assert_eq!(info.unwrap().status, StatusCode::Dismissed)
    }
}
