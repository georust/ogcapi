use serde::Deserialize;
use tide::http::url::Position;
use tide::{Body, Request, Response, Result};

use crate::collections::{Collection, Collections};
use crate::common::{ContentType, Datetime, Link, LinkRelation, BBOX, CRS};
use crate::db::Db;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct Query {
    bbox: Option<BBOX>,
    bbox_crs: Option<CRS>,
    datetime: Option<Datetime>,
    limit: Option<isize>,
    offset: Option<isize>,
}

// impl Query {
//     fn to_string(&self) -> String {
//         let mut query_str = vec![];
//         if let Some(limit) = self.limit {
//             query_str.push(format!("limit={}", limit));
//         }
//         if let Some(offset) = self.offset {
//             query_str.push(format!("offset={}", offset));
//         }
//         if let Some(bbox) = &self.bbox {
//             query_str.push(format!("bbox={}", bbox));
//         }
//         if let Some(bbox_crs) = &self.bbox_crs {
//             query_str.push(format!("bboxCrs={}", bbox_crs.to_string()));
//         }
//         if let Some(datetime) = &self.datetime {
//             query_str.push(format!("datetime={}", datetime.to_string()));
//         }
//         query_str.join("&")
//     }
// }

pub async fn handle_collections(req: Request<Db>) -> Result {
    let url = req.url();

    //let mut query: Query = req.query()?;

    let sql = "SELECT * FROM collections";

    let mut collections: Vec<Collection> = sqlx::query_as(sql).fetch_all(&req.state().pool).await?;

    for collection in collections.iter_mut() {
        let mut links = vec![
            Link {
                href: format!("{}/{}/items", &url[..Position::AfterPath], collection.id),
                rel: LinkRelation::Items,
                r#type: Some(ContentType::GeoJSON),
                title: Some(format!(
                    "Items of {}",
                    collection.title.as_ref().unwrap_or(&collection.id)
                )),
                ..Default::default()
            },
            Link {
                href: format!("{}/{}", &url[..Position::AfterPath], collection.id),
                ..Default::default()
            },
        ];
        collection.links.append(&mut links);
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::JSON),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        crs: Some(vec![CRS::default().to_string()]),
        collections,
        ..Default::default()
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

/// Create new collection metadata
pub async fn create_collection(mut req: Request<Db>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    collection = req.state().create_collection(&collection).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Return collection metadata
pub async fn read_collection(req: Request<Db>) -> Result {
    // let url = req.url();

    let id: &str = req.param("collection")?;

    let collection = req.state().read_collection(id).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Update collection metadata
pub async fn update_collection(mut req: Request<Db>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    let id: &str = req.param("collection")?;
    collection.id = id.to_owned();

    collection = req.state().update_collection(&collection).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Delete collection metadata
pub async fn delete_collection(req: Request<Db>) -> Result {
    let id: &str = req.param("collection")?;

    sqlx::query_file!("sql/collection_delete.sql", id)
        .execute(&req.state().pool)
        .await?;

    Ok(Response::new(200))
}
