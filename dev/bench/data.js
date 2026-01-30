window.BENCHMARK_DATA = {
  "lastUpdate": 1769796632378,
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
          "id": "6a3714d199203d638d046e2d8570c016d830e60d",
          "message": "chore: Update Reth to version 1.10.2 (#198)\n\n* reduce dependencies\n\n* cleanup Cargo.toml\n\n* run Dependabot\n\n* cleanups\n\n* chore: Update Reth to version 1.10.2 (#200)\n\n* update reth to v1.10.2\n\n* update license header\n\n* update license header\n\n* add accidentally removed Cargo.lock",
          "timestamp": "2026-01-30T19:04:01+01:00",
          "tree_id": "a80f318798e55c6cadea4e364c30d24c7f8c30f9",
          "url": "https://github.com/boundless-xyz/zeth/commit/6a3714d199203d638d046e2d8570c016d830e60d"
        },
        "date": 1769796388959,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 888274944,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 728227295,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 717690435,
            "unit": "cycles"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "06e0f422b59dd46cec670960434e13035bd93c85",
          "message": "build(deps): bump reqwest from 0.12.28 to 0.13.1 (#203)\n\nBumps [reqwest](https://github.com/seanmonstar/reqwest) from 0.12.28 to 0.13.1.\n- [Release notes](https://github.com/seanmonstar/reqwest/releases)\n- [Changelog](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/seanmonstar/reqwest/compare/v0.12.28...v0.13.1)\n\n---\nupdated-dependencies:\n- dependency-name: reqwest\n  dependency-version: 0.13.1\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-30T19:09:01+01:00",
          "tree_id": "d1d4d9729e04610e1687f61c29fb3a8640cde9ba",
          "url": "https://github.com/boundless-xyz/zeth/commit/06e0f422b59dd46cec670960434e13035bd93c85"
        },
        "date": 1769796631631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 888274944,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 728227295,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 717690435,
            "unit": "cycles"
          }
        ]
      }
    ]
  }
}