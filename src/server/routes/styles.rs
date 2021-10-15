use tide::{Body, Request, Response, Result, Server};

use crate::server::State;
use crate::styles::{Style, Styles, Stylesheet};

async fn styles(req: Request<State>) -> Result {
    let styles: Vec<Style> = sqlx::query_as("SELECT id, title, links FROM styles")
        .fetch_all(&req.state().db.pool)
        .await?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&Styles { styles })?);
    Ok(res)
}

async fn get(req: Request<State>) -> Result {
    let id: &str = req.param("id")?;
    let style: Stylesheet = sqlx::query_as(&format!(
        "SELECT id, stylesheet FROM styles WHERE id == {}",
        id
    ))
    .fetch_one(&req.state().db.pool)
    .await?;
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&style.value)?);
    Ok(res)
}

// async fn read_style_metadata(req: Request<Service>) -> Result {
//     let id: &str = req.param("id")?;
//     let meta: Metadata = sqlx::query_as(&format!("SELECT * FROM styles WHERE id == {}", id))
//         .fetch_one(&req.state().pool)
//         .await?;
//     let mut res = Response::new(200);
//     res.set_body(Body::from_json(&style)?);
//     Ok(res)
// }

pub(crate) fn register(app: &mut Server<State>) {
    app.at("/styles").get(styles);
    // .post(create_style);
    app.at("/styles/:id").get(get);
    // .put(update_style)
    // .delete(delete_style);
    // app.at("/styles/:id/metadata").get(read_style_matadata);
}
