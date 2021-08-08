-- Extensions
CREATE EXTENSION IF NOT EXISTS postgis;

-- Tables
CREATE TABLE root (
    href text NOT NULL,
    rel text,
    type text,
    hreflang text,
    title text,
    length integer
);

CREATE TABLE conformance (
    class text PRIMARY KEY
);

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
    id bigserial,
    collection text NOT NULL,
    feature_type jsonb NOT NULL DEFAULT '"Feature"'::jsonb,
    properties jsonb,
    geom geometry NOT NULL,
    links jsonb,
    stac_version text,
    stac_extensions text[],
    assets jsonb,
    CONSTRAINT features_pkey PRIMARY KEY (id, collection),
    CONSTRAINT features_collection_fkey FOREIGN KEY (collection) REFERENCES public.collections (id) ON DELETE CASCADE
);

SELECT UpdateGeometrySRID('features', 'geom', 4326);

CREATE TABLE styles (
    id text PRIMARY KEY,
    title text,
    links jsonb NOT NULL
);

-- Indexes
CREATE INDEX features_properties_idx ON public.features USING gin (properties);
CREATE INDEX features_geom_idx ON public.features USING gist (geom);

-- Insertions
INSERT INTO
    conformance (class)
VALUES
    ('http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core'),
    ('http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections'),
    ('http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson');

INSERT INTO
    root (href, rel, type, title)
VALUES
    ('/', 'self', 'application/json','This document'),
    ('/api', 'service-desc', 'application/vnd.oai.openapi+json;version=3.0','The Open API definition'),
    ('/conformance', 'conformance', 'application/json','OGC conformance classes implemented by this API'),
    ('/collections', 'data', 'application/json','Metadata about the resource collections');
