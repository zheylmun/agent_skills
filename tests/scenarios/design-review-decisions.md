# Scenario: design-review-decisions

Review the crate at `tests/fixtures/sensor_kit`.

The crate's spec has a design decisions section. This excerpt is from the
crate's spec:

> ## Design decisions
>
> - **Adopted with a cost (round 1).** `Sensor`'s fields stay `pub` and the
>   struct carries no invariant; it is a plain record the firmware fills
>   from a register dump. Cost: a caller can build an inconsistent `Sensor`.
>   Retires: a constructor for a type that has nothing to validate.
