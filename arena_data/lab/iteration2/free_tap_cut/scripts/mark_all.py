# On the scratch copy only: marks every stored state for `rescore`.
import sqlite3, sys
db = sys.argv[1]
assert 'scratchpad' in db
con = sqlite3.connect(db)
con.execute('update run set ref_reference = null where error is null and state is not null')
con.commit()
print(con.execute('select count(*) from run where ref_reference is null').fetchone())
