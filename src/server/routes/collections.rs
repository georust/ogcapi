use axum::{
    extract::{Extension, OriginalUri, Path},
    http::StatusCode,
    response::Headers,
    Json,
    {routing::get, Router},
};
// use serde::Deserialize;
// use serde_with::{serde_as, DisplayFromStr};
use url::{Position, Url};

use crate::common::{
    collections::{Collection, Collections},
    core::{Link, LinkRel, MediaType},
    crs::Crs,
};
use crate::server::{Result, State};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json",
];

// #[serde_as]
// #[derive(Deserialize, Debug, Clone)]
// #[serde(deny_unknown_fields)]
// struct Query {
//     bbox: Option<Bbox>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     bbox_crs: Option<Crs>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     datetime: Option<Datetime>,
//     limit: Option<isize>,
//     offset: Option<isize>,
// }

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

async fn collections(
    OriginalUri(uri): OriginalUri,
    Extension(state): Extension<State>,
) -> Result<Json<Collections>> {
    tracing::debug!("{:#?}", uri);
    let url = Url::parse("http://localhost:8484/collections").unwrap();

    //let mut query: Query = req.query()?;

    let mut collections: Vec<sqlx::types::Json<Collection>> =
        sqlx::query_scalar("SELECT collection FROM meta.collections")
            .fetch_all(&state.db.pool)
            .await?;

    let collections = collections
        .iter_mut()
        .map(|c| {
            let base = &url[..Position::AfterPath];
            c.0.links.append(&mut vec![
                Link::new(&format!("{}/{}", base, c.id)),
                Link::new(&format!("{}/{}/items", base, c.id))
                    .mime(MediaType::GeoJSON)
                    .title(format!("Items of {}", c.title.as_ref().unwrap_or(&c.id))),
            ]);
            c.0.to_owned()
        })
        .collect();

    let collections = Collections {
        links: vec![Link::new(url.as_str())
            .mime(MediaType::JSON)
            .title("this document".to_string())],
        crs: Some(vec![Crs::default(), Crs::from(4326)]),
        collections,
        ..Default::default()
    };

    Ok(Json(collections))
}

/// Create new collection metadata
async fn insert(
    Json(collection): Json<Collection>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, Headers<Vec<(&'static str, String)>>)> {
    let location = state.db.insert_collection(&collection).await?;
    let headers = Headers(vec![("Location", location)]);
    Ok((StatusCode::CREATED, headers))
}

/// Get collection metadata
async fn read(
    Path(collection_id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Collection>> {
    // TOOD: create custom extractor
    let url = Url::parse(&format!(
        "http://localhost:8484/collections/{}",
        collection_id
    ))
    .unwrap();

    let mut collection = state.db.select_collection(&collection_id).await?;

    collection.links.push(
        Link::new(&format!("{}/items", &url[..Position::AfterPath]))
            .mime(MediaType::GeoJSON)
            .title(format!(
                "Items of {}",
                collection.title.as_ref().unwrap_or(&collection.id)
            )),
    );

    Ok(Json(collection))
}

/// Update collection metadata
async fn update(
    Path(collection_id): Path<String>,
    Json(mut collection): Json<Collection>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    collection.id = collection_id;

    state.db.update_collection(&collection).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete collection metadata
async fn remove(
    Path(collection_id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    state.db.delete_collection(&collection_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new("http://ogcapi.rs/collections")
            .title("Metadata about the resource collections".to_string())
            .relation(LinkRel::Data)
            .mime(MediaType::JSON),
    );

    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new()
        .route("/collections", get(collections).post(insert))
        .route(
            "/collections/:collection_id",
            get(read).put(update).delete(remove),
        )
}
