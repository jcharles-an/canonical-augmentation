#!/bin/bash

N=15;
K=4;
N_C=$N cargo run --release $K 2>&1 | tee results/run_${N}_${K}_cheap.log