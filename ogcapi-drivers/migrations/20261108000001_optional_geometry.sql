-- a feature may have no geometry (RFC 7946, section 3.2), in the collections
-- created before too
DO $$
DECLARE
    c record;
BEGIN
    FOR c IN SELECT id FROM meta.collections LOOP
        IF to_regclass(format('items.%I', c.id)) IS NOT NULL THEN
            EXECUTE format('ALTER TABLE items.%I ALTER COLUMN geom DROP NOT NULL', c.id);
        END IF;
    END LOOP;
END
$$;
