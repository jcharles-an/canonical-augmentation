#!/bin/bash
trap "echo -e '\nScript interrupted.'; exit 1" SIGINT

# -----------------------------------------------------------------------------
# Configuration Parameters
# -----------------------------------------------------------------------------
N=$1 # Number of points/lines in (n_k) configuration
K=$2 # Degree / points per line
O=$3 # Execution strategy: 0 = autorebalance, 1 = bfs
M=$4 # Invariant certificate method: 0 = leastCanon, 1 = unique, 2 = leastFast, 3 = leastFull
C=$5 # Invariant chain sequence: e.g. "0,1,2,3", "0,3", "2,3"
NUM_RUNS=$6

if [[ -z "$N" ]]; then N=7; fi
if [[ -z "$K" ]]; then K=3; fi
if [[ -z "$O" ]]; then O=0; fi
if [[ -z "$M" ]]; then M=2; fi
if [[ -z "$C" ]]; then C="0,1,2,3"; fi
if [[ -z "$NUM_RUNS" ]]; then NUM_RUNS=10; fi


names=("autorebalance_v2" "bfs_v1")
certs=("leastCanon" "unique" "leastFast" "leastFull")

# Format chain tag for filename (replace commas with dashes, e.g. chain_0-1-2-3)
CHAIN_TAG="chain_$(echo "$C" | tr ',' '-')"

export N_C=$N
export N_METHOD=$M
CASE="${N}_${K}"
RESULTS_DIR="results/$CASE"
NAME_PREFIX="canaug_${names[$O]}_${certs[$M]}_${CHAIN_TAG}"
mkdir -p "$RESULTS_DIR"

EXECUTABLE="./target/release/shcreier-sims-app"

# -----------------------------------------------------------------------------
# 1. Pre-compile binary ONCE so build time is never included in timings
# -----------------------------------------------------------------------------
echo "=========================================================================="
echo "Configuration: ${N}_${K} | Strategy: ${names[$O]} ($O) | Cert Method: ${certs[$M]} ($M)"
echo "=========================================================================="
echo "Pre-compiling release binary with N_C=$N and N_METHOD=$M..."
cargo build --release

if [[ ! -f "$EXECUTABLE" ]]; then
  echo "Error: Binary $EXECUTABLE was not found after build."
  exit 1
fi

# -----------------------------------------------------------------------------
# 2. Detect OS & Timing Utility (Linux GNU time vs macOS BSD time)
# -----------------------------------------------------------------------------
IS_LINUX=false
if [[ "$(uname -s)" == "Linux" ]]; then
  IS_LINUX=true
  TIME_BIN="/usr/bin/time"
  TIME_FLAGS="-v"
else
  TIME_BIN="/usr/bin/time"
  TIME_FLAGS="-l"
fi

# -----------------------------------------------------------------------------
# 3. Benchmark Execution Loop
# -----------------------------------------------------------------------------
SUMMARY_FILE="$RESULTS_DIR/summary_${NAME_PREFIX}.txt"
TIMESTAMP_BATCH=$(date +%Y%m%d_%H%M%S)

echo "Starting $NUM_RUNS benchmark runs..."
printf "%-5s | %-12s | %-12s | %-10s | %-12s | %-16s | %-10s\n" \
  "Run" "WallTime(s)" "UserTime(s)" "SysTime(s)" "PeakRAM(KB)" "InternalElapsed" "Leaves"
echo "----------------------------------------------------------------------------------------------"

# Temporary file to collect tabular data for this batch
BATCH_DATA_TMP="$RESULTS_DIR/.tmp_batch_${TIMESTAMP_BATCH}.tsv"
echo -e "run\twall_sec\tuser_sec\tsys_sec\trss_kb\tinternal_elapsed\tsolutions" > "$BATCH_DATA_TMP"

