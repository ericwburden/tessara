-- Every capability declared by an accepted independent-module manifest must
-- be available to Core's role editor. This backfills releases accepted before
-- capability registration became part of deployment-receipt application.

INSERT INTO capabilities (key, description, scope_mode)
SELECT DISTINCT ON (declaration ->> 'id')
    declaration ->> 'id',
    declaration ->> 'description',
    'scope_aware'
FROM module_releases
CROSS JOIN LATERAL jsonb_array_elements(
    COALESCE(manifest -> 'security_capabilities', '[]'::jsonb)
) AS declaration
WHERE manifest IS NOT NULL
  AND btrim(declaration ->> 'id') <> ''
  AND btrim(declaration ->> 'description') <> ''
ORDER BY declaration ->> 'id', module_releases.created_at DESC
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    scope_mode = EXCLUDED.scope_mode;
