# bitview_compute

Stateful and range-based calculations shared by Bitview plugins: rolling
statistics, drawdowns, weighted cohort state, age-band math and resume
preparation. The `statistics` family contains rolling algorithms and the
order-statistics structures they use.

Stateless value transforms live in `bitview_transforms`; result shapes live in
`bitview_collections`; metric vector ownership and view composition live in
`bitview_vecs`. Storage primitives, shared cache accounting, and stored-version
validation belong to vecdb.

`ComputedVecValue`, `NumericValue`, and `FixedRatio` describe calculation/storage
requirements; `Quantity` names a unit's mean (`Fract`) and total (`Sum`) types.
Scalar transforms do not depend on these broader bounds.
