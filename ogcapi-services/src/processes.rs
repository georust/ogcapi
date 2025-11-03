use crate::Error;
use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Request},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use hyper::header::{self, InvalidHeaderValue, LINK, LOCATION};
use mail_builder::headers::HeaderType;
use mail_builder::headers::content_type::ContentType;
use mail_builder::headers::text::Text;
use mail_builder::mime::{BodyPart, MimePart};
use ogcapi_types::{
    common::{Exception, Link},
    processes::{ExecuteResult, ExecuteResults, InlineOrRefData, StatusInfo},
};
use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Write;

pub(crate) struct ProcessResultsResponse {
    pub results: ExecuteResults,
    pub response_mode: ogcapi_types::processes::Response,
}

pub(crate) enum ProcessExecuteResponse {
    Synchronous {
        results: ProcessResultsResponse,
        was_preferred_execution_mode: bool,
    },
    Asynchronous {
        status_info: StatusInfo,
        was_preferred_execution_mode: bool,
        base_url: String,
    },
}

impl ProcessExecuteResponse {
    fn preference_header(&self) -> PreferredResponseHeader {
        match self {
            ProcessExecuteResponse::Synchronous {
                was_preferred_execution_mode: true,
                ..
            } => PreferredResponseHeader::RespondSync,
            ProcessExecuteResponse::Asynchronous {
                was_preferred_execution_mode: true,
                ..
            } => PreferredResponseHeader::RespondAsync,
            _ => PreferredResponseHeader::None,
        }
    }
}

/// Add Preference-Applied header to response based on negotiated execution mode.
///
/// cf. <https://datatracker.ietf.org/doc/html/rfc7240#section-3>
enum PreferredResponseHeader {
    RespondSync,
    RespondAsync,
    None,
}

impl IntoResponseParts for PreferredResponseHeader {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        const PREFERENCE_KEY: &str = "Preference-Applied";

        match self {
            PreferredResponseHeader::RespondSync => {
                res.headers_mut()
                    .insert(PREFERENCE_KEY, HeaderValue::from_static("respond-sync"));
            }
            PreferredResponseHeader::RespondAsync => {
                res.headers_mut()
                    .insert(PREFERENCE_KEY, HeaderValue::from_static("respond-async"));
            }
            PreferredResponseHeader::None => { /* nothing to do */ }
        };

        Ok(res)
    }
}

struct LocationHeader(Result<HeaderValue, InvalidHeaderValue>);

impl LocationHeader {
    fn new(base_url: &str, job_id: &str) -> Self {
        Self(format!("{base_url}/jobs/{job_id}").parse())
    }
}

impl IntoResponseParts for LocationHeader {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        match self.0 {
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to create Location header: {}", e),
                ));
            }
            Ok(location) => res.headers_mut().insert(LOCATION, location),
        };

        Ok(res)
    }
}

impl IntoResponse for ProcessResultsResponse {
    fn into_response(self) -> Response {
        if let ogcapi_types::processes::Response::Document = self.response_mode {
            let results = ogcapi_types::processes::Results {
                results: self
                    .results
                    .into_iter()
                    .map(|(name, result)| (name, result.data))
                    .collect(),
            };
            return Json(results).into_response();
        };

        match self.results.len() {
            0 => (StatusCode::NO_CONTENT, Body::empty()).into_response(),
            1 => {
                let (_, result) = self.results.into_iter().next().expect("checked");
                SingleResponse(result).into_response()
            }
            _ => MultipartResponse::new(self.results).into_response(),
        }
    }
}

impl IntoResponse for ProcessExecuteResponse {
    fn into_response(self) -> Response {
        let preference_header = self.preference_header();

        match self {
            ProcessExecuteResponse::Synchronous {
                results,
                was_preferred_execution_mode: _,
            } => (preference_header, results).into_response(),
            ProcessExecuteResponse::Asynchronous {
                status_info,
                was_preferred_execution_mode: _,
                base_url,
            } => {
                let location_header = LocationHeader::new(&base_url, &status_info.job_id);

                // `/req/core/process-execute-success-async`
                (
                    StatusCode::CREATED,
                    location_header,
                    preference_header,
                    Json(status_info),
                )
                    .into_response()
            }
        }
    }
}

struct SingleResponse(ExecuteResult);

impl IntoResponse for SingleResponse {
    fn into_response(self) -> Response {
        let ExecuteResult { output: _, data } = self.0;

        match data {
            InlineOrRefData::Link(link) => {
                // Cf. link header <https://datatracker.ietf.org/doc/html/rfc8288>

                (StatusCode::NO_CONTENT, LinkHeader(link), Body::empty()).into_response()
            }
            _ => {
                let body = to_binary(data);
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, body.content_type)],
                    Body::from(body.data),
                )
                    .into_response()
            }
        }
    }
}

