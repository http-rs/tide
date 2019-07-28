## 2019-05-03, Version 0.2.0
### Commits
- [[`d887e16`](https://github.com/rustasync/tide/commit/d887e162d88289b43a6dba4ac8a98040af35d02d)] (cargo-release) version 0.2.0 (Wonwoo Choi)
- [[`6d8577b`](https://github.com/rustasync/tide/commit/6d8577b5411c4a5ad636debc056e9af69d3a107b)] Fix wrong changelog (Wonwoo Choi)
- [[`69b5762`](https://github.com/rustasync/tide/commit/69b576208f6191029476e7cbec90b19b057f4081)] Merge pull request #193 from Nemo157/futures-alpha.15 (Wonwoo Choi)
- [[`a676391`](https://github.com/rustasync/tide/commit/a6763919525ed6b9d7ee5a5a3ef1dda5b47b495c)] Update for futures v0.3.0-alpha.15 (Wim Looman)
- [[`6b6a7a1`](https://github.com/rustasync/tide/commit/6b6a7a18a196e94aa532dd5059df2e116950bf03)] fix: readme typos (Prasanna V. Loganathar)
- [[`676b497`](https://github.com/rustasync/tide/commit/676b497f8cafcc2709a5fcc91ec430265af34d2d)] Fix minor bugs found during review (#188) (Murali)
- [[`a4464e8`](https://github.com/rustasync/tide/commit/a4464e8806dbfb817104fa15da7a107128fe53f0)] Merge pull request #170 from mmrath/cookie-improvement (Aaron Turon)
- [[`d76a6d2`](https://github.com/rustasync/tide/commit/d76a6d23556caa0368b958d16e534942c4c6a2c9)] Merge pull request #180 from secretfader/serde_urlencoded (Aaron Turon)
- [[`4263895`](https://github.com/rustasync/tide/commit/4263895dcc8729883b9d8d0cd5c87da192963905)] Merge pull request #175 from secretfader/feat/querystring (Aaron Turon)
- [[`85065cf`](https://github.com/rustasync/tide/commit/85065cf36e8b84bd7a95ca4a49b960f87f8b7fc7)] Add query string extension trait (Nicholas Young)
- [[`998fa45`](https://github.com/rustasync/tide/commit/998fa45af2a8b7dd8c1ac3ccc70c785003fd015d)] Merge pull request #183 from secretfader/serde-2018 (Justin Seabrook-Rocha)
- [[`6438a3d`](https://github.com/rustasync/tide/commit/6438a3db79b4bdda4c75a9595e9f1ff7764c0050)] docs: add supported rust versions (#185) (Prasanna V. Loganathar)
- [[`cf3eee2`](https://github.com/rustasync/tide/commit/cf3eee2252f5e3cdc3a0d91a0f89a1a9d9a5149d)] pragma updates (#182) (Prasanna V. Loganathar)
- [[`4476c69`](https://github.com/rustasync/tide/commit/4476c6976217b9eda116b8e15b4697c71a250eb4)] Merge pull request #178 from rustasync/fix-github-templates (Nicholas)
- [[`87ff6ff`](https://github.com/rustasync/tide/commit/87ff6ff77132686aecdcbee1a47be2395a7bf187)] Update question.md (Theodore Zilist)
- [[`8f1daf7`](https://github.com/rustasync/tide/commit/8f1daf772c44798dc63aae61a7561cfffe61d341)] Use Rust 2018 imports for Serde (Nicholas Young)
- [[`0dedfc0`](https://github.com/rustasync/tide/commit/0dedfc0ec98162e9cffe8244028f5899bf5d5002)] Migrate to serde_urlencoded, and away from serde_qs (Nicholas Young)
- [[`d78001f`](https://github.com/rustasync/tide/commit/d78001f3d776c9a16375597572f2c84b1eff18d5)] Merge pull request #181 from sangheestyle/fix_example_readme (Nicholas)
- [[`0ac0168`](https://github.com/rustasync/tide/commit/0ac01681a13ec46f9dc3c9873a1073c2429aeba8)] fix: error on example code (Sanghee Kim)
- [[`6dab8f7`](https://github.com/rustasync/tide/commit/6dab8f777048e37735a48b9aebbf8441c0254dc4)] Update question.md (Theodore Zilist)
- [[`1a185c5`](https://github.com/rustasync/tide/commit/1a185c57ce60d034b12edf31a65eabc3a72b2cfe)] Fix documentation (Murali Mohan Rath)
- [[`8e3fd26`](https://github.com/rustasync/tide/commit/8e3fd26236bf4b436334ab4400e5c5893aa69235)] Merge branch 'master' of https://github.com/rustasync/tide into cookie-improvement (Murali Mohan Rath)
- [[`e06ae37`](https://github.com/rustasync/tide/commit/e06ae375aa31e8fe2dad3689c72bd33e84775d4e)] Update changelog (Yoshua Wuyts)
- [[`31df6c2`](https://github.com/rustasync/tide/commit/31df6c2cec56157ec675d42ebb7cf41c9fc5476b)] Add test cases (Murali Mohan Rath)
- [[`03cf8f1`](https://github.com/rustasync/tide/commit/03cf8f16d6182f05e27b9d9e90771b4ccd963ff9)] Add documentation (Murali Mohan Rath)
- [[`ca059c0`](https://github.com/rustasync/tide/commit/ca059c0f238b93117d04f30befdfc80453bac074)] Merge branch 'master' of https://github.com/rustasync/tide into cookie-improvement (Murali Mohan Rath)
- [[`f8f6203`](https://github.com/rustasync/tide/commit/f8f6203e83bac4ca923de21a0899af4dce069b8e)] improve error handling (Murali Mohan Rath)
- [[`935151a`](https://github.com/rustasync/tide/commit/935151a56ace7356f6123df92f93f827a0b71785)] Cookie revamp (Murali Mohan Rath)
- [[`2e44305`](https://github.com/rustasync/tide/commit/2e443055ab4f8da80b1a22e315ed47a53b504712)] cookie revamp (Murali Mohan Rath)

### Stats
```diff
 .github/ISSUE_TEMPLATE/question.md |   4 +-
 .travis.yml                        |   2 +-
 CHANGELOG.md                       | 107 +++++++++++++++++------
 Cargo.toml                         |  17 ++--
 README.md                          |  19 ++++-
 examples/body_types.rs             |   6 +-
 examples/catch_all.rs              |   2 +-
 examples/cookie_extractor.rs       |  15 ----
 examples/cookies.rs                |  26 ++++++
 examples/graphql.rs                |   2 +-
 examples/messages.rs               |   6 +-
 examples/multipart-form/main.rs    |   6 +-
 src/app.rs                         |   8 +-
 src/cookies.rs                     |  92 +++++++++++++++-----
 src/endpoint.rs                    |   6 +-
 src/error.rs                       |   4 +-
 src/forms.rs                       |   7 +-
 src/lib.rs                         |  10 +--
 src/middleware/cookies.rs          | 170 +++++++++++++++++++++++++++++++++++++
 src/middleware/default_headers.rs  |   4 +-
 src/middleware/logger.rs           |   4 +-
 src/middleware/mod.rs              |  13 +--
 src/querystring.rs                 |  80 +++++++++++++++++
 src/router.rs                      |   9 +-
 tests/wildcard.rs                  |   2 +-
 25 files changed, 498 insertions(+), 123 deletions(-)
```


## 2019-04-18, Version 0.1.1
### Commits
- [[`8c247b8`](https://github.com/rustasync/tide/commit/8c247b884fe6795c9489a76d7ee9f265a2d6539c)] (cargo-release) version 0.1.1 (Yoshua Wuyts)
- [[`7e57a55`](https://github.com/rustasync/tide/commit/7e57a55e8ff1229594a439141e9bb6c204f74703)] fix: sync up with nightly futures_api (#176) (Prasanna V. Loganathar)
- [[`69c0c60`](https://github.com/rustasync/tide/commit/69c0c60bacef29dd0e0071c799ace3a2f0977aaa)] fix: cargo fmt check in the last commit (Prasanna V. Loganathar)
- [[`0281fa6`](https://github.com/rustasync/tide/commit/0281fa671cafa9761f08282c5df823bb13c3082d)] fix: 2018 edition lints (Prasanna V. Loganathar)
- [[`b09fcda`](https://github.com/rustasync/tide/commit/b09fcdaee34e338e178a8ce51d0bd52b621e5570)] compiler pragmas (Prasanna V. Loganathar)
- [[`9e0a19a`](https://github.com/rustasync/tide/commit/9e0a19ad2ed6168737a44d9f82c00453f9ef8260)] Documents acceptable endpoint fns (#177) (Allen)
- [[`3d22441`](https://github.com/rustasync/tide/commit/3d224417398d81e83edac230d8d0ad2c16849d23)] redo readme (#172) (Yoshua Wuyts)
- [[`be9832d`](https://github.com/rustasync/tide/commit/be9832d3a677944acd904821652eb027853eb7c0)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 .travis.yml     |   2 +-
 CHANGELOG.md    |  31 ++++++++++++++++
 Cargo.toml      |  24 ++++++-------
 README.md       | 109 ++++++++++++++++++++++++++++++++++++++++++++++++++++----
 src/endpoint.rs |  20 ++++++++++-
 src/error.rs    |   2 +-
 src/lib.rs      |   7 +++-
 7 files changed, 173 insertions(+), 22 deletions(-)
```


## 2019-04-15, Version 0.1.0
### Commits
- [[`8c6ecd6`](https://github.com/rustasync/tide/commit/8c6ecd695845b7460ee9098c29ac9c0292886a45)] (cargo-release) version 0.1.0 (Yoshua Wuyts)
- [[`2791a61`](https://github.com/rustasync/tide/commit/2791a61b334da2cc309a2e8206c69b4c2e6e8346)] re-export http crate (#166) (Thomas Lee)
- [[`dc36c89`](https://github.com/rustasync/tide/commit/dc36c898ff145a4175aee074bea0e291a1d47ffb)] fix: Capture wildcard path correctly, resolve #167. (#171) (Nicholas)
- [[`c70d2bc`](https://github.com/rustasync/tide/commit/c70d2bcd5ee4c0a09a87055ba7973e4c59d09156)] example: remove asterisk from route params. Resolves #167. (#168) (Nicholas)
- [[`dc28fd5`](https://github.com/rustasync/tide/commit/dc28fd5601d8b8d071de999e080f2c347c3fd61d)] Follow up to #156 (Aaron Turon)
- [[`7a9519a`](https://github.com/rustasync/tide/commit/7a9519a308f1dcc2572a8887040cdeae4590bfb6)] Merge pull request #161 from rustasync/inline-docs (Aaron Turon)
- [[`e51c0c8`](https://github.com/rustasync/tide/commit/e51c0c863be21760e90b8c3b640e7892df28a207)] inline docs (Yoshua Wuyts)
- [[`f8d00ad`](https://github.com/rustasync/tide/commit/f8d00adfb2c72733fb47a5b11e311af455074a3b)] Merge pull request #156 from aturon/revamp (Aaron Turon)
- [[`adba649`](https://github.com/rustasync/tide/commit/adba649a79a992e1eb1d5db4abfdf3ca805d76b1)] Revamp Tide, dropping Extractors and simplifying the framework (Aaron Turon)
- [[`d14e8f8`](https://github.com/rustasync/tide/commit/d14e8f82f0df3dec25f55a8f26749a4aca384837)] Update references to old repository (#157) (Pradip Caulagi)
- [[`7e87e4e`](https://github.com/rustasync/tide/commit/7e87e4e0699a3fff4453c8a25d3f8f5fa186fdbb)] Update to nightly-2019-02-27 and make it default (#146) (Wonwoo Choi)
- [[`70ed8aa`](https://github.com/rustasync/tide/commit/70ed8aa44a9eba129d879304630c406caab2fee5)] Merge pull request #142 from gruberb/use_external_http_service_mock (Wonwoo Choi)
- [[`56e5076`](https://github.com/rustasync/tide/commit/56e5076c469fc525bf8fec241aece8f2eb14045e)] Update the CI badge to point to rustasync/tide (#145) (David Cornu)
- [[`9bef037`](https://github.com/rustasync/tide/commit/9bef0370dc0554a2f1d49952d893d4d3dfe9da83)] Update blog post URLs after wg-net repo move (#144) (Tim Neumann)
- [[`644f47e`](https://github.com/rustasync/tide/commit/644f47e8ddc614de9786954f1e0e68965516f70a)] Add routes to each test (Bastian Gruber)
- [[`8170c5f`](https://github.com/rustasync/tide/commit/8170c5f27ad52b8b096fe9d541f9a9ca200ef5a4)] Add external crate http_service_mock, delete local TestBackend (Bastian Gruber)
- [[`bb7ad2b`](https://github.com/rustasync/tide/commit/bb7ad2bdcf7a451e7c9211fc13098baa96042a0a)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 .github/PULL_REQUEST_TEMPLATE.md    |   2 +-
 .travis.yml                         |   5 +-
 CHANGELOG.md                        |  31 ++
 Cargo.toml                          |  10 +-
 README.md                           |  12 +-
 examples/body_types.rs              |  53 ++-
 examples/catch_all.rs               |  14 +-
 examples/cli_parsing.rs             |  48 ---
 examples/computed_values.rs         |  38 ---
 examples/configuration.rs           |  34 --
 examples/cookie_extractor.rs        |  11 +-
 examples/default_handler.rs         |  13 -
 examples/default_headers.rs         |   4 +-
 examples/graphql.rs                 |  32 +-
 examples/hello.rs                   |   5 +-
 examples/messages.rs                |  53 ++-
 examples/multipart-form/main.rs     |  17 +-
 examples/named_path.rs              |  29 --
 examples/simple_nested_router.rs    |  38 ---
 src/app.rs                          | 464 ++++++++++++++++---------
 src/body.rs                         | 351 -------------------
 src/configuration/default_config.rs |  79 -----
 src/configuration/mod.rs            | 156 ---------
 src/context.rs                      | 141 ++++++++
 src/cookies.rs                      |  51 ++-
 src/endpoint.rs                     | 167 ++-------
 src/error.rs                        | 102 ++++++
 src/extract.rs                      |  20 --
 src/forms.rs                        |  55 +++
 src/head.rs                         | 225 ------------
 src/lib.rs                          |  41 ++-
 src/middleware/default_headers.rs   |  32 +-
 src/middleware/logger.rs            |  39 ++-
 src/middleware/mod.rs               |  63 ++--
 src/request.rs                      |  65 ----
 src/response.rs                     |  25 +-
 src/route.rs                        | 101 ++++++
 src/router.rs                       | 657 +++---------------------------------
 src/serve.rs                        |   4 -
 tests/wildcard.rs                   |  70 +---
 40 files changed, 1036 insertions(+), 2321 deletions(-)
```


## 2019-02-26, Version 0.0.5
### Commits
- [[`990c80f78e`](https://github.com/rust-net-web/tide/commit/990c80f78e5622a751a5d2d4948cd005616e84f8)] (cargo-release) version 0.0.5 (Yoshua Wuyts)
- [[`61f2aa5bf7`](https://github.com/rust-net-web/tide/commit/61f2aa5bf71f8eff34b1f9dcbb97e83a86f33b92)] Extract serve.rs into a separate crate (#140) (Wonwoo Choi)
- [[`b4d0806a15`](https://github.com/rust-net-web/tide/commit/b4d0806a15e50ccbfcffeaf5bb6f767019269670)] Merge pull request #139 from aturon/http-service (Aaron Turon)
- [[`092fc7f4e4`](https://github.com/rust-net-web/tide/commit/092fc7f4e4764b3a4c0101046f0ceced95f09194)] update rust version (Aaron Turon)
- [[`703d41b79a`](https://github.com/rust-net-web/tide/commit/703d41b79a3089640731cae2de54b4dec0c0c93a)] rustfmt (Aaron Turon)
- [[`fe0c39cc60`](https://github.com/rust-net-web/tide/commit/fe0c39cc60b0ff54bc577a3d8d21b4821c13841a)] Update to use new Body::into_vec method (Aaron Turon)
- [[`369095140d`](https://github.com/rust-net-web/tide/commit/369095140d0acdad0656e0d79a3a64c9aae87436)] update tests (Aaron Turon)
- [[`9fee015612`](https://github.com/rust-net-web/tide/commit/9fee015612fc89c78fb5555a837ad03af2c32ef1)] Refactor to use HttpService internally (Aaron Turon)
- [[`1fb71bf421`](https://github.com/rust-net-web/tide/commit/1fb71bf421ef27b6acf8c23d5d0a267a9b385d62)] Move to http_service::Body (Aaron Turon)
- [[`c727750a69`](https://github.com/rust-net-web/tide/commit/c727750a694b0a2f217eb391a5d8d9f07b722802)] docs: updated docs to have correct default port (#137) (Matt Gathu)
- [[`b02220a06a`](https://github.com/rust-net-web/tide/commit/b02220a06aac5bf42455bcc6c9bdc36368fe9a9f)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 .travis.yml       |   3 +-
 CHANGELOG.md      |  18 +++++++-
 Cargo.toml        |  25 ++++++----
 src/app.rs        |  58 ++++++++----------------
 src/body.rs       | 133 +++++--------------------------------------------------
 src/lib.rs        |   4 +-
 src/request.rs    |   3 +-
 src/response.rs   |   3 +-
 src/router.rs     |  27 ++++-------
 src/serve.rs      |   4 ++-
 tests/wildcard.rs |  96 ++++++++++++++++++++++++++++++++++++++++-
 11 files changed, 189 insertions(+), 185 deletions(-)
```


## 2019-02-04, Version 0.0.4
### Commits
- [[`2128abc1d0`](https://github.com/rust-net-web/tide/commit/2128abc1d01b95eabdfadc8ce2d587ba2bb1d62e)] (cargo-release) version 0.0.4 (Yoshua Wuyts)
- [[`cbd7525557`](https://github.com/rust-net-web/tide/commit/cbd7525557ee7ddb65a6ed5103618df5263cd5a6)] pin nightly version on CI (#136) (Yoshua Wuyts)
- [[`3455f96cf9`](https://github.com/rust-net-web/tide/commit/3455f96cf9eca83baad31d9a2a7170f9f1a6fde3)] Merge pull request #135 from rust-net-web/fix-env-pub (Aaron Turon)
- [[`9402aef6d5`](https://github.com/rust-net-web/tide/commit/9402aef6d58e3207100c6d4d177ba1de2fa2eaa1)] make the Environment struct public (Yoshua Wuyts)
- [[`012b49dc15`](https://github.com/rust-net-web/tide/commit/012b49dc1576821fcb637802e552dae5409041b5)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 .travis.yml              |  1 +
 CHANGELOG.md             | 32 ++++++++++++++++++++++++++++++++
 Cargo.toml               |  2 +-
 src/configuration/mod.rs |  2 +-
 4 files changed, 35 insertions(+), 2 deletions(-)
```


## 2019-01-31, Version 0.0.3
### Commits
- [[`658aa8222a`](https://github.com/rust-net-web/tide/commit/658aa8222a6467e289ee992eeed7b7cfb27ebf5b)] (cargo-release) version 0.0.3 (Yoshua Wuyts)
- [[`4aeb4b831d`](https://github.com/rust-net-web/tide/commit/4aeb4b831d883819a20d61ed48e3f4bce3f7f731)] Fixes GH-130 (#133) (Yoshua Wuyts)
- [[`5f7e387bb0`](https://github.com/rust-net-web/tide/commit/5f7e387bb09f4fae4318db7b030dd6b3ccfc613c)] Merge pull request #132 from bIgBV/cli-example (Bhargav)
- [[`af8db24cdd`](https://github.com/rust-net-web/tide/commit/af8db24cdd491f14ef067efa615120c46e58d144)] change 'Content-Type' for IntoResponse::into_response(Vec<u8>) (#124) (DCjanus)
- [[`fd575921f8`](https://github.com/rust-net-web/tide/commit/fd575921f80d607087f2c46228f80db38f2e6818)] Basic cookie extractor (#114) (Murali)
- [[`6554105144`](https://github.com/rust-net-web/tide/commit/6554105144ae989648ba3827ba799203db30f0c1)] Add example to show CLI parsing integration with tide app (Bhargav Voleti)
- [[`bf1e3d8103`](https://github.com/rust-net-web/tide/commit/bf1e3d8103d66994b8510317f0544e83359035b2)] Merge pull request #131 from rust-net-web/tmp-fix-dep (Wonwoo Choi)
- [[`bf7cb2e145`](https://github.com/rust-net-web/tide/commit/bf7cb2e145f90967962d52ee4bfe33608c971fef)] temporarily override fix-cookie source (Yoshua Wuyts)
- [[`4517d11c32`](https://github.com/rust-net-web/tide/commit/4517d11c32123ef47745a0a32a6ba9c7f2f22806)] Fix links from contributing to code of conduct (#128) (HeroicKatora)
- [[`efd51ac407`](https://github.com/rust-net-web/tide/commit/efd51ac4073a30daff53afd3f807dc8ec23b8f2e)] Clean up doctests and messages example via Default trait (#125) (whentze)
- [[`9b3fe0c9dd`](https://github.com/rust-net-web/tide/commit/9b3fe0c9ddcbeca08b9815f51164b22f263e854b)] Update changelog (Yoshua Wuyts)

### Stats
```diff
 .github/CONTRIBUTING.md      |   6 +-
 .gitignore                   |   1 +-
 CHANGELOG.md                 | 226 ++++++++++++++++++++++++++++++++++++++++++++-
 Cargo.toml                   |   6 +-
 examples/cli_parsing.rs      |  48 +++++++++-
 examples/cookie_extractor.rs |  18 ++++-
 examples/messages.rs         |  10 +--
 src/app.rs                   |  12 +--
 src/cookies.rs               |  55 +++++++++++-
 src/endpoint.rs              |  12 +--
 src/lib.rs                   |   2 +-
 src/response.rs              |  20 +++-
 12 files changed, 381 insertions(+), 35 deletions(-)
```


## 2019-01-18, Version 0.0.1
### Commits
- [[`ae2faa1eef`](https://github.com/rust-net-web/tide/commit/ae2faa1eefba07e551e5766e458555a3a3109062)] (cargo-release) version 0.0.1 (Yoshua Wuyts)
- [[`f0599031e3`](https://github.com/rust-net-web/tide/commit/f0599031e332b2ccfd6dcd391fbcaa56a6cde246)] Merge pull request #122 from tirr-c/intoresponse-no-static (Aaron Turon)
- [[`fe734d7df3`](https://github.com/rust-net-web/tide/commit/fe734d7df3f3b025e7142f74516c448b3f4c476f)] Relieve lifetime bound of IntoResponse (Wonwoo Choi)
- [[`ef4406766a`](https://github.com/rust-net-web/tide/commit/ef4406766a11716f5ab56b7bf953a003b4d826b1)] use path-table crate (#121) (Yoshua Wuyts)
- [[`71393645c0`](https://github.com/rust-net-web/tide/commit/71393645c07075df8cedb0bf1703a7acddca0594)] Per-endpoint configuration (#109) (Wonwoo Choi)
- [[`5f4cc3d6cb`](https://github.com/rust-net-web/tide/commit/5f4cc3d6cbbc3666c70106090cf235a97119d503)] Update futures-preview to 0.3.0-alpha.12 (#119) (Wonwoo Choi)
- [[`5be7493412`](https://github.com/rust-net-web/tide/commit/5be74934125a5b3e79bca7aa9ffd4f240ca04050)] Merge pull request #118 from tirr-c/response-status (Aaron Turon)
- [[`8be6d4c445`](https://github.com/rust-net-web/tide/commit/8be6d4c4454f2bc0853e205bcd8aaa78e7f6b34c)] Run rustfmt (Wonwoo Choi)
- [[`ddd47656fc`](https://github.com/rust-net-web/tide/commit/ddd47656fcca91b292108bfdfb9ad2bb049d8a0a)] Add status code modifier for IntoResponse (Wonwoo Choi)
- [[`14a13e5347`](https://github.com/rust-net-web/tide/commit/14a13e5347fa40c124b156ce9d0afaba93550c10)] Make it allocate less when running middleware chain (#110) (Wonwoo Choi)
- [[`c8e43e52ee`](https://github.com/rust-net-web/tide/commit/c8e43e52ee45c8739bd6674a61723e411cc6535c)] Update futures-preview and pin-utils (#113) (Wonwoo Choi)
- [[`23905de5f5`](https://github.com/rust-net-web/tide/commit/23905de5f501406809ade4332b073faff615ee8a)] Fix repository location in Cargo.toml (#111) (Artem Vorotnikov)
- [[`f3a9357dd1`](https://github.com/rust-net-web/tide/commit/f3a9357dd17d3888c2a907b1c8b3240db9f040fa)] Merge pull request #97 from pbvie/doc-examples (Aaron Turon)
- [[`1ed6a44cda`](https://github.com/rust-net-web/tide/commit/1ed6a44cda0815446f674596cdc724ad5ca17c46)] Add IntoResponse for Vec<u8> and Bytes (Petra Bierleutgeb)
- [[`721f9540d7`](https://github.com/rust-net-web/tide/commit/721f9540d797f0ebd373d126a7af08ed04920a7f)] Merge pull request #93 from tirr-c/catch-all (Aaron Turon)
- [[`4410f4c725`](https://github.com/rust-net-web/tide/commit/4410f4c72513b65fb2bf4e35e21a58df9acf4da7)] Run rustfmt (Wonwoo Choi)
- [[`e18e0b5066`](https://github.com/rust-net-web/tide/commit/e18e0b5066b6b93d37e8080b077446117af52293)] Rename: WildcardCountModifier -> WildcardKind (Wonwoo Choi)
- [[`0da52f993e`](https://github.com/rust-net-web/tide/commit/0da52f993e3988bd76bddf83463760fd7596c8d3)] Add docs about wildcard modifiers (Tirr)
- [[`61e96edf3b`](https://github.com/rust-net-web/tide/commit/61e96edf3bfb5fee4298165481cd53d84b696af9)] Add a tiny catch-all endpoint example (Tirr)
- [[`2e399c1974`](https://github.com/rust-net-web/tide/commit/2e399c19741e7df076e951c0c75173cd979e85b7)] path_table: Add support for wildcard count modifier (Tirr)
- [[`df9876b6c4`](https://github.com/rust-net-web/tide/commit/df9876b6c493d54c8840e01b26de2229303e261a)] Merge pull request #91 from tirr-c/multipart-features (Aaron Turon)
- [[`d87cc0c7a2`](https://github.com/rust-net-web/tide/commit/d87cc0c7a2db763ff74cabd8d985a79b1dec84db)] Update description for ServerData (Petra Bierleutgeb)
- [[`6e4d1ac3d6`](https://github.com/rust-net-web/tide/commit/6e4d1ac3d6cd7d2609d4c8a04598459ec1c7ec7e)] Add examples to API docs (Petra Bierleutgeb)
- [[`e1f5c324c4`](https://github.com/rust-net-web/tide/commit/e1f5c324c43148997f590b3f811bc24795007f82)] Merge pull request #88 from hseeberger/rename-path-component-segment (Theodore Zilist)
- [[`7b39638444`](https://github.com/rust-net-web/tide/commit/7b396384446ff1fb06c7ed87b6d69c215de017b4)] Rename path component to path segment (Heiko Seeberger)
- [[`ab1eea97ee`](https://github.com/rust-net-web/tide/commit/ab1eea97ee30442af3af65d35f56ca89f2f55625)] Merge pull request #94 from tzilist/github-templates (Aaron Turon)
- [[`08405f9b98`](https://github.com/rust-net-web/tide/commit/08405f9b983e20a425c0f261d2e76bdbde3b7b31)] add question template (Theodore Zilist)
- [[`3958ed8b1b`](https://github.com/rust-net-web/tide/commit/3958ed8b1b3a2b33d42f9ad65d1f77d1af36f2f5)] add github templates (Theodore Zilist)
- [[`a973f2758a`](https://github.com/rust-net-web/tide/commit/a973f2758a3a49fac2a4f5d712f2da32d914e3f0)] Merge pull request #92 from DeltaManiac/master (Theodore Zilist)
- [[`c704f41983`](https://github.com/rust-net-web/tide/commit/c704f41983767a1008f7059370d3c12f827f3c25)] Merge pull request #64 from tzilist/feat-default-handler (Wonwoo Choi)
- [[`2e5da53e36`](https://github.com/rust-net-web/tide/commit/2e5da53e3623b6c9f2b12bb7844ba6e75a63469f)] refactor (Theodore Zilist)
- [[`41160d83f4`](https://github.com/rust-net-web/tide/commit/41160d83f47fbe731d585b5f014bfa11c95d0bc7)] remove superflous Arc (Theodore Zilist)
- [[`f209db3386`](https://github.com/rust-net-web/tide/commit/f209db3386518a180a5f5883b9e05b0930a05614)] fix tests (Theodore Zilist)
- [[`9be6fb6bdc`](https://github.com/rust-net-web/tide/commit/9be6fb6bdc3c514ba7ff880b684fc2a1a560bae9)] change example port (Theodore Zilist)
- [[`f6cf7252e0`](https://github.com/rust-net-web/tide/commit/f6cf7252e0fb35e31388060056990eb0de2ed2b6)] chore: refactor to app and clone handler down to router (Theodore Zilist)
- [[`59897f858e`](https://github.com/rust-net-web/tide/commit/59897f858e75b8aa685c2c79e3d3923f41d75e02)] make params optional (Theodore Zilist)
- [[`7abd4e00fe`](https://github.com/rust-net-web/tide/commit/7abd4e00fea1bfa450e377f4cfe89ae193196e64)] refactor: remove unnessary trait (Theodore Zilist)
- [[`5e2b193c1d`](https://github.com/rust-net-web/tide/commit/5e2b193c1ddd7b3e9ce0c167d0da88338731b898)] set small doc on function at app level (Theodore Zilist)
- [[`325eae94b8`](https://github.com/rust-net-web/tide/commit/325eae94b8d82f83d48acbbe5a61bfb6f6bd73ac)] initial implementation of default handler completed (Theodore Zilist)
- [[`36c849877b`](https://github.com/rust-net-web/tide/commit/36c849877b80841e9a0e3456bd851e2e4c20fe85)] Added blog entry to Readme.md (DeltaManiac)
- [[`80e35bdcf0`](https://github.com/rust-net-web/tide/commit/80e35bdcf081103dd2434d292f1e991f1e3c5852)] Merge remote-tracking branch 'upstream/master' (DeltaManiac)
- [[`19d0a2df24`](https://github.com/rust-net-web/tide/commit/19d0a2df240fbe152688a68928889864f3619397)] Enable only needed features for multipart (Tirr)
- [[`c6a03eee1e`](https://github.com/rust-net-web/tide/commit/c6a03eee1e75ad080829837620b7dbb434f78598)] Merge pull request #90 from leaxoy/master (Wonwoo Choi)
- [[`5ec29a5e78`](https://github.com/rust-net-web/tide/commit/5ec29a5e78134557be675067437cb6ec32fec3c9)] fix futures build error (lixiaohui)
- [[`f7e084d3f2`](https://github.com/rust-net-web/tide/commit/f7e084d3f2a29618face710c4598d0d0f797fd12)] Merge pull request #81 from ibaryshnikov/body-types-usage-simplified (Aaron Turon)
- [[`210db7a453`](https://github.com/rust-net-web/tide/commit/210db7a453c6e245fca2fd71e8764fcb9ccf32de)] implemented DerefMut for body and query types (ibaryshnikov)
- [[`3e750752cd`](https://github.com/rust-net-web/tide/commit/3e750752cd1f903a43953a3e2c9a9559d3eeb794)] Merge pull request #76 from hseeberger/18_document_routing_syntax (Aaron Turon)
- [[`f0e82a95e5`](https://github.com/rust-net-web/tide/commit/f0e82a95e55543eba918bfdda54780ab11f761aa)] Merge pull request #80 from simonasker/extend-cookie-example (Aaron Turon)
- [[`fbcc358b3a`](https://github.com/rust-net-web/tide/commit/fbcc358b3adc1c1246b2c1cc99e23a772c51ed3e)] Merge pull request #79 from ibaryshnikov/unify-listening-ports-in-examples (Aaron Turon)
- [[`41f374b679`](https://github.com/rust-net-web/tide/commit/41f374b6798327f59c4a4d0c0c247e6c41f21ca6)] Extend cookie example (Simon Andersson)
- [[`854dbd53c7`](https://github.com/rust-net-web/tide/commit/854dbd53c773acf9d3cdeff51365fa1257c94999)] removed unused import (ibaryshnikov)
- [[`9ae1f13b8d`](https://github.com/rust-net-web/tide/commit/9ae1f13b8dbdd00945619c3695545052420b854a)] implemented Deref for body types and some Path types, updated examples (ibaryshnikov)
- [[`1d97b2c312`](https://github.com/rust-net-web/tide/commit/1d97b2c3125fbc39eb13b611639e8c9341372305)] use the same port in examples, also print the address (ibaryshnikov)
- [[`853f6204c5`](https://github.com/rust-net-web/tide/commit/853f6204c5ce6b103dddc46d332b3e4426baef5a)] Merge pull request #74 from hoodie/feature/example/computed-values (Aaron Turon)
- [[`9863cd9f82`](https://github.com/rust-net-web/tide/commit/9863cd9f826d38bc3b7ea359b8e382aa782fed2c)] Fixes and improvement (Heiko Seeberger)
- [[`cacdeffd8b`](https://github.com/rust-net-web/tide/commit/cacdeffd8b09980603cafcdba0eafb2527abee2c)] Documemnt routing syntax (Heiko Seeberger)
- [[`7c21c01bfa`](https://github.com/rust-net-web/tide/commit/7c21c01bfa6bcca9ce90ecfc107596178c7f350f)] Expose Computed Values Api (Hendrik Sollich)
- [[`0c8a132086`](https://github.com/rust-net-web/tide/commit/0c8a1320867bef6f7ede3eee1f169cc5634be960)] Merge pull request #68 from hseeberger/typo_default_headers (Aaron Turon)
- [[`181f582f1d`](https://github.com/rust-net-web/tide/commit/181f582f1dc4f39feb119467763276947b6bbf82)] Merge pull request #70 from ibaryshnikov/default-content-type-set-to-utf-8 (Aaron Turon)
- [[`c5c830d99e`](https://github.com/rust-net-web/tide/commit/c5c830d99eb814294da5fbd548d8e4f56c049f30)] Merge pull request #71 from caulagi/fix-travis-badge-2 (Aaron Turon)
- [[`3a1623d5dd`](https://github.com/rust-net-web/tide/commit/3a1623d5dd8854d2191e6a7fb8ba6b868ed52e56)] Merge pull request #72 from hoodie/feature/travis-nightly (Aaron Turon)
- [[`2c31ba5fbd`](https://github.com/rust-net-web/tide/commit/2c31ba5fbd1419d6b0b719566d8ab8d970cf01b0)] Pin a specific nightly version for travis builds (Hendrik Sollich)
- [[`8baee03649`](https://github.com/rust-net-web/tide/commit/8baee03649bc8122f15f88a3953a3f4631730171)] Fix the travis badge again (Pradip Caulagi)
- [[`ffc77f16ec`](https://github.com/rust-net-web/tide/commit/ffc77f16ec39ec90afd1ebbeec8b9974c50e399f)] set default content type for strings to utf-8 (ibaryshnikov)
- [[`cb22f4546e`](https://github.com/rust-net-web/tide/commit/cb22f4546eef2b4ed6f01b9da03f2f7b89f9cdd4)] Fix typo in default_headers example (Heiko Seeberger)
- [[`bdde7d3d69`](https://github.com/rust-net-web/tide/commit/bdde7d3d69bf0b25c5ed4edaf05defb72302aa94)] Merge remote-tracking branch 'upstream/master' (DeltaManiac)
- [[`c9999f2cd4`](https://github.com/rust-net-web/tide/commit/c9999f2cd4d67ab5b855180d574cd39314d34482)] Merge pull request #66 from caulagi/fix-travis-badge (Theodore Zilist)
- [[`9a1040f64e`](https://github.com/rust-net-web/tide/commit/9a1040f64ee1df2f17ed5875a8f4e02d780308e7)] Fix travis badge to show build status (Pradip Caulagi)
- [[`07e436ab01`](https://github.com/rust-net-web/tide/commit/07e436ab01b9a3f3a856bcc6729e5c9073a3f837)] Merge pull request #52 from bIgBV/logging-middleware (Aaron Turon)
- [[`84d1a09361`](https://github.com/rust-net-web/tide/commit/84d1a0936172a405bda6f51d9574a79ceb90151b)] Rebase master and update RootLogger to use new middleware structure (Bhargav Voleti)
- [[`7431da8787`](https://github.com/rust-net-web/tide/commit/7431da87878b43641df3e65bf5f6cf4c4117c358)] rustfmt changes (Bhargav Voleti)
- [[`89ad3148a2`](https://github.com/rust-net-web/tide/commit/89ad3148a2fc5556a2c50b7b352a2393f55af7b2)] Use data stored in Head (Bhargav Voleti)
- [[`9ac66bbf3d`](https://github.com/rust-net-web/tide/commit/9ac66bbf3d09595a2e4192e8eff6872ac88b88a2)] Rename Logger to RootLogger (Bhargav Voleti)
- [[`69ca6cd1a9`](https://github.com/rust-net-web/tide/commit/69ca6cd1a9c0f3e8dcce008e2b2287d92ebd1784)] Add basic logger to log request information. (Bhargav Voleti)
- [[`9ee7426fda`](https://github.com/rust-net-web/tide/commit/9ee7426fda8c337d2ead99c9f92612a23f31d243)] Merge pull request #59 from tirr-c/extract-url-table (Aaron Turon)
- [[`4b9b5d53ce`](https://github.com/rust-net-web/tide/commit/4b9b5d53ce2f53ce213e10974ef4bdbde594aecd)] Update .travis.yml to run tests of all crates (Wonwoo Choi)
- [[`20fec26616`](https://github.com/rust-net-web/tide/commit/20fec2661631a57e031e693cc3929b3c6b5975c8)] Fix clippy for path_table (Wonwoo Choi)
- [[`834717ef4c`](https://github.com/rust-net-web/tide/commit/834717ef4c28792b8086e3de8ca4b44222b7b3d2)] Run rustfmt (Wonwoo Choi)
- [[`93dcb5e69a`](https://github.com/rust-net-web/tide/commit/93dcb5e69a5a867cc8c242a138cd8bd32fbc8842)] Extract `url_table` into `path_table` crate (Wonwoo Choi)
- [[`ae6591aefb`](https://github.com/rust-net-web/tide/commit/ae6591aefbd74015ed336cccbaff77b8320a35b8)] Merge pull request #58 from tirr-c/around-middleware (Aaron Turon)
- [[`1b4b4474a8`](https://github.com/rust-net-web/tide/commit/1b4b4474a8990063356ca26b20733e453f7998ad)] Remove ResponseContext (Tirr)
- [[`5a16e88d9c`](https://github.com/rust-net-web/tide/commit/5a16e88d9c434a32a2f81a7e6185f703604410b9)] Get rid of ReqResMiddleware (Tirr)
- [[`e7651f8770`](https://github.com/rust-net-web/tide/commit/e7651f8770a15e6cb938f7e9836ed4c32253eede)] Add Middleware impl for closures (Tirr)
- [[`c123fe900d`](https://github.com/rust-net-web/tide/commit/c123fe900dc5b63a86bdcb0db870a01e11ccedf7)] Relieve trait bound for Middleware (Tirr)
- [[`01e4f7a58c`](https://github.com/rust-net-web/tide/commit/01e4f7a58c029ea34cfe513af362b6279b64dbf0)] Fix tests (Tirr)
- [[`94d95a4558`](https://github.com/rust-net-web/tide/commit/94d95a45584a58c9b2aab99e18b63ec185902186)] Make DefaultHeaders middleware use wrap-around middleware (Tirr)
- [[`06c01a0ba0`](https://github.com/rust-net-web/tide/commit/06c01a0ba0c2c360b1faee1d017deff75f677b12)] Experiment: Wrap-around middleware (Tirr)
- [[`8cc895f0ec`](https://github.com/rust-net-web/tide/commit/8cc895f0ecce882b9579f5626982900110387552)] Merge pull request #46 from tirr-c/subrouter (Wonwoo Choi)
- [[`092cc1e64e`](https://github.com/rust-net-web/tide/commit/092cc1e64ea248c4957df6017aa6b345b8d8f87c)] Take a slightly different approach for middleware (Wonwoo Choi)
- [[`4af918de5b`](https://github.com/rust-net-web/tide/commit/4af918de5bef30d9402d356ef504de4f1c054150)] Fix tests (Wonwoo Choi)
- [[`fe0b77cf69`](https://github.com/rust-net-web/tide/commit/fe0b77cf6986ef9de736709a4d393d17486c2dde)] Merge remote-tracking branch 'upstream/master' into subrouter (Wonwoo Choi)
- [[`6e3530c7aa`](https://github.com/rust-net-web/tide/commit/6e3530c7aa4971bb6fca3a340c8b2dddc4fe7956)] Set method of the Request for testing (Wonwoo Choi)
- [[`17f494063a`](https://github.com/rust-net-web/tide/commit/17f494063a147ac22598f01acd4eb8a40cf6d11f)] Merge pull request #57 from tirr-c/middleware-lifetime (Wonwoo Choi)
- [[`c046ce6f70`](https://github.com/rust-net-web/tide/commit/c046ce6f70e1a3036497d85b91cdca22b4cf0fb0)] Change return type of `request` to Result<(), Response> (Wonwoo Choi)
- [[`ae2416a1bb`](https://github.com/rust-net-web/tide/commit/ae2416a1bb2347552ba73fc58e6286a60d56699a)] Specify output lifetime for Server::at (Aaron Turon)
- [[`eba71f062e`](https://github.com/rust-net-web/tide/commit/eba71f062e9f1e57281a1f12779cd199b2417f41)] Merge pull request #55 from leaxoy/master (Aaron Turon)
- [[`35dd9eb908`](https://github.com/rust-net-web/tide/commit/35dd9eb9083d4161fff0644d8e6b232228b74517)] Merge pull request #47 from liufuyang/multipart-form-file-upload (Aaron Turon)
- [[`2215e2d723`](https://github.com/rust-net-web/tide/commit/2215e2d7232484d05ac02d69435a7af6a61a8dc4)] Relieve middleware FutureObj lifetime bounds (Wonwoo Choi)
- [[`cb11f3a80b`](https://github.com/rust-net-web/tide/commit/cb11f3a80b0952e8128a849289f29a206fbd1097)] Allow handel file upload via multipart form (Fuyang Liu)
- [[`764d83f43b`](https://github.com/rust-net-web/tide/commit/764d83f43b965d04a71169488f6b90fe8fe7a4f6)] Merge pull request #44 from jnicklas/form-extractors (Wonwoo Choi)
- [[`e45f6a592c`](https://github.com/rust-net-web/tide/commit/e45f6a592c66697c70f96fc58beac9b08f72bcc9)] add rest method to Resource (lixiaohui)
- [[`3c83d7ba62`](https://github.com/rust-net-web/tide/commit/3c83d7ba62eb8b22c75e034209f385aa14e99437)] Use impl trait for builder type (Wonwoo Choi)
- [[`8fe1daa530`](https://github.com/rust-net-web/tide/commit/8fe1daa530c6146b408a7102760493a4001a3e8f)] Downcase content types (Jonas Nicklas)
- [[`a2b4599f8d`](https://github.com/rust-net-web/tide/commit/a2b4599f8d3e97218cd174209c28859ccebe68cc)] Make `Resource` public (Tirr)
- [[`263cb249f8`](https://github.com/rust-net-web/tide/commit/263cb249f8b7655eb21c359ef785a643d39af234)] Document `Router` (Tirr)
- [[`fed6edc466`](https://github.com/rust-net-web/tide/commit/fed6edc466a0d363238369dbf23f12561795ad15)] Add Router tests (Tirr)
- [[`f2bc72a450`](https://github.com/rust-net-web/tide/commit/f2bc72a4505ebb863e80c03e79f31c7e7f2f7aa3)] Change middleware application method (Tirr)
- [[`06a358f95d`](https://github.com/rust-net-web/tide/commit/06a358f95dde8432bfaca55aeefa73adfd461aac)] Rename Resource to ResourceData, and make it private (Tirr)
- [[`a125f8fcea`](https://github.com/rust-net-web/tide/commit/a125f8fceaab517d8ba633f18df6a37b6a91e55c)] Add resource existence check to .nest (Wonwoo Choi)
- [[`4bb3d3f198`](https://github.com/rust-net-web/tide/commit/4bb3d3f19861d567ad319372d4ce93a7c04a7970)] Add simple nested router example (Wonwoo Choi)
- [[`dfe72abe23`](https://github.com/rust-net-web/tide/commit/dfe72abe238ad7afd642c4b546596393bd1bdade)] Add subrouter and per-endpoint middleware support (Wonwoo Choi)
- [[`a724ef3561`](https://github.com/rust-net-web/tide/commit/a724ef35610ef4074c6141e2afdd13335a56e896)] Merge pull request #42 from tzilist/feat-default-headers (Wonwoo Choi)
- [[`145911b8b2`](https://github.com/rust-net-web/tide/commit/145911b8b29ab084a903fa241c7d022d2a79b096)] update map_err now maps into http::error::Error (Theodore Zilist)
- [[`73dfd4dc26`](https://github.com/rust-net-web/tide/commit/73dfd4dc260eafcde5252eab736470302bf1a11c)] Add form data extraction from body via serde_qs (Jonas Nicklas)
- [[`1dac61cc6d`](https://github.com/rust-net-web/tide/commit/1dac61cc6d303f8508836f90d1c791756161d612)] Merge pull request #30 from csmoe/extract_query (Aaron Turon)
- [[`0894aa1aa6`](https://github.com/rust-net-web/tide/commit/0894aa1aa6a639bc879e29a4e1376dcc5300c82c)] #23 Handle HEAD requests  (#31) (Harikrishnan Menon)
- [[`30c52918c2`](https://github.com/rust-net-web/tide/commit/30c52918c2b4244a1d9190253818c1db67e36dbe)] Merge remote-tracking branch 'upstream/master' (DeltaManiac)
- [[`e25e1d2ded`](https://github.com/rust-net-web/tide/commit/e25e1d2ded3a4df1bfe2f2af5c1ffe33951729fe)] rustfmt (DeltaManiac)
- [[`a61617d90e`](https://github.com/rust-net-web/tide/commit/a61617d90e381636b33ce39a5c8dd52c2b7bfced)] Readability Enhancements (DeltaManiac)
- [[`656b5cd347`](https://github.com/rust-net-web/tide/commit/656b5cd34772ab7b34a11e27483a13b860a6d2b1)] remove clippy lint opt-out (Theodore Zilist)
- [[`6a77bd8202`](https://github.com/rust-net-web/tide/commit/6a77bd82022603e0beb89120d859a7b13ce1529d)] remove commented out code (Theodore Zilist)
- [[`dd878b4b83`](https://github.com/rust-net-web/tide/commit/dd878b4b83e20114a6464f6cd4d73524184e82dd)] change or_with statement (Theodore Zilist)
- [[`98985fd857`](https://github.com/rust-net-web/tide/commit/98985fd85739afb13d6478b4d2cb946e4c1d41bd)] make changes based on comments (Theodore Zilist)
- [[`567a627033`](https://github.com/rust-net-web/tide/commit/567a627033dcf32f2c8e72610d074d779523d3e0)] add another header in the example (Theodore Zilist)
- [[`7fb05b9276`](https://github.com/rust-net-web/tide/commit/7fb05b9276804a567170f9dff68b1ac4faeaefae)] fix lint issues (Theodore Zilist)
- [[`858b1debfa`](https://github.com/rust-net-web/tide/commit/858b1debfa1ab8d938786574fae9444af991f41b)] feat: deafault headers middleware working (Theodore Zilist)
- [[`38bb80bfd9`](https://github.com/rust-net-web/tide/commit/38bb80bfd97888d4b8c06dae3f1cabe4d740752f)] extract query from url (csmoe)
- [[`a3c6549ada`](https://github.com/rust-net-web/tide/commit/a3c6549ada591174d65ad02097870a0d74d9518d)] changed unwrap to try (DeltaManiac)
- [[`2995b4405d`](https://github.com/rust-net-web/tide/commit/2995b4405dc9b54ea28063fa04425cdc27c4b4ba)] Merge pull request #33 from tzilist/master (Aaron Turon)
- [[`48e05987ee`](https://github.com/rust-net-web/tide/commit/48e05987ee4fcccf31fb7ecf74b6125763d391cb)] Merge remote-tracking branch 'upstream/master' (DeltaManiac)
- [[`4925028091`](https://github.com/rust-net-web/tide/commit/49250280914ced146503179a41369669f8241dc5)] fix formatting (Theodore Zilist)
- [[`f5995b0499`](https://github.com/rust-net-web/tide/commit/f5995b04999b23f17186c80c058f420c38259b55)] merge upstream changes (Theodore Zilist)
- [[`04321df8a4`](https://github.com/rust-net-web/tide/commit/04321df8a4c19a9ab99763803744cd71250ddbf2)] Merge pull request #40 from tirr-c/fix-travis (Aaron Turon)
- [[`6e4ba702bd`](https://github.com/rust-net-web/tide/commit/6e4ba702bd0db72a3d20e21c2748ec5fcec4f799)] Refactor (DeltaManiac)
- [[`029db9794b`](https://github.com/rust-net-web/tide/commit/029db9794b9551d4f5e7b410dacc17f292f43f39)] Fix clippy `type_complexity` (Wonwoo Choi)
- [[`b596a3cdad`](https://github.com/rust-net-web/tide/commit/b596a3cdad98d448734bcf0f49e37327b455dc44)] Fix clippy `wrong_self_convention` (Wonwoo Choi)
- [[`ee51444564`](https://github.com/rust-net-web/tide/commit/ee514445645c56207c953bd153dcb349b035a186)] Set rustfmt edition = "2018" (Wonwoo Choi)
- [[`477875c013`](https://github.com/rust-net-web/tide/commit/477875c013f9c7dbac6f390856f6f7745ce43864)] Fix some clippy lints (Wonwoo Choi)
- [[`651c4f36ba`](https://github.com/rust-net-web/tide/commit/651c4f36ba61d6bb9a3d08da85ce6f2d2b8a2d59)] Run rustfmt (Wonwoo Choi)
- [[`a8191a0efa`](https://github.com/rust-net-web/tide/commit/a8191a0efacad8b099d8003dee0f8bf9a1fd02b5)] Fix broken CI (Wonwoo Choi)
- [[`f498f598cc`](https://github.com/rust-net-web/tide/commit/f498f598ccecf433569415efa7c7da6c7c41d8d4)] Merge pull request #39 from tirr-c/example-graphql (Aaron Turon)
- [[`66885572f8`](https://github.com/rust-net-web/tide/commit/66885572f8a8f54caa2598cbf0048ff0daebb470)] Add some comments for GraphQL example (Wonwoo Choi)
- [[`6c2b9efc52`](https://github.com/rust-net-web/tide/commit/6c2b9efc520a71c683f0c6981592f02b6ef91108)] Merge pull request #38 from tirr-c/refactor-endpoint-impl (Aaron Turon)
- [[`bfc4707419`](https://github.com/rust-net-web/tide/commit/bfc4707419f87917a939d8be8bd4573b1fcc5579)] Add simple GraphQL example (Wonwoo Choi)
- [[`aa69f86619`](https://github.com/rust-net-web/tide/commit/aa69f8661999f8bafd3b9bb08e645c2e1cdb7dda)] Simplify `end_point_impl_raw!` rules (Wonwoo Choi)
- [[`1b0d6b67df`](https://github.com/rust-net-web/tide/commit/1b0d6b67df791f98a51596ef10ee5daf4a2117e8)] Refactor `end_point_impl!` (Wonwoo Choi)
- [[`caf0119534`](https://github.com/rust-net-web/tide/commit/caf011953404ef40b90225a06fd1ccf9bb34717b)] fix formatting to 4 tabs (Theodore Zilist)
- [[`40cbc2981d`](https://github.com/rust-net-web/tide/commit/40cbc2981d3b7252a012f93a59971fecacea89fc)] chore: remove formatting (Theodore Zilist)
- [[`520ca853d7`](https://github.com/rust-net-web/tide/commit/520ca853d7914ae2580f75999e1598401ed7ad85)] refactor: add types to body parsing as well as add a lossy interpreter (Theodore Zilist)
- [[`294c3113bb`](https://github.com/rust-net-web/tide/commit/294c3113bbb55b75cc6c0e746a57cd756b8871cd)] Merge pull request #35 from cramertj/strip-whitespace (Aaron Turon)
- [[`fdcb1774ce`](https://github.com/rust-net-web/tide/commit/fdcb1774ce17db6d9d6bd18f379c498e0d05b32d)] Merge pull request #36 from rust-net-web/revert-34-executor-agnostic (Aaron Turon)
- [[`0ad90b97e6`](https://github.com/rust-net-web/tide/commit/0ad90b97e6a4c40ad6cb932482ab1f34041a0602)] Revert "Allow running Tide on non-Tokio executors" (Aaron Turon)
- [[`2cbbce4288`](https://github.com/rust-net-web/tide/commit/2cbbce4288df11dae91ca3ea76dacd9e5d5814ea)] Merge pull request #34 from cramertj/executor-agnostic (Aaron Turon)
- [[`ae6e6fa220`](https://github.com/rust-net-web/tide/commit/ae6e6fa2204ada5fd3e85dff6d06ee777f96b59f)] Strip whitespace (Taylor Cramer)
- [[`214f789856`](https://github.com/rust-net-web/tide/commit/214f789856185df7c9b091aa075b157ffbee4e00)] Allow running Tide on non-Tokio executors (Taylor Cramer)
- [[`07b0dfb05e`](https://github.com/rust-net-web/tide/commit/07b0dfb05eb4bf3cb446ed7ea89176d840653db5)] Rustfmt (DeltaManiac)
- [[`aeba74d4f5`](https://github.com/rust-net-web/tide/commit/aeba74d4f5dfe8b6d53e7d68d511553710354441)] Fallback to HTTP GET implementation if HEAD implementaion is not present (DeltaManiac)
- [[`b311fccd43`](https://github.com/rust-net-web/tide/commit/b311fccd432fa40ed67af7030aa63f603d46ff91)] chore: run rustfmt on code (Theodore Zilist)
- [[`383b581286`](https://github.com/rust-net-web/tide/commit/383b581286aa7daf087f63df98be1439d1dea3c0)] update examples (Theodore Zilist)
- [[`f2edc44fcd`](https://github.com/rust-net-web/tide/commit/f2edc44fcd3f44d33fc96b1b4e1f3fec1bcbf47c)] feat: add examples of how to extract a string or vec (Theodore Zilist)
- [[`d16c8d56fe`](https://github.com/rust-net-web/tide/commit/d16c8d56fe9f75e003347c04653ed9e5e452499a)] feat: parsing body reqeusts to String and Vec<u8> seems to be working (Theodore Zilist)
- [[`e3564d8d82`](https://github.com/rust-net-web/tide/commit/e3564d8d829f69e1df8445f4126ceb58cc37ccc6)] Merge pull request #32 from Stinners/master (Aaron Turon)
- [[`b82e58fc17`](https://github.com/rust-net-web/tide/commit/b82e58fc172d01db169aacef4a209ca1deb6609f)] Fix spelling (Chris Stinson)
- [[`bbdd369f0e`](https://github.com/rust-net-web/tide/commit/bbdd369f0e6e8eb73d9f749efdb0e28eca674588)] Named path example (Chris Stinson)
- [[`916d7778ab`](https://github.com/rust-net-web/tide/commit/916d7778ab78b476bfe6c3edea700c228a53b3cd)] Add named path extractor (Chris Stinson)
- [[`013be3c6b8`](https://github.com/rust-net-web/tide/commit/013be3c6b8facf5b2d7efbbaf1727f0a375b509a)] handle  HTTP HEAD (DeltaManiac)
- [[`967d41f6d4`](https://github.com/rust-net-web/tide/commit/967d41f6d4d0acbbcaae09a2fc7f25e38cd4823e)] Merge pull request #28 from dtolnay/slice (Aaron Turon)
- [[`b5059acfea`](https://github.com/rust-net-web/tide/commit/b5059acfea283ecf5e85648523b986061306ce37)] Merge pull request #29 from fbstj/patch-1 (Aaron Turon)
- [[`37f00da77c`](https://github.com/rust-net-web/tide/commit/37f00da77cc1320e186d8fe07d182e65567c74ee)] Update README.md (Joe ST)
- [[`6417e9e3cb`](https://github.com/rust-net-web/tide/commit/6417e9e3cb507b2a6e3dc35a8e499ceea740ea50)] Use serde_json::from_slice to deserialize from &[u8] (David Tolnay)
- [[`287acfcf35`](https://github.com/rust-net-web/tide/commit/287acfcf35334442df9402903f193c3658abce95)] accidental upload (Aaron Turon)
- [[`815c3314d6`](https://github.com/rust-net-web/tide/commit/815c3314d6480d4e4fa610d292cd5f1709210c74)] Initial implementation (Aaron Turon)
- [[`4085eed8aa`](https://github.com/rust-net-web/tide/commit/4085eed8aa9f43f6fa4e79186fb3c352117a04eb)] v0.0.0 (Yoshua Wuyts)
- [[`b67ec1b09e`](https://github.com/rust-net-web/tide/commit/b67ec1b09ec072898a2a4eb40b41477cc0e97510)] s/rise/tide (Yoshua Wuyts)
- [[`9da9a1ed0a`](https://github.com/rust-net-web/tide/commit/9da9a1ed0ad00c762168c909a9400b25197e78ca)] . (Yoshua Wuyts)

### Stats
```diff
 .github/CODE_OF_CONDUCT.md                |  75 ++++-
 .github/CONTRIBUTING.md                   |  63 +++-
 .github/ISSUE_TEMPLATE/bug_report.md      |  32 ++-
 .github/ISSUE_TEMPLATE/feature_request.md |  16 +-
 .github/ISSUE_TEMPLATE/question.md        |  13 +-
 .github/PULL_REQUEST_TEMPLATE.md          |  28 +-
 .github/stale.yml                         |  17 +-
 .gitignore                                |   7 +-
 .travis.yml                               |  12 +-
 CERTIFICATE                               |  37 ++-
 Cargo.toml                                |  40 ++-
 LICENSE-APACHE                            | 190 +++++++++-
 LICENSE-MIT                               |  21 +-
 README.md                                 |  18 +-
 examples/body_types.rs                    |  56 +++-
 examples/catch_all.rs                     |  14 +-
 examples/computed_values.rs               |  38 ++-
 examples/configuration.rs                 |  34 ++-
 examples/default_handler.rs               |  13 +-
 examples/default_headers.rs               |  17 +-
 examples/graphql.rs                       |  67 +++-
 examples/hello.rs                         |   8 +-
 examples/messages.rs                      |  85 ++++-
 examples/multipart-form/main.rs           |  73 ++++-
 examples/multipart-form/test.txt          |   1 +-
 examples/named_path.rs                    |  29 +-
 examples/simple_nested_router.rs          |  38 ++-
 rustfmt.toml                              |   2 +-
 src/app.rs                                | 271 +++++++++++++-
 src/body.rs                               | 456 ++++++++++++++++++++++-
 src/configuration/default_config.rs       |  79 ++++-
 src/configuration/mod.rs                  | 156 ++++++++-
 src/endpoint.rs                           | 181 +++++++++-
 src/extract.rs                            |  20 +-
 src/head.rs                               | 225 +++++++++++-
 src/lib.rs                                |  35 ++-
 src/middleware/default_headers.rs         |  50 ++-
 src/middleware/logger.rs                  |  44 ++-
 src/middleware/mod.rs                     |  65 +++-
 src/request.rs                            |  64 +++-
 src/response.rs                           | 124 ++++++-
 src/router.rs                             | 636 +++++++++++++++++++++++++++++++-
 42 files changed, 3450 insertions(+)
```


