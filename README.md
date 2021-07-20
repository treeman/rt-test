Some notes:

# Errors

Serious errors are propagated to main, but faulty transactions (like withdrawal without enough funds) are silently ignored.

# Tests

There are a couple of input/output files in the `test_files` that exercises different edge cases. They are also run in the test suite with `cargo test`. See `main.rs` for the test implementation.

# Type system and robustness

The type system is used to separate the different transaction types, forcing us to handle the different cases. Deposit and withdrawals also require explicit handling.

I use Decimal from the `rust_decimal` crate to ensure precise calculations. But it does allow for negative numbers, which isn't an ideal representation for tracking client funds (an unsigned decimal type would be better). There is a check that we never go negative that is run after each transaction, and the atm bails if that ever happens.

# Assumptions

- I assume that withdrawals can be disputed.
- Maybe it would make sense to ignore transactions for a locked account, but it's not currently done.
- I assume the missing `locked` on the bottom of page 3 is an error.
