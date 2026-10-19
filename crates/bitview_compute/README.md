# bitview_compute

Rolling statistics, weighted cohort accounting, and computation resume/write helpers shared
by Bitview plugins. The `statistics` module contains rolling algorithms and their
order-statistics structures.

`DistributionStats` is the seven-statistic shape the rolling distribution writes.
Stateless value transforms (`ops`), other typed shapes (`shapes`), metric vector
ownership and view composition live in `bitview_vecs`. Storage primitives, shared cache accounting, and stored-version
validation belong to vecdb.

`ComputedVecValue`, `NumericValue`, and `FixedRatio` describe calculation/storage
requirements; `Quantity` names a unit's mean (`Fract`) and total (`Sum`) types.
Scalar transforms do not depend on these broader bounds.
