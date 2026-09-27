# Contributing to unia

Thank you for considering a contribution. This document covers how to get
changes merged and the legal agreement each contribution carries.

## The short version

1. Open an issue before writing anything substantial, so we agree on the approach.
2. Make the change on a branch, with tests.
3. Sign off every commit with the [Developer Certificate of Origin](#developer-certificate-of-origin-dco).
4. Open a pull request.

## Why we ask for a DCO sign-off

unia is MIT licensed. The sign-off line is not ceremony; it is what lets the
project keep its options open.

Each contributor retains copyright in their own work. The DCO confirms you have
the right to contribute it, and that you agree the contribution is provided
under the project's terms. It does **not** assign your copyright to the public
domain, and it is less demanding on contributors than a Contributor License
Agreement, which is why it is used here.

Without a sign-off, the project cannot safely relicense, dual-license, or sell
the codebase later. A single unsigned external contribution permanently removes
that option. If that matters to you, the alternative is to avoid signing and not
contribute code, which is entirely your right.

The author and maintainers of this repository sign off every commit. You can
verify that with:

```sh
git log --format='%H %an <%ae> %s%n%b' | grep -B1 'Signed-off-by:'
```

## Developer Certificate of Origin (DCO)

Every commit must carry a `Signed-off-by:` trailer naming you:

```
Fix intent mapping for underscore-form action identifiers

The bridge normalises action ids to spaces before containment matching, so
an intent phrased with underscores only resolves through aliases.

Signed-off-by: Your Name <you@example.com>
```

The easy way, which adds the trailer for you:

```sh
git commit -s -m "Your commit message"
```

The full text is in [`DCO`](./DCO).

## Development

Requires a stable Rust toolchain. The crate targets 1.75 or later.

```sh
git clone https://github.com/dacineu/unia
cd unia
cargo build
cargo test              # 17 tests across 14 binaries
cargo check --all-targets
cargo run --example full_system_demo
```

`cargo check --all-targets` covers tests and examples as well as the library. It
must be clean before you open a pull request, and it currently emits warnings
from a backlog of unrelated work.

### Examples worth reading

| Example | Shows |
| --- | --- |
| `full_system_demo` | Provisioning, orchestration, SLM execution and learning end to end |
| `verify_fs_pipeline` | The Bridge → Nucleus path with economic accounting |
| `formal_verification_demo` | Property-based verification of the actuation contract |
| `universal_demo` | Multiple resource categories through one pipeline |

### The `.ure` format

`.ure` files are JSON. `registry::ActuatorRegistry::with_base_dir` sets the
directory manifests are resolved from; the default is the process working
directory, which makes tests that provision into a temporary directory fail
confusingly. Prefer `with_base_dir` in new code.

The normative description of the format is [`docs/SPEC.md`](./docs/SPEC.md).

## Commit and PR conventions

- One logical change per commit.
- Explain *why* in the commit body. The diff already shows *what*.
- Reference the issue the change closes.
- Add or update a test for every behavioural change. A bug fix without a
  regression test will be asked for one.
- Keep the existing module boundaries. `src/bridge`, `src/nucleus`,
  `src/registry`, `src/orchestrator` and `src/wmis` have distinct
  responsibilities; do not merge them for convenience.

## Reporting security issues

Do not open a public issue for a vulnerability. See [`SECURITY.md`](./SECURITY.md).

## Licensing

Contributions are accepted under the MIT license in [`LICENSE`](./LICENSE), with
the DCO sign-off described above. If you need your contribution under different
terms, or you need a commercial license, contact <dacineu@proton.me> before
sending a pull request.

## Code of Conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
