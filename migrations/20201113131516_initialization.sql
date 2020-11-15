--Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE EXTENSION  IF NOT EXISTS postgis;

--Types
CREATE TYPE feature_type AS ENUM (
    'Feature'
);

CREATE TYPE collection_type AS ENUM (
    'feature'
);

--Tables
CREATE TABLE features (
    id text PRIMARY KEY DEFAULT uuid_generate_v4()::text,
    collection text NOT NULL,
    feature_type feature_type NOT NULL,
    properties jsonb,
    geometry geometry NOT NULL,
    links jsonb,
    stac_version text,
    stac_extensions text[],
    bbox double precision[],
    assets jsonb
);

CREATE TABLE collections (
    id text PRIMARY KEY,
    title text,
    description text,
    links jsonb NOT NULL,
    extent jsonb,
    collection_type collection_type DEFAULT 'feature',
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
