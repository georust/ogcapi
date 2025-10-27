CREATE TYPE response_type AS ENUM ('raw', 'document');

ALTER TABLE meta.jobs ADD COLUMN response response_type NOT NULL;
