#!/bin/bash
trap "echo -e '\nSuite interrupted.'; exit 1" SIGINT

# =============================================================================
# Automated Benchmark Suite: Case 7_3 (Fano Plane) - leastFast Invariant Chains
# =============================================================================
# This script executes an exhaustive comparison matrix of all invariant chain
# permutations and subsets under the `leastFast` (M=2) canonical method for N=7, K=3:
#
# Available Invariant Links:
#   0 = Connected Components (O(V+E) bitwise BFS)
#   1 = Graph Degeneracy (k-core ordering)
#   2 = Degree Sequence
#   3 = Nauty Canonical Certificate (terminal tie-breaker)
#
# Tested Chain Combinations:
#   - Baseline (Pure Nauty): "3"
#   - Single-link chains: "0,3", "1,3", "2,3"
#   - Dual-link chains: "0,1,3", "0,2,3", "1,0,3", "1,2,3", "2,0,3", "2,1,3"
#   - Full 3-link permutations: "0,1,2,3", "0,2,1,3", "1,0,2,3", "1,2,0,3", "2,0,1,3", "2,1,0,3"
#
# Tested across both execution strategies:
#   - O=0: Auto-balanced parallel (autorebalance_v2)
#   - O=1: Breadth-First Search (bfs_v1)
# =============================================================================

N=13
K=3
M=2 # leastFast method
NUM_RUNS=10
RESULTS_DIR="results/${N}_${K}"
mkdir -p "$RESULTS_DIR"

MASTER_REPORT="$RESULTS_DIR/MASTER_COMPARISON_${N}_${K}_LEASTFAST.txt"
TMP_MASTER="$RESULTS_DIR/.tmp_master_${N}_${K}.tsv"

echo "=========================================================================="
echo " Starting Full leastFast Benchmark Suite for Case ${N}_${K}"
echo " Date: $(date)"
echo " Runs per Experiment: $NUM_RUNS"
echo " Results Directory: $RESULTS_DIR"
echo "=========================================================================="
echo ""

echo -e "strategy\tchain\truns\tavg_wall_sec\tavg_user_sec\tavg_sys_sec\tavg_rss_mb\tcpu_util_pct\tsolutions" > "$TMP_MASTER"

# All leastFast chain configurations to evaluate
CHAINS=(
  # Baseline Pure Nauty (zero invariants)
  "3 Pure_Nauty"
  
  # Single Invariant -> Nauty
  "0,3 Comp_Nauty"
  "1,3 Degen_Nauty"
  "2,3 DegSeq_Nauty"

  # Dual Invariants -> Nauty
  "0,1,3 Comp_Degen_Nauty"
  "0,2,3 Comp_DegSeq_Nauty"
  "1,0,3 Degen_Comp_Nauty"
  "1,2,3 Degen_DegSeq_Nauty"
  "2,0,3 DegSeq_Comp_Nauty"
  "2,1,3 DegSeq_Degen_Nauty"

  # Full 3-link Permutations -> Nauty
  "0,1,2,3 Full_Comp_Degen_DegSeq_Nauty"
  "0,2,1,3 Full_Comp_DegSeq_Degen_Nauty"
  "1,0,2,3 Full_Degen_Comp_DegSeq_Nauty"
  "1,2,0,3 Full_Degen_DegSeq_Comp_Nauty"
  "2,0,1,3 Full_DegSeq_Comp_Degen_Nauty"
  "2,1,0,3 Full_DegSeq_Degen_Comp_Nauty"
)

#CHAINS=("2,3 DegSeq_Nauty")

STRATEGIES=(
  "0 autorebalance_v2"
  "1 bfs_v1"
)

# STRATEGIES=("0 autorebalance_v2")

