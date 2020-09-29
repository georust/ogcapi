use super::{Collection, Collections};
use crate::common::{BBOX, CRS, ContentType, Datetime, Link, LinkRelation};
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
        let link = Json(Link {
            href: format!("{}/{}/items", &url[..Position::AfterPath], collection.id),
            rel: LinkRelation::Items,
            r#type: Some(ContentType::GEOJSON),
            title: Some(format!(
                "Items of {}",
                collection.title.clone().unwrap_or(collection.id.clone())
            )),
            ..Default::default()
        });
        collection.links.push(link);

        // set default item type
        if collection.item_type.is_none() {
            collection.item_type = Some("feature".to_string());
        }
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::JSON),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        crs: vec![CRS::default()],
        collections,
        ..Default::default()
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

/// Return collection metadata
pub async fn read_collection(req: Request<Service>) -> Result {
    // let url = req.url();

    let id: String = req.param("collection")?;

    let mut res = Response::new(200);
    let collection: Collection = sqlx::query_as("SELECT * FROM collections WHERE id = $1")
        .bind(id)
        .fetch_one(&req.state().pool)
        .await?;

    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Create new collection metadata
pub async fn create_collection(mut req: Request<Service>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    let sql = r#"
    INSERT INTO collections (
        id,
        title,
        description,
        links,
        extent,
        item_type,
        crs,
        storage_crs,
        storage_crs_coordinate_epoche,
        stac_version,
        stac_extension,
        keywords,
        licence,
        providers
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
    ) RETURNING *
    "#;

    let mut tx = req.state().pool.begin().await?;
    collection = sqlx::query_as(sql)
        .bind(&collection.id)
        .bind(&collection.title)
        .bind(&collection.description)
        .bind(&collection.links)
        .bind(&collection.extent)
        .bind(&collection.item_type)
        .bind(&collection.crs)
        .bind(&collection.storage_crs)
        .bind(&collection.storage_crs_coordinate_epoch)
        .bind(&collection.stac_version)
        .bind(&collection.stac_extensions)
        .bind(&collection.keywords)
        .bind(&collection.licence)
        .bind(&collection.providers)
        .fetch_one(&mut tx)
        .await?;
    tx.commit().await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Update collection metadata
pub async fn update_collection(mut req: Request<Service>) -> Result {
    let mut collection: Collection = req.body_json().await?;

    let id: String = req.param("collection")?;
    assert!(id == collection.id);

    let sql = r#"
    UPDATE collections
    SET (
        title,
        description,
        links,
        extent,
        item_type,
        crs,
        storage_crs,
        storage_crs_coordinate_epoche,
        stac_version,
        stac_extension,
        keywords,
        licence,
        providers
    ) = (
        $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
    )
    WHERE id = $1
    RETURNING *
    "#;

    let mut tx = req.state().pool.begin().await?;
    collection = sqlx::query_as(sql)
        .bind(&collection.id)
        .bind(&collection.title)
        .bind(&collection.description)
        .bind(&collection.links)
        .bind(&collection.extent)
        .bind(&collection.item_type)
        .bind(&collection.crs)
        .bind(&collection.storage_crs)
        .bind(&collection.storage_crs_coordinate_epoch)
        .bind(&collection.stac_version)
        .bind(&collection.stac_extensions)
        .bind(&collection.keywords)
        .bind(&collection.licence)
        .bind(&collection.providers)
        .fetch_one(&mut tx)
        .await?;
    tx.commit().await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}

/// Delete collection metadata
pub async fn delete_collection(req: Request<Service>) -> Result {
    let id: String = req.param("collection")?;

    let mut tx = req.state().pool.begin().await?;
    sqlx::query("DELETE FROM collections WHERE id = $1")
        .bind(id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;

    Ok(Response::new(200))
}
