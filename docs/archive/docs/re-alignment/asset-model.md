# Asset Model

## Dataset
Reusable row-level analytical asset.

### Owns
- source composition
- joins / unions
- latest / earliest reducers
- row grain
- row filters
- calculated fields
- exposed field contract

### Internal structure
- mutable logical Dataset
- immutable DatasetRevision
- materialized relation for performance

## Component
Versioned presentation asset over a DatasetRevision.

### v1 types
- DetailTable
- AggregateTable
- Bar
- Line
- Pie/Donut
- StatCard

### Owns
- grouping
- measures
- bucketing
- presentation configuration

## Dashboard
Mutable composition asset that references specific ComponentVersions.
Not versioned in v1.

## Future Printable Report
Separate artifact that can compose prose and ComponentVersions.
