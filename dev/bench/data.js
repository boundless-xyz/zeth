window.BENCHMARK_DATA = {
  "lastUpdate": 1771151705705,
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
          "id": "6c70ee04388cfdeb577f60c314121321b296f798",
          "message": "feat: add accelerated `modexp` (#204)\n\n* add accelerated modexp\n\n* enable\n\n* Move benchmarking into own job\n\n* run all tests\n\n* num-bigint should be optional\n\n* update CHANGELOG.md",
          "timestamp": "2026-02-05T12:22:53+01:00",
          "tree_id": "a0b2ddee5ce6ff09c6c6244790486e22ed65a4ce",
          "url": "https://github.com/boundless-xyz/zeth/commit/6c70ee04388cfdeb577f60c314121321b296f798"
        },
        "date": 1770290959889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 887095296,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 725670162,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 715133305,
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
          "id": "dd079ab51e5ec023418f722981aa31b87efed461",
          "message": "Update risc0 to 3.0.5 (#208)\n\n* update risc0 to 3.0.5\n\n* fix merge artifact",
          "timestamp": "2026-02-05T15:34:47Z",
          "tree_id": "e2b4f4dee37f3fa11db993c195553a4e4671d6b0",
          "url": "https://github.com/boundless-xyz/zeth/commit/dd079ab51e5ec023418f722981aa31b87efed461"
        },
        "date": 1770306079721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 884473856,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 725879521,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 715342664,
            "unit": "cycles"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "victor@subgraf.dev",
            "name": "Victor Snyder-Graf",
            "username": "nategraf"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "01612b897119f17e3ecece8a9401da575e6478db",
          "message": "Add support for running against a anvil devnet when using proxy (#205)\n\n* build(deps): bump reqwest from 0.12.28 to 0.13.1 (#203)\n\nBumps [reqwest](https://github.com/seanmonstar/reqwest) from 0.12.28 to 0.13.1.\n- [Release notes](https://github.com/seanmonstar/reqwest/releases)\n- [Changelog](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/seanmonstar/reqwest/compare/v0.12.28...v0.13.1)\n\n---\nupdated-dependencies:\n- dependency-name: reqwest\n  dependency-version: 0.13.1\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\n\n* fix copyright year\n\n* use Anvil chain consistently\n\n* fix tests\n\n---------\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: Wolfgang Welz <welzwo@gmail.com>",
          "timestamp": "2026-02-05T17:41:10+01:00",
          "tree_id": "8cf6cbdc93bb3cc43661f2c51b2153727f1a0aff",
          "url": "https://github.com/boundless-xyz/zeth/commit/01612b897119f17e3ecece8a9401da575e6478db"
        },
        "date": 1770309843387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 884473856,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 725879521,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 715342664,
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
          "id": "b36afbb6e744cabc783c2916a9b8f83bd9267ba9",
          "message": "use risc0_zkp sha256 (#211)",
          "timestamp": "2026-02-09T23:11:24+01:00",
          "tree_id": "943d210c6ff092ccf46a1b3fd868b005349bf0c0",
          "url": "https://github.com/boundless-xyz/zeth/commit/b36afbb6e744cabc783c2916a9b8f83bd9267ba9"
        },
        "date": 1770675367166,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 884998144,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 725848704,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 715312981,
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
          "id": "0b1b407c85f93efd0640f978d6766e9b87565733",
          "message": "build(deps): bump anyhow from 1.0.100 to 1.0.101 (#209)\n\nBumps [anyhow](https://github.com/dtolnay/anyhow) from 1.0.100 to 1.0.101.\n- [Release notes](https://github.com/dtolnay/anyhow/releases)\n- [Commits](https://github.com/dtolnay/anyhow/compare/1.0.100...1.0.101)\n\n---\nupdated-dependencies:\n- dependency-name: anyhow\n  dependency-version: 1.0.101\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: Wolfgang Welz <welzwo@gmail.com>",
          "timestamp": "2026-02-12T10:52:04+01:00",
          "tree_id": "61ea376fa87edd0e787d21673e511411098694d0",
          "url": "https://github.com/boundless-xyz/zeth/commit/0b1b407c85f93efd0640f978d6766e9b87565733"
        },
        "date": 1770890043332,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 884998144,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 725848704,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 715312981,
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
          "id": "190b9b68ad8ce20b13de506cb4ec4399b103ca39",
          "message": "Add accelerated `P256VERIFY`, `BN254_ADD` and `BN254_MUL` (#207)\n\n* Move benchmarking into own job\n\n* run all tests\n\n* add accelerated secp256r1_verify_signature\n\n* minor cleanups\n\n* minor cleanups\n\n* update CHANGELOG.md\n\n* extern sys_ only works without zkvm-platform import\n\n* Add accelerated `BN254_ADD` and `BN254_MUL` (#210)\n\n* implement bn254\n\n* crypto module cleanups\n\n* fix cfg\n\n* cleanup Cargo.toml\n\n* Use checked modular ops in satisfies_curve_equation to prevent dishonest prover forgery\n\n* Add overflow guards to limb conversion functions\n\n* use arkworks for the host implementation\n\n* cleanup dependabot.yml\n\n* Optimize satisfies_curve_equation for curves with a=0",
          "timestamp": "2026-02-12T18:24:08+01:00",
          "tree_id": "83db9fe71027b775c092365220aac51a1ab6fb3e",
          "url": "https://github.com/boundless-xyz/zeth/commit/190b9b68ad8ce20b13de506cb4ec4399b103ca39"
        },
        "date": 1770917225551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 875560960,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715701813,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705166089,
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
          "id": "7f4ab41b2c509b3dc5731e7c37bd257a01f12a36",
          "message": "build(deps): bump the alloy group with 4 updates (#212)\n\n* build(deps): bump the alloy group with 4 updates\n\nBumps the alloy group with 4 updates: [alloy](https://github.com/alloy-rs/alloy), [alloy-eips](https://github.com/alloy-rs/alloy), [alloy-genesis](https://github.com/alloy-rs/alloy) and [alloy-primitives](https://github.com/alloy-rs/core).\n\n\nUpdates `alloy` from 1.6.1 to 1.6.3\n- [Release notes](https://github.com/alloy-rs/alloy/releases)\n- [Changelog](https://github.com/alloy-rs/alloy/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/alloy-rs/alloy/compare/v1.6.1...v1.6.3)\n\nUpdates `alloy-eips` from 1.6.1 to 1.6.3\n- [Release notes](https://github.com/alloy-rs/alloy/releases)\n- [Changelog](https://github.com/alloy-rs/alloy/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/alloy-rs/alloy/compare/v1.6.1...v1.6.3)\n\nUpdates `alloy-genesis` from 1.6.1 to 1.6.3\n- [Release notes](https://github.com/alloy-rs/alloy/releases)\n- [Changelog](https://github.com/alloy-rs/alloy/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/alloy-rs/alloy/compare/v1.6.1...v1.6.3)\n\nUpdates `alloy-primitives` from 1.5.4 to 1.5.6\n- [Release notes](https://github.com/alloy-rs/core/releases)\n- [Changelog](https://github.com/alloy-rs/core/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/alloy-rs/core/compare/v1.5.4...v1.5.6)\n\n---\nupdated-dependencies:\n- dependency-name: alloy\n  dependency-version: 1.6.3\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: alloy\n- dependency-name: alloy-eips\n  dependency-version: 1.6.3\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: alloy\n- dependency-name: alloy-genesis\n  dependency-version: 1.6.3\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: alloy\n- dependency-name: alloy-primitives\n  dependency-version: 1.5.6\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n  dependency-group: alloy\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\n\n* update guest Cargo.lock\n\n---------\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: Wolfgang Welz <welzwo@gmail.com>",
          "timestamp": "2026-02-12T18:45:24+01:00",
          "tree_id": "abc740270e5631bc0209ad69cf436e7897e4f61f",
          "url": "https://github.com/boundless-xyz/zeth/commit/7f4ab41b2c509b3dc5731e7c37bd257a01f12a36"
        },
        "date": 1770918415286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 874512384,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715714836,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705179112,
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
          "id": "152b334b0c241e5979730913b0937c1383e9e6c0",
          "message": "Add Wycheproof tests for P-256 ECDSA verification (#216)",
          "timestamp": "2026-02-12T20:32:40+01:00",
          "tree_id": "09bb8b3be5e575aedccaffd0ab303b55b32f4ec3",
          "url": "https://github.com/boundless-xyz/zeth/commit/152b334b0c241e5979730913b0937c1383e9e6c0"
        },
        "date": 1770925189860,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 874512384,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715714836,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705179112,
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
          "id": "c290c83f616f16e35e54cfd178be33bf4b90b843",
          "message": "build(deps): bump tempfile from 3.24.0 to 3.25.0 (#214)\n\nBumps [tempfile](https://github.com/Stebalien/tempfile) from 3.24.0 to 3.25.0.\n- [Changelog](https://github.com/Stebalien/tempfile/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/Stebalien/tempfile/commits)\n\n---\nupdated-dependencies:\n- dependency-name: tempfile\n  dependency-version: 3.25.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-15T11:01:23+01:00",
          "tree_id": "451701cdf0c815824baadefac71a4077ddb38f58",
          "url": "https://github.com/boundless-xyz/zeth/commit/c290c83f616f16e35e54cfd178be33bf4b90b843"
        },
        "date": 1771149792553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 874512384,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715714836,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705179112,
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
          "id": "f9e2137d6c7ed07f792a82480a777ea1b188b4f0",
          "message": "build(deps): bump clap from 4.5.57 to 4.5.58 (#213)\n\nBumps [clap](https://github.com/clap-rs/clap) from 4.5.57 to 4.5.58.\n- [Release notes](https://github.com/clap-rs/clap/releases)\n- [Changelog](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.5.57...clap_complete-v4.5.58)\n\n---\nupdated-dependencies:\n- dependency-name: clap\n  dependency-version: 4.5.58\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-15T11:04:57+01:00",
          "tree_id": "30ff5735775f293a32c6653f619a92065c5aafcb",
          "url": "https://github.com/boundless-xyz/zeth/commit/f9e2137d6c7ed07f792a82480a777ea1b188b4f0"
        },
        "date": 1771149981864,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 874512384,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715714836,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705179112,
            "unit": "cycles"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "victor@risczero.com",
            "name": "Victor Snyder-Graf",
            "username": "nategraf"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8595c915f59c06688293f52fde38b33c8fbe770f",
          "message": "Increase default keep-alive on zeth-rpc-proxy (#219)\n\n* Increase default keep-alive on zeth-rpc-proxy\n\n* fix\n\n---------\n\nCo-authored-by: Wolfgang Welz <welzwo@gmail.com>",
          "timestamp": "2026-02-15T11:31:28+01:00",
          "tree_id": "d18f3523cdabbfcfef93331c8005afeff94b4425",
          "url": "https://github.com/boundless-xyz/zeth/commit/8595c915f59c06688293f52fde38b33c8fbe770f"
        },
        "date": 1771151705027,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "total_cycles",
            "value": 874512384,
            "unit": "cycles"
          },
          {
            "name": "user_cycles",
            "value": 715714836,
            "unit": "cycles"
          },
          {
            "name": "read_input_cycles",
            "value": 10504980,
            "unit": "cycles"
          },
          {
            "name": "validation_cycles",
            "value": 705179112,
            "unit": "cycles"
          }
        ]
      }
    ]
  }
}