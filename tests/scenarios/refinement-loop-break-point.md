# Scenario: refinement-loop-break-point

`rust-refinement-loop` is running on a feature branch that has a spec. It
has run five cycles in this batch. Cycle 5's stage 1 applied nothing and
added nothing new, its stage 2 confirmed one bug, whose fix and test were
committed, and its stage 3 applied one cleanup. Two trade-offs are pending
on the list. What happens next?
