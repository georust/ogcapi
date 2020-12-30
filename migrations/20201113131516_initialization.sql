-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE EXTENSION IF NOT EXISTS postgis;

-- Tables
CREATE TABLE collections (
    id text PRIMARY KEY,
    title text,
    description text,
    links jsonb NOT NULL,
    extent jsonb,
    item_type jsonb DEFAULT '"unknown"'::jsonb,
    crs text[] DEFAULT '{"http://www.opengis.net/def/crs/OGC/1.3/CRS84"}',
    storage_crs text,
    storage_crs_coordinate_epoch real,
    stac_version text,
    stac_extensions text[],
    keywords text[],
    licence text,
    providers jsonb,
    summaries jsonb
);

CREATE TABLE features (
    id text DEFAULT uuid_generate_v4() ::text,
    collection text NOT NULL,
    feature_type jsonb NOT NULL,
    properties jsonb,
    geometry geometry NOT NULL,
    links jsonb,
    stac_version text,
    stac_extensions text[],
    bbox jsonb GENERATED ALWAYS AS (ST_AsGeoJSON(geometry, 9, 1)::jsonb -> 'bbox') STORED,
    assets jsonb,
    CONSTRAINT features_pkey PRIMARY KEY (id, collection),
    CONSTRAINT features_collection_fkey FOREIGN KEY (collection) REFERENCES public.collections (id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX features_properties_idx ON public.features USING gin (properties);
CREATE INDEX features_bbox_idx ON public.features USING gin (bbox);

CREATE INDEX features_geometry_idx ON public.features USING gist (geometry);

