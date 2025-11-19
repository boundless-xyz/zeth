#!/usr/bin/env python3

import argparse, glob, json, os, re, subprocess, sys, urllib.request
from concurrent.futures import ThreadPoolExecutor

# Configuration
ETH_RPC_URL = os.environ.get("ETH_RPC_URL", "https://ethereum-rpc.publicnode.com")
CLI_BIN = "./target/release/cli"
CSV_FILE = "block-benchmarks.csv"


def build():
    print("Building for raw performance (tracing disabled)...")
    subprocess.check_call(["cargo", "build", "--release"])


def get_block(block_hash):
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_getBlockByHash",
        "params": [block_hash, False],
        "id": 1
    }
    req = urllib.request.Request(
        ETH_RPC_URL,
        data=json.dumps(payload).encode('utf-8'),
        headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req) as response:
        data = json.load(response)
        return data.get("result")


def parse_metrics(block_hash, output):
    # Helper to extract regex matches
    def get_val(pattern, default="N/A"):
        match = re.search(pattern, output)
        return match.group(1) if match else default

    # 1. Execution Time
    if time_match := re.search(r"execution time: ([0-9.]+)(ms|s)", output):
        val, unit = time_match.groups()
        exec_time = f"{float(val) / 1000:.6f}" if unit == "ms" else val
    else:
        exec_time = "N/A"

    # 2. Cycles & Counts
    metrics = {
        "total_cycles": get_val(r"(\d+) total cycles"),
        "user_cycles": get_val(r"(\d+) user cycles"),
        "paging_cycles": get_val(r"(\d+) paging cycles"),
        "bigint_cycles": get_val(r"BigInt calls, (\d+) cycles"),
        "keccak_calls": get_val(r"(\d+) Keccak calls"),
    }

    # 3. Gas & Block Number
    try:
        block_data = get_block(block_hash)
        block_number = int(block_data['number'], 16)
        gas_used = int(block_data['gasUsed'], 16)
    except Exception:
        block_number = "N/A"
        gas_used = "N/A"

    return [
        str(block_number), exec_time, metrics['total_cycles'], metrics['user_cycles'], metrics['paging_cycles'],
        metrics['bigint_cycles'], metrics['keccak_calls'], str(gas_used)
    ]


def run_benchmark(file_path):
    # cache/input_0x1234.json -> 0x1234
    block_hash = os.path.basename(file_path).replace("input_", "").replace(".json", "")
    print(f"Benchmarking block: {block_hash}")

    my_env = os.environ.copy()
    my_env["RUST_LOG"] = "info"
    my_env["RISC0_INFO"] = "true"
    my_env["RISC0_DEV_MODE"] = "true"

    cmd = [CLI_BIN, "--eth-rpc-url", ETH_RPC_URL, "--block", block_hash, "prove"]

    try:
        result = subprocess.run(cmd, env=my_env, capture_output=True, text=True, check=True)
        return parse_metrics(block_hash, result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error proving {block_hash}: {e.stderr}", file=sys.stderr)
        return None


def main():
    parser = argparse.ArgumentParser(description="Run Zeth benchmarks")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()

    build()

    files = glob.glob("cache/input_0x*.json")
    print(f"Benchmarking {len(files)} blocks with {args.jobs} jobs...")

    # Write Header
    with open(CSV_FILE, "w") as f:
        f.write(
            "block_number,execution_time,total_cycles,user_cycles,paging_cycles,bigint_cycles,keccak_calls,gas_used\n")

    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        for result in executor.map(run_benchmark, files):
            if result:
                with open(CSV_FILE, "a") as f:
                    f.write(",".join(result) + "\n")

    print(f"Done. Results saved to {CSV_FILE}")


if __name__ == "__main__":
    main()
