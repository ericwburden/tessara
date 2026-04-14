# Implementation Spec

## Dataset Query Responsibilities
- compose base materialized sources and upstream DatasetRevisions
- apply row filters
- compute calculated fields
- expose a stable contract

## Component Responsibilities
- consume a DatasetRevision
- define grouping
- define measures
- apply bucketing
- render as table / chart / stat card

## Compatibility
When rebinding a Component draft to a newer DatasetRevision:
- findings classify as compatible / warning / blocking
- publication is blocked if blocking issues remain
- users may skip carry-forward instead of resolving every dependent artifact
