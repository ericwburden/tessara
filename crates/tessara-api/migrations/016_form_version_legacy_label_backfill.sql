UPDATE form_versions
SET
    version_major = (regexp_match(version_label, '([0-9]+)$'))[1]::integer,
    version_minor = 0,
    version_patch = 0,
    started_new_major_line = true
WHERE version_major IS NULL
  AND version_label ~ '([0-9]+)$';
