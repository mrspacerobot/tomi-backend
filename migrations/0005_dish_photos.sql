-- The review service records a candidate before it can publish a photo.
CREATE TABLE tomi.dish_photos (
    recipe_id text NOT NULL CHECK (recipe_id ~ '^recipe/[a-z0-9-]+$'),
    recipe_fingerprint text NOT NULL CHECK (recipe_fingerprint ~ '^[0-9a-f]{64}$'),
    object_name text NOT NULL CHECK (object_name ~ '^[0-9a-f]{64}\.webp$'),
    shows text[] NOT NULL CHECK (cardinality(shows) > 0 AND array_position(shows, NULL) IS NULL),
    model text,
    prompt text,
    status text NOT NULL DEFAULT 'generated' CHECK (status IN ('generated', 'approved', 'rejected')),
    reviewed_by text,
    reviewed_at timestamptz,
    published_at timestamptz,
    PRIMARY KEY (recipe_id, recipe_fingerprint, object_name),
    CHECK (status = 'generated' OR (reviewed_by IS NOT NULL AND btrim(reviewed_by) <> '' AND reviewed_at IS NOT NULL)),
    CHECK (published_at IS NULL OR (status = 'approved' AND published_at >= reviewed_at))
);

ALTER TABLE tomi.dish_photos ENABLE ROW LEVEL SECURITY;

-- A single published photo per recipe version, with old versions available on rollback.
CREATE UNIQUE INDEX dish_photos_published_recipe_version
    ON tomi.dish_photos (recipe_id, recipe_fingerprint)
    WHERE status = 'approved' AND published_at IS NOT NULL;
