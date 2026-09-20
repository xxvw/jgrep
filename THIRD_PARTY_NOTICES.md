# Third-party notices

This file records notices for components that are central to a `jgrep` source
build or its default semantic-search model. It complements, and does not
replace, the license metadata in `Cargo.lock` and the license files shipped by
individual dependencies.

## Qwen2.5-0.5B-Instruct GGUF

Semantic mode can download this separately distributed model:

- Publisher/repository: [Qwen / Qwen2.5-0.5B-Instruct-GGUF](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF)
- Pinned revision: `6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e`
- File: `qwen2.5-0.5b-instruct-q8_0.gguf`
- SHA-256: `ca59ca7f13d0e15a8cfa77bd17e65d24f6844b554a7b6c12e07a5f89ff76844e`
- License: [Apache License 2.0](LICENSES/Apache-2.0.txt)
- Copyright: Qwen team / Alibaba Cloud, as stated by the upstream model card

The model is not included in this Git repository or project source archive.
It is downloaded on demand, stored in a user cache, and remains governed by
its upstream terms. Redistributors who include the model must preserve the
upstream model card, license, and notices.

## llama.cpp and Rust bindings

`jgrep` embeds llama.cpp through a Rust binding so it can run a GGUF model
locally. The resolved dependency versions and their authoritative license
metadata are locked in `Cargo.lock`. Release packaging runs the pinned
`cargo about generate` command with
[`scripts/release/cargo-about.toml`](scripts/release/cargo-about.toml) against
the locked dependency graph, then includes the resulting exact inventory and
copied license files alongside this notice.

- [llama.cpp](https://github.com/ggml-org/llama.cpp) — MIT License; copyright
  and notice information are retained from its distributed source. A copy of
  the MIT text is provided at [LICENSES/MIT.txt](LICENSES/MIT.txt).
- [llama-cpp-rs / llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) —
  license and copyright as recorded by the resolved crate release.

## Other Rust dependencies

All other Rust crates are transitive or direct build dependencies identified by
the committed `Cargo.lock`. The generated release inventory preserves their
reported license expressions and copies discovered license or notice files from
the locked dependency graph. Do not infer a dependency's terms from this
project license.

## Project relationship

The project takes conceptual inspiration from TypeSafe's description of
[System One / Jev](https://docs.typesafe.ai/concepts/system-one), but it does
not include Jev software, weights, APIs, trademarks, or documentation. No
relationship, endorsement, compatibility, or performance equivalence is
claimed.
