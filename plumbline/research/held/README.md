# Work built apart and not yet in the tree

A patch here was built by a worker in a second working copy while another worker had the
main tree, read and run by the lead, and is waiting to be applied (`git apply` from the
repository root), run on the whole workspace and committed. It is kept in the repository
only so a restart of the machine cannot lose it; it is removed in the commit that applies
it. `STATUS.md` says what each is and why it is held.

- `call170.patch`: a wall moved, and the walls that meet it follow (DECISIONS.md call 170).
- `call168.patch`: a plan set on paper (DECISIONS.md call 168). It carries its own pictures
  (`demos/captured/planset/`) as binary pieces of the patch.
