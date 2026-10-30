# The lab's database, read-only, and a smoke test's scratch database attached when given as 's'.
import sqlite3
RUNS = 'arena_data/lab/runs.sqlite'
def connect(scratch=None):
    c = sqlite3.connect(f'file:{RUNS}?mode=ro', uri=True)
    if scratch:
        c.execute(f"attach 'file:{scratch}?mode=ro' as s")
    return c, ('main', 's') if scratch else ('main',)