struct LinkHeader(Link);

impl IntoResponseParts for LinkHeader {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        fn add_quoted_string(header: &mut String, key: &str, value: &str) {
            header.push_str("; ");
            header.push_str(key);
            header.push_str("=\"");
            for c in value.chars() {
                if c == '"' || c == '\\' {
                    header.push('\\');
                }
                header.push(c);
            }
            header.push('"');
        }

        let link = self.0;

        let mut link_header = String::from("<");
        link_header.push_str(&link.href);
        link_header.push('>');

        add_quoted_string(&mut link_header, "rel", &link.rel);

        if let Some(type_) = &link.r#type {
            add_quoted_string(&mut link_header, "type", type_);
        }
        if let Some(href_lang) = &link.hreflang {
            add_quoted_string(&mut link_header, "hreflang", href_lang);
        }

        if let Some(title) = &link.title {
            add_quoted_string(&mut link_header, "title", title);
        }

        if let Some(length) = link.length {
            let _ = write!(link_header, "; length={}", length);
        }

        match link_header.parse() {
            Ok(header_value) => {
                res.headers_mut().insert(LINK, header_value);
                Ok(res)
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create Link header: {}", e),
            )),
        }
    }
}

struct MultipartResponse {
    parts: ExecuteResults,
    boundary: Option<String>,
}
impl MultipartResponse {
    fn new(parts: ExecuteResults) -> Self {
        Self {
            parts,
            boundary: None,
        }
    }

    #[cfg(test)]
    fn new_with_boundary(parts: ExecuteResults, boundary: String) -> Self {
        Self {
            parts,
            boundary: Some(boundary),
        }
    }
}

impl IntoResponse for MultipartResponse {
    fn into_response(self) -> Response {
        let mut mime_parts = Vec::<MimePart>::with_capacity(self.parts.len());

        let parts = self.parts;
        #[cfg(test)]
        let parts = parts
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();

        for (name, result) in parts {
            let mut mime_part = to_mime_part(result.data);
            mime_part = mime_part.header("Content-ID", Text::new(name));
            mime_parts.push(mime_part);
        }

        let mut content_type = ContentType::new("multipart/related");
        if let Some(boundary) = self.boundary {
            content_type = content_type.attribute("boundary", boundary);
        }
        let multipart_mime_part = MimePart::new(content_type, mime_parts);

        let mut body = Vec::new();
        let write_result = multipart_mime_part.write_part(&mut body);

        if let Err(e) = write_result {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write multipart MIME response: {}", e),
            )
                .into_response();
        }

        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "multipart/related")],
            Body::from(body),
        )
            .into_response()
    }
}

struct BinaryData {
    data: Vec<u8>,
    content_type: String,
}

fn to_mime_part(data: InlineOrRefData) -> MimePart<'static> {
    fn extract_header(
        (name, value): (&axum::http::HeaderName, &axum::http::HeaderValue),
    ) -> (Cow<'static, str>, HeaderType<'static>) {
        let name = name.as_str().to_string();
        let value = value.to_str().unwrap_or_default().to_string();
        (Cow::Owned(name), HeaderType::Text(Text::new(value)))
    }

    match data {
        InlineOrRefData::Link(link) => {
            let link_response = (LinkHeader(link), ()).into_response();

            MimePart {
                headers: link_response
                    .headers()
                    .into_iter()
                    .map(extract_header)
                    .collect(),
                contents: BodyPart::from(&[] as &[u8]),
            }
        }
        data => {
            let body = to_binary(data);
            MimePart::new(body.content_type, body.data)
        }
    }
}

fn to_binary(data: InlineOrRefData) -> BinaryData {
    match data {
        InlineOrRefData::InputValueNoObject(ivno) => input_value_no_object_to_binary(ivno),
        InlineOrRefData::Link(_link) => BinaryData {
            data: Vec::new(),
            content_type: String::new(),
        },
        InlineOrRefData::QualifiedInputValue(qiv) => qualified_input_value_to_binary(qiv),
    }
}

