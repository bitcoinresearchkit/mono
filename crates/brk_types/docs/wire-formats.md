# Shared wire-format invariants

Block hashes use one fixed-size lowercase hex encoder for display, serialization
and vector JSON. Bitcoin's conventional byte order is preserved. Tests compare
native Bitcoin formatting, flags, JSON round trips and appended vector output.
