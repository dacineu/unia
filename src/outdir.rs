//! Where unia writes the files it synthesises.
//!
//! Manifests used to be written to the process working directory, which had two
//! costs that turned out to be the same cost. Running the test suite littered
//! the repository root with one `.ure` per synthesis, and the corpus measured
//! locally was therefore not the corpus a cloner gets: 63 patterns here against
//! 12 in a fresh clone, on the same commit.
//!
//! Synthesis output now lands in one gitignored directory that stays inside the
//! store's walk, so a synthesised actuator is still part of the corpus it was
//! added to. The relative location is configurable rather than fixed, because
//! the caller that knows where its corpus lives is not always the process that
//! synthesises into it.

use std::path::PathBuf;

/// Directory that synthesised `.ure` manifests are written into.
///
/// Defaults to `.unia/out` beneath the working directory, and honours
/// `UNIA_OUT_DIR` when set. The directory is created on demand: a write that
/// silently fails because a directory is missing is worse than the litter this
/// exists to remove.
pub fn out_dir() -> PathBuf {
    let dir = match std::env::var_os("UNIA_OUT_DIR") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(".unia").join("out"),
    };
    if !dir.exists() {
        // A concurrent synthesis can win the race, so a failed create is not
        // itself an error: the `fs::write` that follows reports the real
        // problem, and names the directory that could not be created.
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Full path for a synthesised manifest, creating the output directory first.
///
/// Takes the resource id rather than a file name so no caller can forget the
/// extension, which is the other way a manifest ends up somewhere unexpected.
pub fn manifest_path(resource_id: &str) -> PathBuf {
    out_dir().join(format!("{resource_id}.ure"))
}