fn input_value_no_object_to_binary(
    ivno: ogcapi_types::processes::InputValueNoObject,
) -> BinaryData {
    match ivno {
        ogcapi_types::processes::InputValueNoObject::String(s) => BinaryData {
            data: s.into_bytes(),
            content_type: "text/plain; charset=utf-8".to_string(),
        },
        ogcapi_types::processes::InputValueNoObject::Integer(i) => BinaryData {
            data: i.to_string().into_bytes(),
            content_type: "text/plain; charset=utf-8".to_string(),
        },
        ogcapi_types::processes::InputValueNoObject::Number(f) => BinaryData {
            data: f.to_string().into_bytes(),
            content_type: "text/plain; charset=utf-8".to_string(),
        },
        ogcapi_types::processes::InputValueNoObject::Boolean(b) => BinaryData {
            data: b.to_string().into_bytes(),
            content_type: "text/plain; charset=utf-8".to_string(),
        },
        ogcapi_types::processes::InputValueNoObject::Array(items) => {
            // TODO: verify correct serialization
            BinaryData {
                data: items.join(",").into_bytes(),
                content_type: "application/json".to_string(),
            }
        }
        ogcapi_types::processes::InputValueNoObject::Bbox(bounding_box) => {
            // TODO: verify correct serialization
            BinaryData {
                data: serde_json::to_vec(&bounding_box).unwrap_or_default(),
                content_type: "application/json".to_string(),
            }
        }
    }
}

fn qualified_input_value_to_binary(
    qiv: ogcapi_types::processes::QualifiedInputValue,
) -> BinaryData {
    match qiv.value {
        ogcapi_types::processes::InputValue::InputValueNoObject(value) => {
            let mut binary_data = input_value_no_object_to_binary(value);
            // TODO: verify that this is enough to respect the format
            if let Some(media_type) = qiv.format.media_type {
                binary_data.content_type = media_type;

                if let Some(encoding) = qiv.format.encoding {
                    binary_data.content_type.push_str("; charset=");
                    binary_data.content_type.push_str(&encoding);
                }
            }
            binary_data
        }
        ogcapi_types::processes::InputValue::Object(object) => {
            // TODO: verify correct serialization
            BinaryData {
                data: serde_json::to_vec(&object).unwrap_or_default(),
                content_type: "application/json".to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct ValidParams<T>(pub T);

impl<T, S> FromRequest<S> for ValidParams<T>
where
    T: FromRequest<S>,
    T::Rejection: std::fmt::Display,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let json = T::from_request(req, state).await;

        match json {
            Ok(value) => Ok(ValidParams(value)),
            Err(rejection) => {
                // let response_body = rejection.body_text();
                Err(Error::ApiException(
                    Exception::new("InvalidParameterValue")
                        .status(404)
                        .title("InvalidParameterValue")
                        .detail(format!(
                            "The following parameters are not recognized: {rejection}",
                        )),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ogcapi_types::processes::Output;
    use std::collections::HashMap;

    #[test]
    fn it_creates_link_headers() {
        let link = Link::new("http://example.com", "examp\"le");
        let response = (LinkHeader(link), ()).into_response();
        assert_eq!(
            response.headers().get("Link").unwrap(),
            "<http://example.com>; rel=\"examp\\\"le\""
        );
    }

    #[tokio::test]
    async fn it_creates_multipart_response() {
        let mut results = HashMap::new();
        results.insert(
            "output1".to_string(),
            ExecuteResult {
                output: Output {
                    format: None,
                    transmission_mode: ogcapi_types::processes::TransmissionMode::Value,
                },
                data: InlineOrRefData::InputValueNoObject(
                    ogcapi_types::processes::InputValueNoObject::String("Hello".to_string()),
                ),
            },
        );
        results.insert(
            "output2".to_string(),
            ExecuteResult {
                output: Output {
                    format: None,
                    transmission_mode: ogcapi_types::processes::TransmissionMode::Value,
                },
                data: InlineOrRefData::InputValueNoObject(
                    ogcapi_types::processes::InputValueNoObject::Integer(42),
                ),
            },
        );

        let multipart_response =
            MultipartResponse::new_with_boundary(results, "boundary-boundary-boundary".into());
        let response = multipart_response.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("multipart/related")
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str: String = String::from_utf8(body.as_ref().to_vec()).unwrap();

        assert_eq!(
            body_str,
            concat!(
                "Content-Type: multipart/related; boundary=\"boundary-boundary-boundary\"\r\n\r\n\r\n",
                "--boundary-boundary-boundary\r\n",
                "Content-Type: text/plain; charset=utf-8\r\n",
                "Content-ID: output1\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "Hello\r\n",
                "--boundary-boundary-boundary\r\n",
                "Content-Type: text/plain; charset=utf-8\r\n",
                "Content-ID: output2\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\n",
                "42\r\n",
                "--boundary-boundary-boundary--\r\n"
            )
        )
    }
}
