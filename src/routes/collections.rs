use crate::common::crs;
use crate::common::link::{ContentType, Link, LinkRelation};
use crate::features::schema::{Collection, Collections};
use crate::features::service::State;
use sqlx::types::Json;
use tide::http::{url::Position, Method};
use tide::{Body, Request, Response, Result};

pub async fn handle_collections(req: Request<State>) -> Result {
    let url = req.url();

    let mut collections: Vec<Collection> = sqlx::query_as("SELECT * FROM meta.collections")
        .fetch_all(&req.state().pool)
        .await?;

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
        // handle default crs
        match &collection.crs {
            Some(crs) => crs.to_owned().push("#/crs".to_string()),
            None => collection.crs = Some(vec!["#/crs".to_owned()]),
        }
    }

    let collections = Collections {
        links: vec![Link {
            href: url.to_string(),
            r#type: Some(ContentType::JSON),
            title: Some("this document".to_string()),
            ..Default::default()
        }],
        crs: vec![crs::EPSG_WGS84.to_owned(), crs::EPSG_4979.to_owned()],
        collections,
    };

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&collections)?);
    Ok(res)
}

pub async fn handle_collection(mut req: Request<State>) -> Result {
    let url = req.url();
    let method = req.method();

    let id: Option<String> = if method != Method::Post {
        Some(req.param("collection")?)
    } else {
        None
    };

    let mut res = Response::new(200);
    let mut collection: Collection;

    match method {
        Method::Get => {
            collection = sqlx::query_as("SELECT * FROM meta.collections WHERE id = $1")
                .bind(id)
                .fetch_one(&req.state().pool)
                .await?;

            let link = Json(Link {
                href: format!("{}/items", &url[..Position::AfterPath]),
                rel: LinkRelation::Items,
                r#type: Some(ContentType::GEOJSON),
                title: collection.title.clone(),
                ..Default::default()
            });
            collection.links.push(link);
        }
        Method::Post | Method::Put => {
            collection = req.body_json().await?;

            let mut sql = if method == Method::Post {
                vec![
                    "INSERT INTO meta.collections",
                    "(id, title, description, links, extent, item_type, crs, storage_crs, storage_crs_coordinate_epoche)",
                    "VALUES ($1, $2, $3, $4, $5, $6, $7)",
                ]
            } else {
                vec![
                    "UPDATE meta.collections",
                    "SET title = $2, description = $3, links = $4, extent = $5, item_type = $6, crs = $7, storage_crs = $8, storage_crs_coordinate_epoche = $9)",
                    "WHERE id = $1",
                ]
            };
            sql.push("RETURNING id, title, description, links, extent, item_type, crs, storage_crs, storage_crs_coordinate_epoche");

            let mut tx = req.state().pool.begin().await?;
            collection = sqlx::query_as(&sql.join(" ").as_str())
                .bind(&collection.id)
                .bind(&collection.title)
                .bind(&collection.description)
                .bind(&collection.links)
                .bind(&collection.extent)
                .bind(&collection.item_type)
                .bind(&collection.crs)
                .bind(&collection.storage_crs)
                .bind(&collection.storage_crs_coordinate_epoch)
                .fetch_one(&mut tx)
                .await?;
            tx.commit().await?;
        }
        Method::Delete => {
            let mut tx = req.state().pool.begin().await?;
            let _deleted = sqlx::query("DELETE FROM meta.collections WHERE id = $1")
                .bind(id)
                .execute(&mut tx)
                .await?;
            tx.commit().await?;

            return Ok(res);
        }
        _ => unimplemented!(),
    }
    res.set_body(Body::from_json(&collection)?);
    Ok(res)
}
