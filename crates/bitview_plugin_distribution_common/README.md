# Distribution Common

Shared accounting, price-map state and metric helpers used by Age, Size and the
optional Entry plugin. This library owns no database, producer, checkpoint or
pipeline loop. Age and Entry each own their derived price-map instances and
recovery; Size owns its scalar checkpoint. Optional price-map operations are
compiled out for Size. Realized balances live only in shared accounting state.
