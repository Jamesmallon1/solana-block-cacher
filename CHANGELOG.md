## [1.0.8](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.7...v1.0.8) (2024-01-08)


### Bug Fixes

* **fbs:** improved thread completion counter using atomics ([b92aa04](https://github.com/Jamesmallon1/solana-block-cacher/commit/b92aa0488d7321e62dab9d7a29e30f40e687be71))

## [1.0.7](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.6...v1.0.7) (2024-01-05)


### Bug Fixes

* **block-fetcher:** remove error messages pertaining to block missing ([49bd4a1](https://github.com/Jamesmallon1/solana-block-cacher/commit/49bd4a14f1939a31f3fef2f705e78b62577dd25c))

## [1.0.6](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.5...v1.0.6) (2024-01-05)


### Bug Fixes

* **verioning:** increment to deploy previous fixes ([5f474cc](https://github.com/Jamesmallon1/solana-block-cacher/commit/5f474ccea9de61857486de0d05c4ebd44d9eaecb))

## [1.0.5](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.4...v1.0.5) (2024-01-05)


### Bug Fixes

* **block-fetcher:** added latest transaction support ([ab301bf](https://github.com/Jamesmallon1/solana-block-cacher/commit/ab301bf951065714c6395cc2dcc52631b401d40e))
* **block-fetcher:** set verbosity of error lower as block has been skip ([5ea40d9](https://github.com/Jamesmallon1/solana-block-cacher/commit/5ea40d98fc7e4d045b5320f25dfb38736b5026be))
* **threading:** added random sampling of numbers within range ([4f46b64](https://github.com/Jamesmallon1/solana-block-cacher/commit/4f46b64bdd035b93f83f098f4c791ac0546b3147))
* **threading:** update sample block for threading calc ([294c53e](https://github.com/Jamesmallon1/solana-block-cacher/commit/294c53e16b471d2edf7aec9a68c28a60a3391280))

## [1.0.4](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.3...v1.0.4) (2024-01-04)


### Bug Fixes

* **write-service:** remove last comma and close array ([4d4c767](https://github.com/Jamesmallon1/solana-block-cacher/commit/4d4c7676cfed894228f4be5c2083ef423114b3ec))

## [1.0.3](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.2...v1.0.3) (2024-01-04)


### Bug Fixes

* **semver:** remove cargo-semantic-release for manual method ([d74f500](https://github.com/Jamesmallon1/solana-block-cacher/commit/d74f5001db8b4e0c68c5846ef396e20d4a0ec773))

## [1.0.2](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.1...v1.0.2) (2024-01-04)


### Bug Fixes

* **fbs:** resolved issue where threads panicn due to start_slot ([bab474d](https://github.com/Jamesmallon1/solana-block-cacher/commit/bab474dc2bb6d2aedc73aba45a0951d4edda61e7))

## [1.0.1](https://github.com/Jamesmallon1/solana-block-cacher/compare/v1.0.0...v1.0.1) (2024-01-04)


### Bug Fixes

* **main:** validation error message of cli arguments ([7fdbb8f](https://github.com/Jamesmallon1/solana-block-cacher/commit/7fdbb8f29f6298e8138e5f7abc62dfe890c02ec9))

# 1.0.0 (2024-01-04)


### Bug Fixes

* **fbs:** added block fetcher factory to files for further testing ([d49facd](https://github.com/Jamesmallon1/solana-block-cacher/commit/d49facd12a7fa39b3eaf7a5bb15bf56455784838))
* **fbs:** align block fetch service with new queue logic ([ba16669](https://github.com/Jamesmallon1/solana-block-cacher/commit/ba16669d0def3065ca9c5b72d44bab09af733f35))
* **fbs:** blocks now do not jump outside of given ranges ([0467570](https://github.com/Jamesmallon1/solana-block-cacher/commit/0467570fc4ad5f0e644777a4aec96366b77c40ac))
* **fbs:** enhanced service to pull contiguous batches in parallel ([2604941](https://github.com/Jamesmallon1/solana-block-cacher/commit/2604941b89cfe79fb7aceebe84238e770f8ea07e))
* **logging:** removed warning from unused variable ([694b7b9](https://github.com/Jamesmallon1/solana-block-cacher/commit/694b7b96dbbb817b112b872c3f3421fb371e54a6))
* **main:** added explicit use for queue trait ([8a1b931](https://github.com/Jamesmallon1/solana-block-cacher/commit/8a1b9315bd9f0848dd7d56bef4cfa9349e13f8b0))
* **main:** resolved args in main and added test cases for validation ([3a07d23](https://github.com/Jamesmallon1/solana-block-cacher/commit/3a07d23f709d9e4daf769967992ae0d506d7ae45))
* **mocking:** add automock ([b920414](https://github.com/Jamesmallon1/solana-block-cacher/commit/b920414108505ee287e66c5df071308741c7a045))
* **priority-queue:** separate trait from struct ([b54efff](https://github.com/Jamesmallon1/solana-block-cacher/commit/b54efffb922728a62ee81bd41beeb10e2c1aaa27))
* **rate-limiter:** separate trait from struct for mocking + lc ([1006ed5](https://github.com/Jamesmallon1/solana-block-cacher/commit/1006ed57fa712cc5bd572e3555babaaabe29033f))
* **threading:** increased testability ([4dccf95](https://github.com/Jamesmallon1/solana-block-cacher/commit/4dccf9569f9398b0b2ea472a1eb21ef923f3721f))
* **threading:** retrieves correct optimum number of threads to run ([79703a8](https://github.com/Jamesmallon1/solana-block-cacher/commit/79703a8402b4afbaea9ba361750ba06dacf5fc71))
* **write-service:** improve file usage by clearing then writing ([8d065fd](https://github.com/Jamesmallon1/solana-block-cacher/commit/8d065fd6a059b8c2351c0678e92c1c8b3593acc8))
* **write-service:** resolved priority queue ordering and writing ([16b62cc](https://github.com/Jamesmallon1/solana-block-cacher/commit/16b62ccc596cd126c5dcc461bf00e9940f4f14e2))


### Features

* **block-fetcher-factory:** added a factory for generating rpc clients ([bb33ca9](https://github.com/Jamesmallon1/solana-block-cacher/commit/bb33ca91749ba1693749d22d74a5b3abe8e93bcc))
* **cli:** added the CLI arguments and logger initialization ([66ff471](https://github.com/Jamesmallon1/solana-block-cacher/commit/66ff471eeb60733f06bac39d11c781d8b8d01153))
* **config:** rust format configuration ([e50712e](https://github.com/Jamesmallon1/solana-block-cacher/commit/e50712e5e3bd9ccc5651266012a7cdfa03e7badc))
* **fbs:** added block fetching initial function ([3a0a7b4](https://github.com/Jamesmallon1/solana-block-cacher/commit/3a0a7b4e5cd09436842109b405051276489f2520))
* **fbs:** enhanced block pulling efficiency by pulling multiple blocks ([de75156](https://github.com/Jamesmallon1/solana-block-cacher/commit/de7515621006944d689b1b1ee77e040830b8869e))
* **fbs:** updated the service with further logic ([ef83208](https://github.com/Jamesmallon1/solana-block-cacher/commit/ef832087cdc907b3b60ca999ea929b504496f37b))
* **gitignore:** added gitignore file ([7da2ba5](https://github.com/Jamesmallon1/solana-block-cacher/commit/7da2ba5250bf97e57a8b502f581d79a0eb302c9e))
* **initial-project:** added the bare rust project ([0289660](https://github.com/Jamesmallon1/solana-block-cacher/commit/02896603a32413a5ae046b3fd2769a7e5f703985))
* **priority-queue:** added thread idle mech wait_for_data ([a980c16](https://github.com/Jamesmallon1/solana-block-cacher/commit/a980c1657afca4b5122d93c7280a4ba715599c56))
* **priority-queue:** implemented a priority queue for writing blocks ([a41e1c6](https://github.com/Jamesmallon1/solana-block-cacher/commit/a41e1c6204d03de9542e97b8620ca26cee1e57a2))
* **progress-bar:** added progress bar to cli ([ff53a86](https://github.com/Jamesmallon1/solana-block-cacher/commit/ff53a8604e4541228473b2ee3c399e10f9541286))
* **rate-limiter:** added rate limiter to the application ([5f0135e](https://github.com/Jamesmallon1/solana-block-cacher/commit/5f0135e884c9c880eba50980b3b99097cfde9690))
* **solana-block:** added batch and block structs and impls ([91d76f7](https://github.com/Jamesmallon1/solana-block-cacher/commit/91d76f7a0ec5d902368ce8802c1e8130db088483))
* **solana-block:** added new and push functions ([235c343](https://github.com/Jamesmallon1/solana-block-cacher/commit/235c34326e62b5b038135b5123eb0a986e000c32))
* **threading:** added basic threading module to handle concurrency ([ae24694](https://github.com/Jamesmallon1/solana-block-cacher/commit/ae24694fb1a571b6657c88682ff2388bbd3c8c0a))
* **threading:** updated project name and added thread spawn calc ([d996ec3](https://github.com/Jamesmallon1/solana-block-cacher/commit/d996ec31df2262c84172ae85117fcc45ed1e121d))
* **write-service:** added service that spawns thread to read queue ([7d4c69a](https://github.com/Jamesmallon1/solana-block-cacher/commit/7d4c69a2c07da94c4d7a9cb229548cd6369c0a04))
