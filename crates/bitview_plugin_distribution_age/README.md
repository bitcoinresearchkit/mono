# Distribution Age

Consumes published UTXO history plus block prices and monotonic timestamps.
It owns age, creation-year, epoch, and term cohorts, their advanced
cost-basis maps and age analytics. Those maps are in-memory derived state;
restart reconstructs them from the canonical history snapshot and diff suffix.
Append updates retain validated state. Reorgs and failed updates rebuild it.

It owns no address state and stores no URPD files. Block-based URPD models and
queries reconstruct rounded price distributions through `bitview_urpd`.

Entry-price classification belongs to the optional `bitview_plugin_distribution_entry`
plugin. Creation-year and epoch groups are fixed by birth timestamp and height;
they require no external classification state.
