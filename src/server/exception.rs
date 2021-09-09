use crate::common::core::Exception;
use tide::{Body, Response, Result};

pub async fn exception(mut res: Response) -> Result {
    if let Some(err) = res.error() {
        let exception = Exception {
            r#type: format!(
                "https://httpwg.org/specs/rfc7231.html#status.{}",
                res.status().to_string()
            ),
            status: Some(res.status() as isize),
            // NOTE: You may want to avoid sending error messages in a production server.
            detail: Some(err.to_string()),
            ..Default::default()
        };
        res.set_body(Body::from_json(&exception)?);
    }
    Ok(res)
}
