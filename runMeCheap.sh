#!/bin/bash
trap "echo -e '\nScript interrupted.'; exit 1" SIGINT

N=$1;
K=$2;
O=$3; #0 is autorebalance 1 is bfs
M=$4; #0 is canon, 1 is unique, 2 least fast, 3 least full
if [[ $N == "" ]]; then N=7; fi
if [[ $K == "" ]]; then K=3; fi
if [[ $O == "" ]]; then O=0; fi
if [[ $M == "" ]]; then M=2; fi

names=("autorebalance_v2" "bfs_v1")
certs=("leastCanon" "unique" "leastFast" "leastFull")

export N_C=$N
export N_METHOD=$M
CASE="${N}_${K}"
RESULTS_DIR=results/$CASE
NAME_PREFIX=canaug_${names[$O]}_${certs[M]}
mkdir -p $RESULTS_DIR

for i in {0..10}
do
  name="${NAME_PREFIX}_$(date +%Y%m%d_%H%M%S)"
  echo "$name"
  # N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_test_inv.log
  #N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_orbits_impl.log
  # N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_choose_impl.log
  #/usr/bin/time -l cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_autorebalance_v2.log
  /usr/bin/time -l cargo run --release $K $O 2>&1 | tee $RESULTS_DIR/$name.log

  mv "$RESULTS_DIR/$name.log" "$RESULTS_DIR/full_$name.log"
done

rm -f $RESULTS_DIR/summary_$NAME_PREFIX.txt

echo "time:" >> $RESULTS_DIR/summary_$NAME_PREFIX.txt
cat $RESULTS_DIR/full_$NAME_PREFIX* | grep "User time" | awk '{print $4}' >> $RESULTS_DIR/summary_$NAME_PREFIX.txt

echo "wall time:" >> summary_$NAME_PREFIX.txt
cat $RESULTS_DIR/full_$NAME_PREFIX* | grep "wall clock"  | awk '{print $8}' >> $RESULTS_DIR/summary_$NAME_PREFIX.txt

echo "ram kb:" >> summary_$NAME_PREFIX.txt
cat $RESULTS_DIR/full_$NAME_PREFIX* | grep "aximum resident set size"  | awk -F':' '{print $2}' >> $RESULTS_DIR/summary_$NAME_PREFIX.txt




