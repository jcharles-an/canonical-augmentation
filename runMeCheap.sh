#!/bin/bash

N=13;
K=3;
# N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_test_inv.log
#N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_orbits_impl.log
 N_C=$N cargo run --release $K 2>&1 | tee results/canaug_${N}_${K}_choose_impl.log