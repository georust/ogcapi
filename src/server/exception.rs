use crate::common::Exception;
use tide::{Body, Response, Result};

pub async fn exception(mut res: Response) -> Result {
    if let Some(err) = res.error() {
        let exception = Exception {
            r#type: res.status().to_string(),
            title: None,
            status: Some(res.status() as isize),
            // NOTE: You may want to avoid sending error messages in a production server.
            detail: Some(err.to_string()),
            instance: None,
        };
        res.set_body(Body::from_json(&exception)?);
    }
    Ok(res)
}
