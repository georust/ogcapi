use std::{collections::HashMap, sync::OnceLock};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use ogcapi_types::{
    common::{
        Crs, Exception, Link,
        link_rel::{TILESETS_VECTOR, TILING_SCHEME},
        media_type::JSON,
    },
    tiles::{
        DataType, TileMatrix, TileMatrixSet, TileMatrixSetId, TileMatrixSetItem, TileMatrixSets,
        TileQuery, TileSet, TileSets, TilesCrs,
    },
};

use crate::{
    AppState, Error, Result,
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

const WEB_MERCARTOR_QUAD: &[u8; 8005] = include_bytes!("../../assets/tms/WebMercartorQuad.json");

static TMS: OnceLock<HashMap<TileMatrixSetId, TileMatrixSet>> = OnceLock::new();
static TM: OnceLock<HashMap<TileMatrixSetId, HashMap<String, TileMatrix>>> = OnceLock::new();

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
    let tms = TMS.get().expect("TMS cell to be inizialized");

    let mut tile_matrix_sets = Vec::new();

    for tms in tms.values() {
        let item = TileMatrixSetItem {
            id: Some(tms.id.to_owned()),
            title: tms.title.to_owned(),
            links: vec![Link::new(
                url.join(&format!(
                    "tileMatrixSets/{}",
                    serde_json::to_string(&tms.id).unwrap()
                ))?,
                TILING_SCHEME,
            )],
            ..Default::default()
        };

        tile_matrix_sets.push(item);
    }

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
    match TMS.get().and_then(|tms| tms.get(&id)) {
        Some(tms) => Ok(Json(tms.to_owned())),
        None => Err(Error::ApiException(
            (StatusCode::NOT_FOUND, "Unable to find resource".to_string()).into(),
        )),
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
async fn tiles() -> Result<Json<TileSets>> {
    let tile_sets = TileSets {
        tilesets: vec![],
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
        ),
        TileQuery,
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
async fn tile_set() -> Result<Json<TileSet>> {
    let tile_set = TileSet {
        title: Default::default(),
        description: Default::default(),
        keywords: Default::default(),
        data_type: DataType::Vector,
        tile_matrix_set_uri: Default::default(),
        tile_matrix_set_limits: Default::default(),
        crs: TilesCrs::Simple(Crs::default().to_string()),
        epoch: Default::default(),
        links: Default::default(),
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

    Ok(Json(tile_set))
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
    let tms = TMS
        .get()
        .and_then(|tms| tms.get(&params.tile_matrix_set_id))
        .expect("Get tms from TMS");

    let tiles = state
        .drivers
        .tiles
        .tile(
            &query.collections,
            tms,
            &params.tile_matrix,
            params.tile_row,
            params.tile_col,
        )
        .await?;

    Ok(tiles)
}

#[derive(Serialize, Deserialize, IntoParams, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TileParams {
    /// Identifier selecting one of the TileMatrixSetId supported by the resource.
    tile_matrix_set_id: TileMatrixSetId,
    /// Identifier selecting one of the scales defined in the TileMatrixSet
    /// and representing the scaleDenominator the tile.
    tile_matrix: String,
    /// Row index of the tile on the selected TileMatrix. It cannot exceed
    /// the MatrixWidth-1 for the selected TileMatrix.
    tile_row: u32,
    /// Column index of the tile on the selected TileMatrix. It cannot exceed
    /// the MatrixHeight-1 for the selected TileMatrix.
    tile_col: u32,
}

/// Retrieve a vector tile from a collection.
#[utoipa::path(get,
    path = "/collections/{collectionId}/tiles/{tileMatrixSetId}/{tileMatrix}/{tileRow}/{tileCol}", 
    tag = "Vector Tiles",
    params(
        (
            "collectionId" = String, Path, 
            description = "Local identifier of a vector tile collection"
        ),
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
    Path(collection_id): Path<String>,
    Path(params): Path<TileParams>,
    Qs(mut query): Qs<TileQuery>,
    State(state): State<AppState>,
) -> Result<Vec<u8>> {
    let tms = TMS
        .get()
        .and_then(|tms| tms.get(&params.tile_matrix_set_id))
        .expect("Get tms from TMS");

    let collections = if !query.collections.is_empty() {
        if !query.collections.contains(&collection_id) {
            query.collections.push(collection_id);
        }
        query.collections
    } else {
        vec![collection_id]
    };

    let tiles = state
        .drivers
        .tiles
        .tile(
            &collections,
            tms,
            &params.tile_matrix,
            params.tile_row,
            params.tile_col,
        )
        .await?;

    Ok(tiles)
}

pub(crate) fn router(state: &AppState) -> OpenApiRouter<AppState> {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new("tiles", TILESETS_VECTOR)
            .title("List of available vector features tilesets for the dataset")
            .mediatype(JSON),
    );

    state.conformance.write().unwrap().extend(&CONFORMANCE);

    // Setup tile matrix sets
    let mut tms_map = HashMap::new();
    let web_mercartor_quad: TileMatrixSet =
        serde_json::from_slice(WEB_MERCARTOR_QUAD).expect("parse tms");
    tms_map.insert(web_mercartor_quad.id.to_owned(), web_mercartor_quad);

    let mut tm = HashMap::new();
    for tms in tms_map.values() {
        tm.insert(tms.id.to_owned(), HashMap::new());
        for tile_matrix in &tms.tile_matrices {
            tm.get_mut(&tms.id).and_then(|tm_map| {
                tm_map.insert(tile_matrix.id.to_owned(), tile_matrix.to_owned())
            });
        }
    }
    TMS.set(tms_map).expect("set `TMS` once cell content");
    TM.set(tm).expect("set `TM` once cell content");

    OpenApiRouter::new()
        .routes(routes!(tile_matrix_sets))
        .routes(routes!(tile_matrix_set))
        .routes(routes!(tiles))
        .routes(routes!(tile_set))
        .routes(routes!(tiles_tile))
        // .route("/collections/{collection_id}/tiles", get(tiles))
        .routes(routes!(collection_tile))
}
