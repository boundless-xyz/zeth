window.BENCHMARK_DATA = {
  "lastUpdate": 1769789183353,
  "repoUrl": "https://github.com/boundless-xyz/zeth",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "welzwo@gmail.com",
            "name": "Wolfgang Welz",
            "username": "Wollac"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "20b936f9672549071df112f6a2604d620ef5cff7",
          "message": "ci: Add cycle count regression tracking (#202)\n\n* ci: Add cycle count regression tracking\n\nExtract total and user cycle counts from e2e test output and track\nthem using github-action-benchmark. On PRs, posts a comparison comment\nand fails the build if cycles regress more than 5%.\n\n* use @v1",
          "timestamp": "2026-01-30T17:05:04+01:00",
          "tree_id": "ab2194e420e75c5c91d1177cbafe77f71f44fc78",
          "url": "https://github.com/boundless-xyz/zeth/commit/20b936f9672549071df112f6a2604d620ef5cff7"
        },
        "date": 1769789182669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 0,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 0,
            "unit": "cycles"
          }
        ]
      }
    ]
  }
}