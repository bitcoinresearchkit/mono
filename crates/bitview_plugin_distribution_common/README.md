# Distribution Common

Shared accounting primitives and metric helpers used by Age and Size.
This is a library, not a scheduled plugin. It owns no database, producer,
checkpoint directory or pipeline loop. Age owns its advanced price maps;
Size owns its scalar checkpoint vector. Optional price-map operations are
compiled out for Size; realized balances live only in shared accounting state.
