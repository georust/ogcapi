use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ogcapi_processes::ProcessResponseBody;

pub(crate) struct ProcessResponse(pub(crate) ProcessResponseBody);

impl IntoResponse for ProcessResponse {
    fn into_response(self) -> Response {
        match self.0 {
            ProcessResponseBody::Requested(body) => Response::new(body.into()),
            ProcessResponseBody::Results(results) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&results).unwrap()))
                .unwrap(),
            ProcessResponseBody::Empty => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap(),
            ProcessResponseBody::StatusInfo(status_info) => Response::builder()
                .status(StatusCode::CREATED)
                .header("Content-Type", "application/json")
                .header("Location", &format!("../../jobs/{}", status_info.job_id))
                .body(Body::from(serde_json::to_vec(&status_info).unwrap()))
                .unwrap(),
        }
    }
}
