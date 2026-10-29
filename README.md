# Canonical Augmentation Solver (`shcreier-sims-app`)

A $(n_k)$ combinatorial configurations solution approach via canonical augmentation, Schreier-Sims stabilizer chains, and customizable graph invariant chains.

## Prerequisites

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **C Toolchain**: Required by `nauty-Traces-sys` to compile the underlying `nauty` C library during `cargo build` (e.g., `xcode-select --install` on macOS or `build-essential` on Linux).
3. **Standard Bash Utilities**: `bash`, `awk`, `column`, and `/usr/bin/time` (used by the benchmark wrapper scripts).

---

## Structure and Execution 

The project is structured into three layers:

### 1. Core App (`src/main.rs`)
`src/main.rs` is the main driver for the Rust application.

- **Compile-time parameters** (`build.rs` & `src/data_structs/consts.rs`):
  - `N_C`: The configuration parameter $n$ (number of points/lines). Because the [`bitmaps`](https://crates.io/crates/bitmaps) crate requires compile-time constants (`Bitmap<N>`, `Bitmap<N_CONF>`, `Bitmap<N_CERT>`) to generate stack-allocated, size-tailored bitmaps, changing $n$ requires recompiling the binary.
  - `N_METHOD`: Canonical certificate method (`0` = `leastCanon`, `1` = `unique`, `2` = `leastFast`, `3` = `leastFull`).
- **Runtime CLI arguments**:
  ```bash
  ./target/release/shcreier-sims-app <K> <OPTION> <INVARIANT_CHAIN>
  ```
  - `K`: Degree $k$ (points per line / lines per point, e.g., `3` or `4`).
  - `OPTION`: Execution strategy (`0` = `autorebalance_v2` parallel work-stealing, `1` = `bfs_v1` breadth-first search).
  - `INVARIANT_CHAIN`: Comma-separated list of invariant links (`0` = Connected Components, `1` = Degeneracy, `2` = Degree Sequence, `3` = Nauty Canonical Certificate; e.g., `"0,1,2,3"` or `"2,3"`).

### 2. Single-Experiment Wrapper (`runMeCheap.sh`)
`runMeCheap.sh` wraps Rust binary to handle compile-time variables and timing.

- Exports `N_C` and `N_METHOD` and runs `cargo build --release` before timing begins and the `bitmaps` crate creates the exact bitmaps needed for $n$.
- Executes the compiled binary `NUM_RUNS` times running `/usr/bin/time`, capturing wall time, CPU, peak RSS memory, and solution number.
- Saves individual run logs (`full_*.log`) and an averaged summary report (`summary_*.txt`) in `results/${N}_${K}/`.

**Usage:**
```bash
./runMeCheap.sh <N> <K> <STRATEGY> <METHOD> <CHAIN> <NUM_RUNS>
```
**Example** (run $(10_3)$ with `autorebalance_v2`, `leastFast` method, chain `2,3`, for 5 runs):
```bash
./runMeCheap.sh 10 3 0 2 "2,3" 5
```

### 3. Full Experiment Wrapper (`run_case_7_3.sh`)
`run_case_7_3.sh` is the top level script for running full experiments. 

Note: legacy filename, not hardcoded to $(7_3)$—the `N`, `K`, `M`, `NUM_RUNS`, `CHAINS`, and `STRATEGIES` parameters can be edited directly at the top of the script.

- Iterates over all strategies (`STRATEGIES`) and invariant chain combinations (`CHAINS`).
- Invokes `runMeCheap.sh` for each combination.
- Parses the summary files and aggregates metrics into a single master comparison table in `results/${N}_${K}/MASTER_COMPARISON_${N}_${K}_LEASTFAST.txt`.

**Usage:**
1. Edit `N`, `K`, `NUM_RUNS`, `CHAINS`, or `STRATEGIES` at the top of `run_case_7_3.sh` as needed.
2. Run the suite:
   ```bash
   ./run_case_7_3.sh
   ```
