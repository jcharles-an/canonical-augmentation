#!/bin/bash

N=15;
K=4;
# N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_test_inv.log
#N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_orbits_impl.log
# N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_choose_impl.log
 N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_autorebalance.log