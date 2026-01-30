window.BENCHMARK_DATA = {
  "lastUpdate": 1769795683689,
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
            "value": 900792320,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 737425694,
            "unit": "cycles"
          }
        ]
      },
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
          "id": "d42eefd10cb868e2a088e1836de8d6a353f65818",
          "message": "chore: Add Hoodi network (#197)\n\n* replace Holesky with Hoodi config\n\n* update CHANGELOG.md\n\n* test base_fee_params\n\n* Remove Holesky completely\n\n* Update license header\n\n* update license header\n\n* fail on error\n\n* improve bench\n\n* Make extract script executable",
          "timestamp": "2026-01-30T18:53:15+01:00",
          "tree_id": "7493e4312d13112a9f53801b39a9afe337ddf87e",
          "url": "https://github.com/boundless-xyz/zeth/commit/d42eefd10cb868e2a088e1836de8d6a353f65818"
        },
        "date": 1769795682707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 900792320,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 737425694,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 726888821,
            "unit": "cycles"
          }
        ]
      }
    ]
  }
}