for ((i = 1; i <= NUM_RUNS; i++)); do
  RUN_NAME="${NAME_PREFIX}_${TIMESTAMP_BATCH}_run${i}"
  LOG_FILE="$RESULTS_DIR/full_${RUN_NAME}.log"

  # Execute and time the compiled binary directly
  $TIME_BIN $TIME_FLAGS $EXECUTABLE $K $O "$C" > "$LOG_FILE" 2>&1

  # Parse metrics based on OS
  if [[ "$IS_LINUX" == "true" ]]; then
    # GNU time -v parsing
    USER_SEC=$(grep "User time (seconds):" "$LOG_FILE" | awk -F': ' '{print $2}' | tr -d ' ')
    SYS_SEC=$(grep "System time (seconds):" "$LOG_FILE" | awk -F': ' '{print $2}' | tr -d ' ')
    RSS_KB=$(grep "Maximum resident set size (kbytes):" "$LOG_FILE" | awk -F': ' '{print $2}' | tr -d ' ')
    
    # Elapsed wall clock format in GNU time can be m:ss.ss or h:mm:ss
    WALL_RAW=$(grep "Elapsed (wall clock) time" "$LOG_FILE" | awk -F': ' '{print $2}' | tr -d ' ')
    WALL_SEC=$(echo "$WALL_RAW" | awk -F: '{ if (NF == 3) print ($1 * 3600) + ($2 * 60) + $3; else if (NF == 2) print ($1 * 60) + $2; else print $1 }')
  else
    # macOS BSD time -l fallback
    WALL_SEC=$(grep -E '[0-9\.]+ real' "$LOG_FILE" | awk '{print $1}')
    USER_SEC=$(grep -E '[0-9\.]+ real' "$LOG_FILE" | awk '{print $3}')
    SYS_SEC=$(grep -E '[0-9\.]+ real' "$LOG_FILE" | awk '{print $5}')
    RSS_BYTES=$(grep "maximum resident set size" "$LOG_FILE" | awk '{print $1}')
    RSS_KB=$(( RSS_BYTES / 1024 ))
  fi

  # Internal solver metrics
  INTERNAL_TIME=$(grep -E "Elapsed:" "$LOG_FILE" | tail -n 1 | awk -F'Elapsed:' '{print $2}' | tr -d ' ')
  SOLUTIONS=$(grep -E "effective leaves:|All Solutions:" "$LOG_FILE" | tail -n 1 | awk -F':' '{print $NF}' | tr -d ' ')

  # Fallback defaults if empty
  [[ -z "$USER_SEC" ]] && USER_SEC="N/A"
  [[ -z "$SYS_SEC" ]] && SYS_SEC="N/A"
  [[ -z "$WALL_SEC" ]] && WALL_SEC="N/A"
  [[ -z "$RSS_KB" ]] && RSS_KB="N/A"
  [[ -z "$INTERNAL_TIME" ]] && INTERNAL_TIME="N/A"
  [[ -z "$SOLUTIONS" ]] && SOLUTIONS="N/A"

  printf "%-5s | %-12s | %-12s | %-10s | %-12s | %-16s | %-10s\n" \
    "$i" "$WALL_SEC" "$USER_SEC" "$SYS_SEC" "$RSS_KB" "$INTERNAL_TIME" "$SOLUTIONS"

  echo -e "$i\t$WALL_SEC\t$USER_SEC\t$SYS_SEC\t$RSS_KB\t$INTERNAL_TIME\t$SOLUTIONS" >> "$BATCH_DATA_TMP"
done

# -----------------------------------------------------------------------------
# 4. Generate Structured Summary File
# -----------------------------------------------------------------------------
{
  echo "=========================================================================="
  echo "Summary Benchmark Report: $NAME_PREFIX"
  echo "Configuration: N=$N, K=$K | Strategy=$O (${names[$O]}) | Method=$M (${certs[$M]})"
  echo "Date: $(date)"
  echo "=========================================================================="
  echo ""
  echo "Raw Data per Run:"
  column -t -s $'\t' "$BATCH_DATA_TMP"
  echo ""
  echo "--------------------------------------------------------------------------"
  echo "Averages (across $NUM_RUNS runs):"
  awk 'NR > 1 && $2 != "N/A" { sum_wall += $2; sum_user += $3; sum_sys += $4; sum_rss += $5; count++ }
       END {
         if (count > 0) {
           printf "Average Wall Time : %.4f s\n", (sum_wall / count);
           printf "Average User Time : %.4f s\n", (sum_user / count);
           printf "Average Sys Time  : %.4f s\n", (sum_sys / count);
           printf "Average Peak RSS  : %.2f KB (%.2f MB)\n", (sum_rss / count), (sum_rss / count / 1024);
           printf "CPU Utilization   : %.2f %%\n", ((sum_user + sum_sys) / sum_wall * 100);
         }
       }' "$BATCH_DATA_TMP"
  echo "=========================================================================="
} > "$SUMMARY_FILE"

rm -f "$BATCH_DATA_TMP"

echo "----------------------------------------------------------------------------------------------"
echo "Benchmark completed successfully."
echo "Summary written to: $SUMMARY_FILE"





