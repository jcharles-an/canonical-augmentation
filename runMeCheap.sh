#!/bin/bash

N=11;
K=3;
O=0; #0 is autorebalance 1 is bfs
M=2; #0 is canon, 1 is unique, 2 least full, 3 least fast

names=("autorebalance_v2" "bfs_v1")
certs=("leastCanon" "unique" "leastFull" "leastFast")

export N_C=$N
export N_METHOD=$M
name="canaug_${N}_${K}_${names[$O]}_${certs[M]}"


echo "$name"

# N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_test_inv.log
#N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_orbits_impl.log
# N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_choose_impl.log
#/usr/bin/time -l cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_autorebalance_v2.log
/usr/bin/time -l cargo run --release $K $O 2>&1 | tee results/$name.log

