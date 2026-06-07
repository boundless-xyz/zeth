#!/usr/bin/env python3

import argparse, csv, glob, gzip, json, os, re, subprocess, sys, tempfile
from array import array
from concurrent.futures import ThreadPoolExecutor
from collections import defaultdict
from pathlib import Path

_CACHE_RE = re.compile(r"^input_(0x[0-9a-fA-F]+)(?:\.v(\d))?$")

# Configuration
ETH_RPC_URL = os.environ.get("ETH_RPC_URL", "https://ethereum-rpc.publicnode.com")
CLI_BIN = "./target/release/cli"
CSV_FILE = "opcode-profile.csv"


def find_cached_blocks():
    best = {}  # block_hash -> version
    for f in glob.glob("cache/input_0x*.json"):
        m = _CACHE_RE.match(Path(f).stem)
        if not m:
            continue
        block_hash, ver = m.group(1), int(m.group(2) or 0)
        if ver > best.get(block_hash, -1):
            best[block_hash] = ver
    return list(best)


def build():
    print("Building with cycle-tracker enabled...")
    subprocess.check_call(
        ["cargo", "build", "--release", "--features", "cycle-tracker"]
    )


def run_trace(block_hash, output_dir):
    # Create a unique path inside the temporary directory
    trace_file = os.path.join(output_dir, f"trace_{block_hash}.json.gz")

    print(f"Tracing block: {block_hash}")

    my_env = os.environ.copy()
    my_env["RISC0_DEV_MODE"] = "true"
    my_env["TRACE_FILE"] = trace_file

    cmd = [CLI_BIN, "--eth-rpc-url", ETH_RPC_URL, "--block", block_hash, "prove"]

    try:
        subprocess.run(cmd, env=my_env, capture_output=True, text=True, check=True)
        return trace_file
    except subprocess.CalledProcessError as e:
        print(f"Error proving {block_hash}: {e.stderr}", file=sys.stderr)
        return None


def analyze_traces(trace_files, output_csv):
    print("Analyzing trace data...")

    cycle_data = defaultdict(lambda: array('Q'))
    gas_data = defaultdict(lambda: array('Q'))

    # Helper function defined ONCE, outside the loop
    def median_sorted(s):
        if not s: return 0
        n = len(s)
        mid = n // 2
        return (s[mid - 1] + s[mid]) // 2 if n % 2 == 0 else s[mid]

    for i, filename in enumerate(trace_files, 1):
        print(f"  Loading trace {i}/{len(trace_files)}: {os.path.basename(filename)}")
        with gzip.open(filename, "rb") as f:
            trace = json.load(f)

        for name, entries in trace.items():
            cycles = cycle_data[name]
            gas = gas_data[name]
            for c, g in entries:
                cycles.append(c)
                gas.append(g)

        # Free the large decoded JSON dict before loading the next file
        del trace

    with open(output_csv, "w") as f:
        writer = csv.writer(f)
        header = [
            "name",
            "count",
            "min cpg",
            "median cpg",
            "max cpg",
            "min cycles",
            "median cycles",
            "max cycles",
            "total cycles",
        ]
        writer.writerow(header)

        for name in sorted(cycle_data.keys()): # Sort keys for consistent CSV output
            cycle_arr = cycle_data[name]
            gas_arr = gas_data[name]

            # Calculate CPG (Cycles Per Gas)
            cpg_list = sorted(c // g for c, g in zip(cycle_arr, gas_arr) if g > 0)
            # Use the raw cycle array for cycle stats
            cycle_list = sorted(cycle_arr)

            if cpg_list:
                cpg_min, cpg_med, cpg_max = cpg_list[0], median_sorted(cpg_list), cpg_list[-1]
            else:
                cpg_min, cpg_med, cpg_max = "N/A", "N/A", "N/A"

            writer.writerow(
                [
                    name,
                    len(cycle_arr),
                    cpg_min,
                    cpg_med,
                    cpg_max,
                    cycle_list[0],
                    median_sorted(cycle_list),
                    cycle_list[-1],
                    sum(cycle_arr),
                ]
            )


def main():
    parser = argparse.ArgumentParser(description="Run Zeth cycle profiling")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()

    build()

    blocks = find_cached_blocks()
    print(f"Profiling {len(blocks)} blocks with {args.jobs} jobs...")

    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"Using temporary directory: {temp_dir}")

        generated_files = []
        with ThreadPoolExecutor(max_workers=args.jobs) as executor:
            for res in executor.map(lambda h: run_trace(h, temp_dir), blocks):
                if res:
                    generated_files.append(res)

        analyze_traces(generated_files, CSV_FILE)

    print(f"Done. Results saved to {CSV_FILE}")


if __name__ == "__main__":
    main()
