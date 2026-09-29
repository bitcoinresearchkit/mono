# Distribution Age

Consumes published UTXO history plus block prices and monotonic timestamps.
It owns age, creation-year, epoch, term and entry-price cohorts, their advanced
cost-basis maps and age analytics. Those maps are in-memory derived state;
restart reconstructs them from the canonical history snapshot and diff suffix.
Append updates retain validated state. Reorgs and failed updates rebuild it.

It owns no address state and stores no URPD files. Block-based URPD models and
queries reconstruct rounded price distributions through `bitview_urpd`.
