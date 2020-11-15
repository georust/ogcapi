use super::{Collection, CollectionType, Collections, Extent, Provider, Summaries};
use crate::common::{ContentType, Datetime, Link, LinkRelation, BBOX, CRS};
use crate::service::Service;
use serde::Deserialize;
use sqlx::types::Json;
use tide::http::url::Position;
use tide::{Body, Request, Response, Result};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct Query {
    limit: Option<isize>,
    offset: Option<isize>,
    bbox: Option<BBOX>,
    bbox_crs: Option<CRS>,
    datetime: Option<Datetime>,
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

pub async fn handle_collections(req: Request<Service>) -> Result {
    let url = req.url();

    //let mut query: Query = req.query()?;

    let sql = "SELECT * FROM collections";

    let mut collections: Vec<Collection> = sqlx::query_as(sql).fetch_all(&req.state().pool).await?;

    for collection in &mut collections {
        let link = Link {
            href: format!("{}/{}/items", &url[..Position::AfterPath], collection.id),
            rel: LinkRelation::Items,
            r#type: Some(ContentType::GEOJSON),
            title: Some(format!(
                "Items of {}",
                collection.title.clone().unwrap_or(collection.id.clone())
            )),
            ..Default::default()
        };
        collection.links.push(link);
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::JSON),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        crs: vec![CRS::default().to_string()],
        collections,
        ..Default::default()
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

/// Create new collection metadata
pub async fn create_collection(mut req: Request<Service>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    collection = sqlx::query_file_as!(
        Collection,
        "sql/collection_insert.sql",
        collection.id,
        collection.title,
        collection.description,
        collection.links as _,
        collection.extent as _,
        collection.collection_type as Option<CollectionType>,
        collection.crs.as_deref(),
        collection.storage_crs,
        collection.storage_crs_coordinate_epoch,
        collection.stac_version,
        collection.stac_extensions.as_deref(),
        collection.keywords.as_deref(),
        collection.licence,
        collection.providers as _,
        collection.summaries as _
    )
    .fetch_one(&req.state().pool)
    .await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Return collection metadata
pub async fn read_collection(req: Request<Service>) -> Result {
    // let url = req.url();

    let id: &str = req.param("collection")?;

    let collection: Collection = sqlx::query_file_as!(Collection, "sql/collection_select.sql", id)
        .fetch_one(&req.state().pool)
        .await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Update collection metadata
pub async fn update_collection(mut req: Request<Service>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    let id: &str = req.param("collection")?;
    assert!(id == collection.id);

    collection = sqlx::query_file_as!(
        Collection,
        "sql/collection_update.sql",
        collection.id,
        collection.title,
        collection.description,
        collection.links as _,
        collection.extent as _,
        collection.collection_type as Option<CollectionType>,
        collection.crs.as_deref(),
        collection.storage_crs,
        collection.storage_crs_coordinate_epoch,
        collection.stac_version,
        collection.stac_extensions.as_deref(),
        collection.keywords.as_deref(),
        collection.licence,
        collection.providers as _,
        collection.summaries as _
    )
    .fetch_one(&req.state().pool)
    .await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Delete collection metadata
pub async fn delete_collection(req: Request<Service>) -> Result {
    let id: &str = req.param("collection")?;

    sqlx::query_file!("sql/collection_delete.sql", id)
        .execute(&req.state().pool)
        .await?;

    Ok(Response::new(200))
}
