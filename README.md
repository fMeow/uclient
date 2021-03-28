<!-- cargo-sync-readme start -->

 # Universal HTTP Client Interface for Rust

 `uclient` seeks to provide a unified interface for http client in rust.

[![Build Status](https://github.com/fMeow/uclient/workflows/CI%20%28Linux%29/badge.svg?branch=main)](https://github.com/fMeow/uclient/actions)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/uclient.svg)](https://crates.io/crates/uclient)
[![uclient](https://docs.rs/uclient/badge.svg)](https://docs.rs/uclient)

 Feature gates are used to conditionally enable specific http ecosystem.
 Currently reqwest(both blocking and async) and surf(async only) are
 supported out of the box.

 But it's possible to incorporate custom ecosystem. See
 `examples/custom_client.rs`.

<!-- cargo-sync-readme end -->
