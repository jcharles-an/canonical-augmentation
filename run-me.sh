#!/bin/bash
#Run with this:
CASE=
DBPATH="/Users/jcharlesan/rust/shcreier-sims-app/parallel.db"

#replace slashes
DBURL="sqlite3://$( echo $DBPATH | sed 's:/:%2F:g')"
TABLE="line_1"
DBURLTABLE="${DBURL}/${TABLE}"
echo $DBURLTABLE

parallel --sqlandworker $DBURLTABLE  echo ::: foo bar ::: baz quux