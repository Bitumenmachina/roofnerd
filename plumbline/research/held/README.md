# Work built apart and not yet in the tree

A patch here was built by workers in second working copies while another worker had the
main tree, read and run by the lead, and is waiting to be applied (`git apply` from the
repository root), run on the whole workspace and committed. It is kept in the repository
only so a restart of the machine cannot lose it; it is removed in the commit that applies
it. `STATUS.md` says what each piece is and why it is held.

- `calls-164-165-168-170.patch`: the three pieces put together by the lead (one line of
  the command line's usage text and two lists were merged by hand): a wall moved and the
  walls that meet it follow (call 170); a plan set on paper, with its pictures (call 168);
  a beam, its posts and their pad footings, joists and rafters counted (calls 164, 165).
  Together, on every crate but the window's: 543 tests pass, and `stretch.sh`,
  `planset.sh`, `structure.sh`, `joists.sh`, `hidden.sh`, `kitchen.sh`, `partwork.sh` and
  `library.sh` pass.
