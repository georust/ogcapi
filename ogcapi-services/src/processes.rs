use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ogcapi_processes::ProcessResponseBody;
use ogcapi_types::common::Exception;

pub(crate) struct ProcessResponse(pub(crate) ProcessResponseBody);

impl IntoResponse for ProcessResponse {
    fn into_response(self) -> Response {
        match self.0 {
            ProcessResponseBody::Requested { outputs, mut parts } => {
                if outputs.len() > 1 {
                    let status = StatusCode::INTERNAL_SERVER_ERROR;
                    let exeption = Exception::new_from_status(status.as_u16())
                        .detail("Multiple raw outputs as multipart/related not implemented yet!");
                    Response::builder()
                        .status(status)
                        .body(Body::from(serde_json::to_vec(&exeption).unwrap()))
                        .unwrap()
                } else {
                    let Some(content_type) = outputs
                        .values()
                        .next()
                        .unwrap()
                        .format
                        .as_ref()
                        .and_then(|f| f.media_type.as_ref())
                    else {
                        let status = StatusCode::INTERNAL_SERVER_ERROR;
                        let exeption = Exception::new_from_status(status.as_u16()).detail("Single sync raw response requires format and media-type in output definition");
                        return Response::builder()
                            .status(status)
                            .body(Body::from(serde_json::to_vec(&exeption).unwrap()))
                            .unwrap();
                    };
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", content_type)
                        .body(Body::from(parts.remove(0)))
                        .unwrap()
                }
            }
            ProcessResponseBody::Results(results) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&results).unwrap()))
                .unwrap(),
            ProcessResponseBody::Empty(link) => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .header("Link", link)
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
