#!/bin/bash
#Run with this:
N=$1
K=$2
CASE=${N}_${K}
SCHEMA="conf_${CASE}"
DBPATH="localhost"
USER="jcharlesan"
PASS="190128"
#replace slashes
# postgresql://scott:tiger@pg.example.com/mydatabase/mytable
DBURL="postgresql://${USER}:${PASS}@${DBPATH}/n_k/${SCHEMA}"


N_C=$N cargo build --release

function process_lines(){
  local executable="./target/release/shcreier-sims-app"
  TABLE="line_${LINE}"
  DBURLTABLE="${DBURL}.${TABLE}"

  set_up_connection $SCHEMA

  previous_line="null"
  for ((LINE=0; LINE < $N; LINE++)); do
    TABLE="line_${LINE}"
    DBURLTABLE="${DBURL}/${TABLE}"

    create_table $TABLE
    get_graph ${previous_line} | parallel --sqlandworker +$DBURLTABLE -j 20 "$executable $K $LINE"

    wait_for_parallel $TABLE

    create_results_table $LINE
    previous_line=$LINE
  done
}

function get_empty_graph(){
  result=""
  for ((i=1; i<=2*N; i++)); do
      if ((i > 1)); then
          result+=":"
      fi
      result+="0"
  done

  echo "$result"
}

function query(){
  local q=$1
  psql -U $USER -d "n_k" -c "$q"
}

function set_up_connection(){
  local schema=$1
  query "create schema IF NOT EXISTS ${schema};"
}

function wait_for_parallel(){
  local finished=0
  local TABLE=$1
  while [ $finished -eq "0" ]; do
    val=$(query "SELECT seq FROM ${TABLE} WHERE Exitval = -1000 LIMIT 1;");
    if [ -z $val ]; then
      finished=1
      echo "$TABLE finished!"
    fi;
    sleep 0.5
  done;
}

function create_table(){
  local TABLE=$1
  #echo "Creating table $TABLE"
  if [ "$(query "SELECT count(*) FROM sqlite_schema WHERE name='${TABLE}'")" -eq "0" ]; then
    parallel --sqlmaster $DBURLTABLE echo ::: "starting"
    query "create unique index ${TABLE}_V1_uindex on ${TABLE} (V1);"
  fi;

}

function get_graph(){
  local line=$1
  if [ "$line" = "null" ]; then
    get_empty_graph
  else
    query "SELECT graph FROM line_${line}_results;"
  fi;
}

function create_results_table(){
  local LINE=$1
  local results_name="line_${LINE}_results"
  #echo "Creating $results_name"
  query "create table if not exists ${results_name}
                   (
                       cert  TEXT
                           constraint ${results_name}_pk
                               primary key,
                       graph TEXT
                   );"
  query "DELETE FROM line_${LINE} WHERE V1 = 'starting';"
  query "SELECT Stdout FROM line_${LINE};" | while IFS= read -r current_line; do
    if [ -z "$current_line" ]; then continue; fi
    #echo "Current_line is: ${current_line}"
    read -r cert graph <<< "$current_line"
    query "INSERT INTO ${results_name}(cert, graph) VALUES ('${cert}','${graph}')"
  done
}

function reset_DB() {
  rm $SCHEMA

}

  executable="./target/release/shcreier-sims-app"
  TABLE="line_${LINE}"
  DBURLTABLE="${DBURL}.${TABLE}"
  set_up_connection $SCHEMA
  seq 10 | parallel --sqlmaster $DBURLTABLE "echo"

  echo "Sleeping"
  sleep 60

  echo "Slept"
  parallel --sqlworker $DBURLTABLE "echo"

#reset_DB
#process_lines