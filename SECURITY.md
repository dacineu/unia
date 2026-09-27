# Security Policy

unia is a pre-1.0 research implementation. It is not hardened, and it should not
be exposed to untrusted input in production.

## Supported versions

| Version | Supported |
| --- | --- |
| `main` (0.1.x) | Yes |
| Anything before `0.1.0` | No |

unia has not reached a stable release. There is no long-term-support branch and
no backport policy.

## Reporting a vulnerability

**Do not open a public issue, discussion, or pull request for a security
problem.** Public reports give an attacker a head start and tend to force an
uncoordinated fix.

Email **<dacineu@proton.me>** with:

- what the issue is and which component is affected
- steps to reproduce, ideally a minimal `.ure` manifest and command
- the impact you believe it has
- any suggested fix

You will get an acknowledgement within a few days. Please give reasonable time
for a fix to be prepared before disclosing publicly.

## What to look for

This project parses untrusted input by design, and the interesting boundaries
are:

- **`.ure` manifest parsing.** Manifests are deserialised from JSON on the hot
  path in `registry::ActuatorRegistry::resolve_actuator` and
  `ActuatorRegistry::register_ure_file`. Treat any manifest from outside the
  trust boundary as hostile.
- **Resource identifiers.** Identifiers are used to build filesystem paths. See
  `docs/SPEC.md` for the `ure_RES_CAT_LOC_UNIQ` scheme and check that path
  components cannot escape their intended directory.
- **The economic and permission layer.** `src/wmis/economy.rs` gates actuation
  on a token balance and a permission engine. A bypass of either is in scope.
- **Encryption.** `register_ure_file` accepts an optional 32-byte key for
  identifier derivation. See the dedicated warning below.
- **Wasm surface.** `src/wasm_core.rs` and the `web/` build run in a browser
  context with different trust assumptions than the native build.

## Known weakness: fixed AES-GCM nonce in identifier derivation

`DuUuid::encrypt_payload` in `src/identifiers/mod.rs` encrypts the manifest with
AES-256-GCM using the constant 12-byte nonce `b"ure_det_nonc"`.

The nonce is fixed on purpose. It has to be: the operation must map identical
content to an identical identifier, and a random nonce would defeat that. The
consequence is that this is **not a confidentiality mechanism**. GCM with a
reused nonce leaks the XOR of any two plaintexts encrypted under the same key,
and it does not provide semantic security.

This is acceptable for what it is used for today, generating a stable identifier
where secrecy is not claimed. It is not acceptable for anything else. In
particular:

- **Do not treat `.ure` identifiers as concealing manifest content.** They do
  not. Anyone with the key can recover plaintext, and someone without the key
  can still exploit the nonce reuse.
- **Do not add a feature that encrypts a manifest with this path and then
  relies on the ciphertext being secret.**
- The key is supplied by the caller and there is no key management, rotation,
  or validation in this repository.

If deterministic encryption is genuinely needed, derive the nonce from a hash of
the plaintext or from a per-manifest secret, and document the trade-off. If
confidentiality is needed, use a deterministic AEAD with a properly derived
nonce, or accept that the identifier is public and encrypt the content
separately.


## Threat model assumptions

unia assumes:

- the host operating system is trusted
- the Rust toolchain and dependency tree are not compromised
- manifests loaded from the configured base directory are as trustworthy as
  whoever can write there

It does **not** assume the `.ure` content itself is safe. That is the boundary
worth hardening first.
