use std::sync::{Arc, OnceLock};

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use dashmap::DashMap;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Exception, Link,
        link_rel::{ITEM, SELF, TILESETS_VECTOR, TILING_SCHEME},
        media_type::{JSON, MVT},
    },
    tiles::{
        CollectionTileParams, DataType, TileMatrixSet, TileMatrixSetId, TileMatrixSetItem,
        TileMatrixSets, TileParams, TileQuery, TileSet, TileSetItem, TileSets,
    },
};

use crate::{
    AppState, Result,
    extractors::{Qs, RemoteUrl},
};

const CONFORMANCE: [&str; 7] = [
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tileset",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tilesets-list",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/dataset-tilesets",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geodata-tilesets",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/collection-selection",
    // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/jpeg",
    // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/png",
    "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/mvt",
    // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/geojson",
    // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/tiff",
    // "http://www.opengis.net/spec/ogcapi-tiles-1/1.0/conf/netcdf",
];

const WEB_MERCARTOR_QUAD: &[u8; 8744] = include_bytes!("../../assets/tms/WebMercartorQuad.json");

static TMS: OnceLock<Arc<DashMap<TileMatrixSetId, TileMatrixSet>>> = OnceLock::new();

/// Retrieve the list of available tiling schemes (tile matrix sets)
#[utoipa::path(get, path = "/tileMatrixSets", tag = "Tiling Schemes",
    responses(
        (
            status = 200,
            description = "List of tile matrix sets (tiling schemes).", 
            body = TileMatrixSets
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn tile_matrix_sets(RemoteUrl(url): RemoteUrl) -> Result<Json<TileMatrixSets>> {
    let registry = TMS.get().expect("TMS cell to be inizialized");

    let tile_matrix_sets = registry
        .iter()
        .map(|tms| {
            let path = format!("tileMatrixSets/{}", &tms.id);
            let url = url.join(&path).expect("failed to parse url");
            let link = Link::new(url, TILING_SCHEME);
            TileMatrixSetItem {
                id: Some(tms.id.to_owned()),
                title: tms.title.to_owned(),
                links: vec![link],
                ..Default::default()
            }
        })
        .collect();

    Ok(Json(TileMatrixSets { tile_matrix_sets }))
}

/// Retrieve the definition of the specified tiling scheme (tile matrix set)
#[utoipa::path(get, path = "/tileMatrixSets/{tileMatrixSetId}", tag = "Tiling Schemes",
    params(
        (
            "tileMatrixSetId" = TileMatrixSetId, Path,
            description = "Identifier for a supported TileMatrixSet"
        ),
    ),
    responses(
        (
            status = 200,
            description = "Tile matrix sets (tiling scheme).", 
            body = TileMatrixSet
        ),
        (
            status = 404,
            description = "The requested tile matrix set id was not found",
            body = Exception, example = json!(Exception::new_from_status(404))
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn tile_matrix_set(Path(id): Path<TileMatrixSetId>) -> Result<Json<TileMatrixSet>> {
    let registry = TMS.get().expect("TMS cell to be inizialized");

    if let Some(tms) = registry.get(&id) {
        Ok(Json(tms.to_owned()))
    } else {
        // fetch online
        let url = format!(
            "https://raw.githubusercontent.com/opengeospatial/2D-Tile-Matrix-Set/master/registry/json/{id}.json"
        );
        match reqwest::get(url).await {
            Ok(r) => match r.json::<TileMatrixSet>().await {
                Ok(tms) => {
                    // registry.insert(tms.id.to_owned(), tms);
                    Ok(Json(tms.to_owned()))
                }
                Err(e) => Err(Exception::new_from_status(500).detail(e.to_string()).into()),
            },
            Err(e) => {
                let status = e.status().map(|s| s.as_u16()).unwrap_or(500);
                Err(Exception::new_from_status(status)
                    .detail(e.to_string())
                    .into())
            }
        }
    }
}

/// Retrieve a list of available vector tilesets for the dataset
#[utoipa::path(get, path = "/tiles", tag = "Vector Tiles",
    responses(
        (
            status = 200,
            description = "List of available tilesets.", 
            body = TileSets
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn tiles(RemoteUrl(url): RemoteUrl) -> Result<Json<TileSets>> {
    let registry = TMS.get().expect("TMS cell to be inizialized");

    let mut tilesets = Vec::new();
    for tms in registry.iter() {
        let tms_path = format!("tileMatrixSets/{}", tms.id);
        let tms_url = url.join(&tms_path).expect("failed to parse url");
        let tms_link = Link::new(tms_url, TILING_SCHEME)
            .title("Tiling scheme definition")
            .mediatype(JSON);

        let self_path = format!("tiles/{}", tms.id);
        let self_url = url.join(&self_path).expect("failed to parse url");
        let self_link = Link::new(self_url, SELF)
            .title("Tileset definition")
            .mediatype(JSON);

        let tiles_path = format!("tiles/{}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}", tms.id);
        let tiles_url = url.join(&tiles_path).expect("failed to parse url");
        let tiles_link = Link::new(tiles_url, ITEM).mediatype(MVT).templated(true);

        let tileset = TileSetItem {
            title: Some(format!("Whole dataset in {}", tms.id)),
            data_type: DataType::Vector,
            crs: tms.crs.to_owned(),
            tile_matrix_set_uri: None,
            links: vec![self_link, tiles_link, tms_link],
        };

        tilesets.push(tileset);
    }

    let tile_sets = TileSets {
        tilesets,
        links: vec![],
    };

    Ok(Json(tile_sets))
}

/// Retrieve the vector tileset metadata for the whole dataset and the
/// specified tiling scheme (tile matrix set)
#[utoipa::path(get, path = "/tiles/{tileMatrixSetId}", tag = "Vector Tiles",
    params(
        (
            "tileMatrixSetId" = TileMatrixSetId, Path,
            description = "Identifier for a supported TileMatrixSet"
        )
    ),
    responses(
        (
            status = 200,
            description = "Description of the tileset", 
            body = TileSet
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn tiles_tile_set(
    RemoteUrl(url): RemoteUrl,
    Path(tms_id): Path<TileMatrixSetId>,
) -> Result<(HeaderMap, Json<TileSet>)> {
    let mut headers = HeaderMap::new();

    // tms
    let registry = TMS.get().expect("TMS cell to be inizialized");

    let Some(tms) = registry.get(&tms_id) else {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile matrix set `{tms_id}` not found"))
            .into());
    };

    let tms_path = format!("../tileMatrixSets/{tms_id}");
    let tms_url = url.join(&tms_path).expect("failed to parse url");

    // links
    let self_link = Link::new(url.clone(), SELF).mediatype(JSON);

    let tms_link = Link::new(tms_url.clone(), TILING_SCHEME).mediatype(JSON);

    let tiles_path = format!("{tms_id}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}");
    let tiles_url = url.join(&tiles_path).expect("failed to parse url");
    headers.insert(
        "Link-Template",
        format!(
            "<{}>; rel=\"{ITEM}\"; type=\"{MVT}\"; var-base=\"./vars/\"",
            tiles_url
        )
        .parse()
        .unwrap(),
    );
    let tiles_link = Link::new(tiles_url, ITEM).mediatype(MVT).templated(true);

    // tileset
    let tile_set = TileSet {
        title: Some(tms_id.to_string()),
        description: Default::default(),
        keywords: Default::default(),
        data_type: DataType::Vector,
        tile_matrix_set_uri: tms.uri.to_owned(),
        tile_matrix_set_limits: Default::default(),
        crs: tms.crs.to_owned(),
        epoch: Default::default(),
        links: vec![self_link, tms_link, tiles_link],
        layers: Default::default(),
        bounding_box: Default::default(),
        style: Default::default(),
        center_point: Default::default(),
        attribution: Default::default(),
        license: Default::default(),
        access_constraints: Default::default(),
        version: Default::default(),
        created: Default::default(),
        updated: Default::default(),
        point_of_contact: Default::default(),
        media_types: Default::default(),
    };

    Ok((headers, Json(tile_set)))
}

/// Retrieve a vector tile including one or more collections from the dataset.
#[utoipa::path(get,
    path = "/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}", 
    tag = "Vector Tiles",
    params(
        TileParams,
        TileQuery,
    ),
    responses(
        (
            status = 200,
            description = "A vector tile returned as a response.", 
            body = Vec<u8>
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn tiles_tile(
    Path(params): Path<TileParams>,
    Qs(query): Qs<TileQuery>,
    State(state): State<AppState>,
) -> Result<Vec<u8>> {
    // tile matrix set
    let tms_id = &params.tile_matrix_set_id;
    let Some(tms) = TMS.get().and_then(|tms| tms.get(tms_id)) else {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile matrix set `{tms_id}` not found",))
            .into());
    };

    // tile matrix
    let tm_id = &params.tile_matrix;
    let Some(tm) = tms.tile_matrices.iter().find(|tm| tm.id.as_str() == tm_id) else {
        return Err(Exception::new_from_status(404)
            .detail(format!(
                "No tile matrix with id `{tm_id}` in tile matrix set `{tms_id}`"
            ))
            .into());
    };

    // check bounds
    let row = params.tile_row;
    let col = params.tile_col;

    if row >= tm.matrix_height.get() as u32 || col >= tm.matrix_width.get() as u32 {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile row/col `{row}/{col}` out of bounds"))
            .into());
    }

    let tiles = state
        .drivers
        .tiles
        .tile(
            &query.collections,
            &tms,
            &params.tile_matrix,
            params.tile_row,
            params.tile_col,
        )
        .await?;

    Ok(tiles)
}

/// Retrieve a list of available vector tilesets for the collection
#[utoipa::path(get, path = "/collections/{collectionId}/tiles", tag = "Vector Tiles",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection")
    ),
    responses(
        (
            status = 200,
            description = "List of available tilesets.", 
            body = TileSets
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn collection_tiles(
    RemoteUrl(url): RemoteUrl,
    Path(collection_id): Path<String>,
) -> Result<Json<TileSets>> {
    let registry = TMS.get().expect("TMS cell to be inizialized");

    let mut tilesets = Vec::new();
    for tms in registry.iter() {
        let tms_path = format!("../../tileMatrixSets/{}", tms.id);
        let tms_url = url.join(&tms_path).expect("failed to parse url");
        let tms_link = Link::new(tms_url, TILING_SCHEME)
            .title("Tiling scheme definition")
            .mediatype(JSON);

        let tileset_path = format!("tiles/{}", tms.id);
        let tileset_url = url.join(&tileset_path).expect("failed to parse url");
        let tileset_link = Link::new(tileset_url, SELF)
            .title("Tileset definition")
            .mediatype(JSON);

        let tiles_path = format!("tiles/{}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}", tms.id);
        let tiles_url = url.join(&tiles_path).expect("failed to parse url");
        let tiles_link = Link::new(tiles_url, ITEM).mediatype(MVT).templated(true);

        let tileset = TileSetItem {
            title: Some(collection_id.to_owned()),
            data_type: DataType::Vector,
            crs: tms.crs.to_owned(),
            tile_matrix_set_uri: None,
            links: vec![tileset_link, tiles_link, tms_link],
        };

        tilesets.push(tileset);
    }

    let tile_sets = TileSets {
        tilesets,
        links: vec![],
    };

    Ok(Json(tile_sets))
}

/// Retrieve the vector tileset metadata for a specific collection and the
/// specified tiling scheme (tile matrix set)
#[utoipa::path(get, path = "/collections/{collectionId}/tiles/{tileMatrixSetId}", tag = "Vector Tiles",
    params(
        ("collectionId" = String, Path, description = "local identifier of a collection"),
        (
            "tileMatrixSetId" = TileMatrixSetId, Path,
            description = "Identifier for a supported TileMatrixSet"
        ),
    ),
    responses(
        (
            status = 200,
            description = "Description of the tileset", 
            body = TileSet
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn collection_tile_set(
    State(_state): State<AppState>,
    RemoteUrl(url): RemoteUrl,
    Path((collection_id, tms_id)): Path<(String, TileMatrixSetId)>,
) -> Result<(HeaderMap, Json<TileSet>)> {
    let mut headers = HeaderMap::new();

    // tms
    let registry = TMS.get().expect("TMS cell to be inizialized");

    let Some(tms) = registry.get(&tms_id) else {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile matrix set `{tms_id}` not found"))
            .into());
    };

    let tms_path = format!("../../../tileMatrixSets/{tms_id}");
    let tms_url = url.join(&tms_path).expect("failed to parse url");

    // links
    let self_url = url.join(&tms_id.to_string()).expect("failed to parse url");
    let self_link = Link::new(self_url, SELF).mediatype(JSON);

    let tms_link = Link::new(tms_url.clone(), TILING_SCHEME).mediatype(JSON);

    let tiles_path = format!(
        "/collections/{collection_id}/tiles/{tms_id}/{{tileMatrix}}/{{tileRow}}/{{tileCol}}"
    );
    let tiles_url = url.join(&tiles_path).expect("failed to parse url");
    headers.insert(
        "Link-Template",
        format!(
            "<{}>; rel=\"{ITEM}\"; type=\"{MVT}\"; var-base=\"./vars/\"",
            tiles_url
        )
        .parse()
        .unwrap(),
    );
    let tiles_link = Link::new(tiles_url, ITEM).mediatype(MVT).templated(true);

    // tileset
    let tile_set = TileSet {
        title: Some(collection_id.to_string()),
        description: Default::default(),
        keywords: Default::default(),
        data_type: DataType::Vector,
        tile_matrix_set_uri: tms.uri.to_owned(),
        tile_matrix_set_limits: Default::default(),
        crs: tms.crs.to_owned(),
        epoch: Default::default(),
        links: vec![self_link, tms_link, tiles_link],
        layers: Default::default(),
        bounding_box: Default::default(),
        style: Default::default(),
        center_point: Default::default(),
        attribution: Default::default(),
        license: Default::default(),
        access_constraints: Default::default(),
        version: Default::default(),
        created: Default::default(),
        updated: Default::default(),
        point_of_contact: Default::default(),
        media_types: Default::default(),
    };

    Ok((headers, Json(tile_set)))
}

/// Retrieve a vector tile from a collection.
#[utoipa::path(get,
    path = "/collections/{collectionId}/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}", 
    tag = "Vector Tiles",
    params(
        TileParams,
        TileQuery,
    ),
    responses(
        (
            status = 200,
            description = "A vector tile returned as a response.", 
            body = Vec<u8>
        ),
        (
            status = 500, description = "A server error occurred.", 
            body = Exception, example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn collection_tile(
    Path(params): Path<CollectionTileParams>,
    Qs(mut query): Qs<TileQuery>,
    State(state): State<AppState>,
) -> Result<Vec<u8>> {
    // tile matrix set
    let tms_id = &params.tile_params.tile_matrix_set_id;
    let Some(tms) = TMS.get().and_then(|tms| tms.get(tms_id)) else {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile matrix set `{tms_id}` not found",))
            .into());
    };

    // tile matrix
    let tm_id = &params.tile_params.tile_matrix;
    let Some(tm) = tms.tile_matrices.iter().find(|tm| tm.id.as_str() == tm_id) else {
        return Err(Exception::new_from_status(404)
            .detail(format!(
                "No tile matrix with id `{tm_id}` in tile matrix set `{tms_id}`"
            ))
            .into());
    };

    // check bounds
    let row = params.tile_params.tile_row;
    let col = params.tile_params.tile_col;

    if row >= tm.matrix_height.get() as u32 || col >= tm.matrix_width.get() as u32 {
        return Err(Exception::new_from_status(404)
            .detail(format!("Tile row/col `{row}/{col}` out of bounds"))
            .into());
    }

    let collections = if query.collections.is_empty() {
        vec![params.collection_id]
    } else {
        if !query.collections.contains(&params.collection_id) {
            query.collections.push(params.collection_id);
        }
        query.collections
    };

    let tiles = state
        .drivers
        .tiles
        .tile(
            &collections,
            &tms,
            &params.tile_params.tile_matrix,
            params.tile_params.tile_row,
            params.tile_params.tile_col,
        )
        .await?;

    Ok(tiles)
}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    let mut root = state.root.write().unwrap();
    root.links.extend([Link::new("tiles", TILESETS_VECTOR)
        .title("List of available vector features tilesets for the dataset")
        .mediatype(JSON)]);

    state.conformance.write().unwrap().extend(&CONFORMANCE);

    // Setup tile matrix sets
    let tms_map = DashMap::new();
    let web_mercartor_quad: TileMatrixSet =
        serde_json::from_slice(WEB_MERCARTOR_QUAD).expect("parse tms");
    tms_map.insert(web_mercartor_quad.id.to_owned(), web_mercartor_quad);
    TMS.set(Arc::new(tms_map)).expect("set `TMS` content");

    OpenApiRouter::new()
        .routes(routes!(tile_matrix_sets))
        .routes(routes!(tile_matrix_set))
        .routes(routes!(tiles))
        .routes(routes!(tiles_tile_set))
        .routes(routes!(tiles_tile))
        .routes(routes!(collection_tiles))
        .routes(routes!(collection_tile_set))
        .routes(routes!(collection_tile))
}
