use axum::http::StatusCode;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use url::Url;

pub struct RemoteUrl(pub Url);

#[async_trait]
impl<B> FromRequest<B> for RemoteUrl
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(url) = req
            .headers()
            .and_then(|headers| {
                headers.get("Forwarded").and_then(|header| {
                    header.to_str().ok().and_then(|value| {
                        value.split(';').find_map(|key_equals_value| {
                            let parts = key_equals_value.split('=').collect::<Vec<_>>();
                            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("for") {
                                Some(parts[1])
                            } else {
                                headers.get("X-Forwarded-For").and_then(|header| {
                                    header
                                        .to_str()
                                        .ok()
                                        .and_then(|value| value.split(',').next())
                                })
                            }
                        })
                    })
                })
            })
            .and_then(|value| Url::parse(value).ok())
        {
            Ok(RemoteUrl(url))
        } else {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to extract remote URL",
            ))
        }
    }
}
