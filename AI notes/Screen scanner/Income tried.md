# Income tried

- **A top-level Snapshots page** (`/snapshots`, `SnapshotsView.vue`) listed a roster's snapshots with From / To radios (To could be now) and a character select, and showed the change between any two. Replaced by the character's Income page, which shows the change from the snapshot before only.
- **Breaking the fit at decreases automatically** (a snapshot with any decrease starting a new run) was planned and dropped before it was built: a different chest state, or a chest opened another way than the optimizer chose, shows as a decrease too. The first version had only ignoring, with one plain line; a spend then dragged the slope unless every snapshot on one side was ignored. Runs now start where the user marks them.
- **Storing the optimizer's chest states in a snapshot** was dropped: the states of the moment are of the old upgrades and budgets and not optimized for the snapshot. Each snapshot is optimized on its own instead, cached by payload hash.
