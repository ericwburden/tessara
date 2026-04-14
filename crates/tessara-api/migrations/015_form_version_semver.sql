ALTER TABLE form_versions
    ALTER COLUMN version_label DROP NOT NULL;

ALTER TABLE form_versions
    ADD COLUMN version_major integer,
    ADD COLUMN version_minor integer,
    ADD COLUMN version_patch integer,
    ADD COLUMN semantic_bump text,
    ADD COLUMN started_new_major_line boolean NOT NULL DEFAULT false;

ALTER TABLE form_versions
    ADD CONSTRAINT form_versions_semantic_bump_check CHECK (
        semantic_bump IS NULL OR semantic_bump IN ('INITIAL', 'PATCH', 'MINOR', 'MAJOR')
    );

CREATE UNIQUE INDEX form_versions_semver_unique_idx
    ON form_versions (form_id, version_major, version_minor, version_patch)
    WHERE version_major IS NOT NULL
      AND version_minor IS NOT NULL
      AND version_patch IS NOT NULL;

ALTER TABLE dataset_sources
    ADD COLUMN form_version_major integer;

ALTER TABLE reports
    ADD COLUMN form_version_major integer;

ALTER TABLE dataset_sources
    DROP CONSTRAINT dataset_sources_one_source_check;

ALTER TABLE dataset_sources
    ADD CONSTRAINT dataset_sources_one_source_check CHECK (
        (form_id IS NOT NULL AND compatibility_group_id IS NULL)
        OR (form_id IS NULL AND compatibility_group_id IS NOT NULL)
    ),
    ADD CONSTRAINT dataset_sources_form_version_major_check CHECK (
        form_version_major IS NULL OR (form_id IS NOT NULL AND form_version_major > 0)
    );

ALTER TABLE reports
    ADD CONSTRAINT reports_form_version_major_check CHECK (
        form_version_major IS NULL OR (form_id IS NOT NULL AND form_version_major > 0)
    );

UPDATE form_versions
SET
    version_major = (regexp_match(version_label, '^([0-9]+)\\.([0-9]+)\\.([0-9]+)$'))[1]::integer,
    version_minor = (regexp_match(version_label, '^([0-9]+)\\.([0-9]+)\\.([0-9]+)$'))[2]::integer,
    version_patch = (regexp_match(version_label, '^([0-9]+)\\.([0-9]+)\\.([0-9]+)$'))[3]::integer,
    started_new_major_line = (
        (regexp_match(version_label, '^([0-9]+)\\.([0-9]+)\\.([0-9]+)$'))[2]::integer = 0
        AND (regexp_match(version_label, '^([0-9]+)\\.([0-9]+)\\.([0-9]+)$'))[3]::integer = 0
    )
WHERE version_label ~ '^[0-9]+\\.[0-9]+\\.[0-9]+$'
  AND version_major IS NULL;

UPDATE dataset_sources
SET form_version_major = latest.version_major
FROM (
    SELECT DISTINCT ON (form_id)
        form_id,
        version_major
    FROM form_versions
    WHERE status = 'published'::form_version_status
      AND version_major IS NOT NULL
    ORDER BY
        form_id,
        version_major DESC,
        version_minor DESC NULLS LAST,
        version_patch DESC NULLS LAST,
        published_at DESC NULLS LAST,
        created_at DESC
) AS latest
WHERE dataset_sources.form_id = latest.form_id
  AND dataset_sources.form_id IS NOT NULL
  AND dataset_sources.form_version_major IS NULL;

UPDATE reports
SET form_version_major = latest.version_major
FROM (
    SELECT DISTINCT ON (form_id)
        form_id,
        version_major
    FROM form_versions
    WHERE status = 'published'::form_version_status
      AND version_major IS NOT NULL
    ORDER BY
        form_id,
        version_major DESC,
        version_minor DESC NULLS LAST,
        version_patch DESC NULLS LAST,
        published_at DESC NULLS LAST,
        created_at DESC
) AS latest
WHERE reports.form_id = latest.form_id
  AND reports.form_id IS NOT NULL
  AND reports.form_version_major IS NULL;
