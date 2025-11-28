import argparse, csv, glob, gzip, json, os, statistics, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor
from collections import defaultdict

# Configuration
ETH_RPC_URL = os.environ.get("ETH_RPC_URL", "https://ethereum-rpc.publicnode.com")
CLI_BIN = "./target/release/cli"
CSV_FILE = "opcode-profile.csv"


def build():
    print("Building with cycle-tracker enabled...")
    subprocess.check_call(
        ["cargo", "build", "--release", "--features", "cycle-tracker"]
    )


def run_trace(file_path, output_dir):
    # cache/input_0x1234.json -> 0x1234
    block_hash = os.path.basename(file_path).replace("input_", "").replace(".json", "")

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

    data = defaultdict(list)
    for filename in trace_files:
        with gzip.open(filename, "rb") as f:
            for name, entries in json.load(f).items():
                data[name].extend((c, g) for c, g in entries)

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
            "total cycles"
        ]
        writer.writerow(header)

        for name, traces in data.items():
            cycle_list = [c for c, _ in traces]
            cpg_list = [c // g for c, g in traces if g > 0]
            if not cpg_list:
                continue

            writer.writerow(
                [
                    name,
                    len(cpg_list),
                    min(cpg_list),
                    int(statistics.median(cpg_list)),
                    max(cpg_list),
                    min(cycle_list),
                    int(statistics.median(cycle_list)),
                    max(cycle_list),
                    sum(cycle_list)
                ]
            )


def main():
    parser = argparse.ArgumentParser(description="Run Zeth cycle profiling")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()

    build()

    files = glob.glob("cache/input_0x*.json")
    print(f"Profiling {len(files)} blocks with {args.jobs} jobs...")

    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"Using temporary directory: {temp_dir}")

        generated_files = []
        with ThreadPoolExecutor(max_workers=args.jobs) as executor:
            for res in executor.map(lambda file: run_trace(file, temp_dir), files):
                if res:
                    generated_files.append(res)

        analyze_traces(generated_files, CSV_FILE)

    print(f"Done. Results saved to {CSV_FILE}")


if __name__ == "__main__":
    main()
