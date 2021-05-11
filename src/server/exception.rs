use serde::Serialize;
use tide::{Body, Response, Result};

#[derive(Serialize)]
struct Exception {
    code: String,
    description: Option<String>,
}

pub async fn exception(mut res: Response) -> Result {
    if let Some(err) = res.error() {
        let exception = Exception {
            code: res.status().to_string(),
            // NOTE: You may want to avoid sending error messages in a production server.
            description: Some(err.to_string()),
        };
        res.set_body(Body::from_json(&exception)?);
    }
    Ok(res)
}
