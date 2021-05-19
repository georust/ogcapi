use tide::{Body, Request, Response, Result};

use crate::styles::{Style, Styles, Stylesheet};
use crate::Service;

pub async fn handle_styles(req: Request<Service>) -> Result {
    let styles: Vec<Style> = sqlx::query_as("SELECT id, title, links FROM styles")
        .fetch_all(&req.state().pool)
        .await?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&Styles { styles })?);
    Ok(res)
}

pub async fn read_style(req: Request<Service>) -> Result {
    let id: &str = req.param("id")?;
    let style: Stylesheet = sqlx::query_as(&format!(
        "SELECT id, stylesheet FROM styles WHERE id == {}",
        id
    ))
    .fetch_one(&req.state().pool)
    .await?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&style.value)?);
    Ok(res)
}

// pub async fn read_style_metadata(req: Request<Service>) -> Result {
//     let id: &str = req.param("id")?;
//     let meta: Metadata = sqlx::query_as(&format!("SELECT * FROM styles WHERE id == {}", id))
//         .fetch_one(&req.state().pool)
//         .await?;
//     let mut res = Response::new(200);
//     res.set_body(Body::from_json(&style)?);
//     Ok(res)
// }