TOTAL_EXP=$(( ${#STRATEGIES[@]} * ${#CHAINS[@]} ))
EXP_NUM=0

for strat_entry in "${STRATEGIES[@]}"; do
  read -r STRAT STRAT_NAME <<< "$strat_entry"

  for chain_entry in "${CHAINS[@]}"; do
    read -r CH_SEQ CH_DESC <<< "$chain_entry"
    EXP_NUM=$((EXP_NUM + 1))

    echo ""
    echo ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>"
    echo " [Experiment $EXP_NUM / $TOTAL_EXP] Strategy: $STRAT_NAME | Chain: $CH_SEQ ($CH_DESC) | Runs: $NUM_RUNS"
    echo ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>"

    # Execute runMeCheap.sh with N, K, STRATEGY, METHOD=2 (leastFast), custom CHAIN, and NUM_RUNS
    ./runMeCheap.sh $N $K $STRAT $M "$CH_SEQ" $NUM_RUNS

    CHAIN_TAG="chain_$(echo "$CH_SEQ" | tr ',' '-')"
    SUM_FILE="$RESULTS_DIR/summary_canaug_${STRAT_NAME}_leastFast_${CHAIN_TAG}.txt"

    if [[ -f "$SUM_FILE" ]]; then
      AVG_WALL=$(grep "Average Wall Time" "$SUM_FILE" | awk '{print $5}')
      AVG_USER=$(grep "Average User Time" "$SUM_FILE" | awk '{print $5}')
      AVG_SYS=$(grep "Average Sys Time" "$SUM_FILE" | awk '{print $5}')
      AVG_RSS=$(grep "Average Peak RSS" "$SUM_FILE" | awk -F'(' '{print $2}' | awk '{print $1}')
      CPU_UTIL=$(grep "CPU Utilization" "$SUM_FILE" | awk '{print $4}')
      SOLS=$(grep -E "^\s*1\s+" "$SUM_FILE" | head -n 1 | awk '{print $NF}')

      [[ -z "$AVG_WALL" ]] && AVG_WALL="N/A"
      [[ -z "$AVG_USER" ]] && AVG_USER="N/A"
      [[ -z "$AVG_SYS" ]] && AVG_SYS="N/A"
      [[ -z "$AVG_RSS" ]] && AVG_RSS="N/A"
      [[ -z "$CPU_UTIL" ]] && CPU_UTIL="N/A"
      [[ -z "$SOLS" ]] && SOLS="N/A"

      echo -e "${STRAT_NAME}\t$CH_SEQ\t$NUM_RUNS\t$AVG_WALL\t$AVG_USER\t$AVG_SYS\t$AVG_RSS\t$CPU_UTIL\t$SOLS" >> "$TMP_MASTER"
    else
      echo "Warning: Summary file $SUM_FILE not found."
    fi
  done
done

# =============================================================================
# Build Master Summary Report Table
# =============================================================================
{
  echo "===================================================================================================="
  echo " MASTER COMPARISON REPORT: CASE ${N}_${K} (leastFast Invariant Chain Ablation)"
  echo " Generated: $(date)"
  echo " Configuration: N=$N, K=$K (Levi Graph Vertices: $((N * 2)))"
  echo " Method: leastFast (M=2)"
  echo " Repetitions Averaged per Configuration: $NUM_RUNS"
  echo "===================================================================================================="
  echo ""
  echo "Summary Comparison Matrix:"
  echo ""
  column -t -s $'\t' "$TMP_MASTER"
  echo ""
  echo "===================================================================================================="
  echo " Link Reference:"
  echo "   0 = Connected Components Count (bitwise BFS)"
  echo "   1 = Graph Degeneracy (k-core ordering)"
  echo "   2 = Degree Sequence"
  echo "   3 = Nauty Canonical Certificate (terminal tie-breaker)"
  echo "===================================================================================================="
} > "$MASTER_REPORT"

rm -f "$TMP_MASTER"

echo ""
echo "=========================================================================="
echo " All leastFast ${N}_${K} Benchmark Experiments Completed!"
echo " Master report written to: $MASTER_REPORT"
echo "=========================================================================="
cat "$MASTER_REPORT"

