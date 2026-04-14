# Tessara Roadmap Addendum: Dataset / Report / Aggregation Layering

---

## Purpose

This addendum refines the data modeling and reporting architecture for Tessara by introducing a **Dataset layer** and clarifying responsibilities across:

- Forms
- Datasets
- Reports
- Aggregations

It is derived directly from limitations observed in the previous system, where reports implicitly acted as both data modeling and query layers.

---

# 1. Legacy Model (Pre-Refactor)

## Structure

- **Form**
  - Defines fields and captures data

- **Report**
  - Flattens form data
  - Selects fields
  - Applies filters

- **Aggregation**
  - Groups and summarizes report data

## Key Limitation

Reports were responsible for:
- Data flattening
- Schema definition
- Filtering
- (implicitly) joining or combining form data

This created tight coupling between:
- data structure
- query logic
- reporting logic

### Resulting Problems

- Difficult to combine multiple forms in one report
- Hard to reuse report definitions
- Complex and brittle logic inside reports
- Limited flexibility for cross-form analytics

---

# 2. New Model (Post-Refactor)

## Structure

```
Forms → Dataset → Report → Aggregation → Chart → Dashboard
```

---

# 3. Layer Responsibilities

## 3.1 Forms (Unchanged Role)

### Responsibility
- Define data capture structure
- Validate submissions
- Store raw values

### Notes
- Forms no longer participate directly in reporting logic
- Forms feed into datasets

---

## 3.2 Dataset (New Layer)

### Responsibility
Defines the **semantic data model** used for reporting.

### Includes

- Source forms (one or more)
- Compatibility groups
- Join or union logic
- Row grain (required)
- Field mapping
- Record selection rules (latest, earliest, etc.)
- Deduplication rules

### Examples

- “All intake submissions across versions”
- “Client case summary (intake + assessment + discharge)”
- “Program participation dataset”

### Concerns moved from Reports → Dataset

| Concern | Previous Location | New Location |
|--------|------------------|--------------|
| Flattening form data | Report | Dataset |
| Combining multiple forms | Report | Dataset |
| Selecting latest/earliest records | Report (implicit) | Dataset |
| Defining row grain | Implicit | Dataset (explicit) |
| Compatibility across versions | Partial | Dataset |

---

## 3.3 Report (Refined Role)

### Responsibility
Defines a **row-level query** over a dataset.

### Includes

- Field selection
- Filtering
- Ordering
- Computed fields (NEW)

### Does NOT include

- Grouping
- Aggregation
- Row collapsing
- Dataset structure changes

---

### Computed Fields (New Capability)

Computed fields are expressions evaluated per row.

#### Use Cases

- Derived values (e.g., `age = today - birth_date`)
- Labels (e.g., `full_name`)
- Flags (e.g., `is_overdue`)
- Normalization (e.g., `score_band`)

#### Constraints

- Must be row-level
- Cannot reference other rows
- Cannot perform aggregation

---

### Concerns moved from Dataset → Report

| Concern | Reason |
|--------|-------|
| Display-friendly transformations | Report is presentation layer |
| Row-level derived logic | Belongs with query logic |
| Filtering | Context-specific, not structural |

---

## 3.4 Aggregation (Unchanged but Clarified)

### Responsibility
Defines **grouping and summarization** over report results.

### Includes

- Group-by fields
- Metrics (count, sum, avg, etc.)

### Does NOT include

- Filtering (comes from Report)
- Data modeling (comes from Dataset)

---

### Clarification

Grouping exists **only here**.

This avoids ambiguity and keeps:
- Reports reusable
- Logic centralized
- UI simpler

---

# 4. Summary of Responsibility Shifts

## What moved OUT of Reports

- Multi-form composition → Dataset
- Compatibility logic → Dataset
- Row grain definition → Dataset
- Latest/earliest selection → Dataset

## What stayed in Reports

- Field selection
- Filtering
- Ordering

## What was added to Reports

- Computed fields (row-level expressions)

---

# 5. Benefits of New Model

## 5.1 Clear Separation of Concerns

| Layer | Responsibility |
|------|---------------|
| Dataset | What the data *is* |
| Report | Which rows/fields you want |
| Aggregation | How to summarize it |

---

## 5.2 Multi-Form Reporting Enabled

Datasets can:
- join forms
- union compatible forms
- expose unified schema

Reports no longer need to handle this complexity.

---

## 5.3 Reusability

- One dataset → many reports
- One report → many aggregations

---

## 5.4 Alignment with Analytical Systems

Conceptual mapping:

| SQL Concept | Tessara Layer |
|------------|--------------|
| FROM / JOIN | Dataset |
| SELECT / WHERE | Report |
| GROUP BY | Aggregation |

---

# 6. New Concepts Introduced

## Dataset
A reusable, composable data model.

## Dataset Source
A form or compatibility group contributing data.

## Dataset Join / Union
Defines how multiple sources combine.

## Report Computed Field
Row-level derived expression.

---

# 7. Example

## Use Case
Client summary across multiple forms.

## Dataset
- Grain: one row per client
- Sources:
  - Intake form
  - Assessment form
  - Discharge form
- Join: client node id
- Selection:
  - Intake → earliest
  - Assessment → latest
  - Discharge → latest

## Report
- Fields:
  - client_id
  - intake_date
  - latest_score
  - discharge_reason
- Filter:
  - program = X
- Computed:
  - risk_band

## Aggregation
- Group by: program
- Metrics:
  - avg score
  - count clients

---

# 8. Implementation Notes

## Required for v1

- Explicit dataset grain
- Explicit join keys
- Latest/earliest selection rules
- Basic computed field expressions

## Defer

- Fuzzy joins
- Complex window functions
- Cross-dataset joins

---

# 9. Final Model

```
Forms → Dataset → Report → Aggregation → Visualization
```

---

## Final Principle

> Forms capture data.  
> Datasets define meaning.  
> Reports select and shape rows.  
> Aggregations summarize.

This separation resolves the original limitations and enables scalable, flexible reporting.
