WITH collection AS (SELECT * from features WHERE collection = $1)
(
  SELECT ST_AsMVT(mvtgeom, 'collection', 4096, 'geom', 'id') FROM (
    SELECT ST_AsMVTGeom(ST_Transform(ST_Force2D(geom), 3857), ST_TileEnvelope($2, $3, $4), 4096, 64, TRUE) AS geom, collection, id, properties
    FROM collection
    WHERE geom && ST_Transform(ST_TileEnvelope($2, $3, $4, margin => (64.0 / 4096)), 4326)
  ) AS mvtgeom
)


--martin https://github.com/urbica/martin/blob/master/src/scripts/get_tile.sql
--
-- WITH bounds AS (SELECT {mercator_bounds} as mercator, {original_bounds} as original)
-- SELECT ST_AsMVT(tile, '{id}', {extent}, 'geom' {id_column}) FROM (
--   SELECT
--     ST_AsMVTGeom({geometry_column_mercator}, bounds.mercator, {extent}, {buffer}, {clip_geom}) AS geom {properties}
--   FROM {id}, bounds
--   WHERE {geometry_column} && bounds.original
-- ) AS tile WHERE geom IS NOT NULL


--pg_tileserve https://github.com/CrunchyData/pg_tileserv/blob/master/layer_table.go
--
-- SELECT ST_AsMVT(mvtgeom, {{ .MvtParams }}) FROM (
-- 		SELECT ST_AsMVTGeom(
-- 			ST_Transform(ST_Force2D(t."{{ .GeometryColumn }}"), {{ .TileSrid }}),
-- 			bounds.geom_clip,
-- 			{{ .Resolution }},
-- 			{{ .Buffer }}
-- 		  ) AS "{{ .GeometryColumn }}"
-- 		  {{ if .Properties }}
-- 		  , {{ .Properties }}
-- 		  {{ end }}
-- 		FROM "{{ .Schema }}"."{{ .Table }}" t, (
-- 			SELECT {{ .TileSql }}  AS geom_clip,
-- 					{{ .QuerySql }} AS geom_query
-- 			) bounds
-- 		WHERE ST_Intersects(t."{{ .GeometryColumn }}",
-- 							ST_Transform(bounds.geom_query, {{ .Srid }}))
-- 		{{ .Limit }}
-- 	) mvtgeom

