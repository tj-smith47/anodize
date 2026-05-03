use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context as _, Result};

use anodizer_core::artifact::{Artifact, ArtifactKind};
use anodizer_core::config::{
    BuildConfig, BuildIgnore, BuildOverride, CrossStrategy, HookEntry, UniversalBinaryConfig,
};
use anodizer_core::context::Context;
use anodizer_core::env_expand::expand_env as expand_env_vars;
use anodizer_core::hooks::run_hooks;
use anodizer_core::stage::Stage;
use anodizer_core::target::map_target;
use anodizer_core::util::find_binary;

pub mod binstall;
pub mod version_sync;

// ---------------------------------------------------------------------------
// BuildCommand — a description of the command to run
// ---------------------------------------------------------------------------

mod command;
pub use command::*;

// ---------------------------------------------------------------------------
// detect_cargo_profile — parse --release / --profile flags from cargo flags
// ---------------------------------------------------------------------------

mod profile;
pub(crate) use profile::*;

// ---------------------------------------------------------------------------
// build_universal_binary — run `lipo` to combine arm64 + x86_64 macOS binaries
// ---------------------------------------------------------------------------

mod universal;
pub(crate) use universal::*;

// ---------------------------------------------------------------------------
// Build ignore/override helpers
// ---------------------------------------------------------------------------

mod targets;
pub(crate) use targets::*;

// ---------------------------------------------------------------------------
// strip_glibc_suffix — strip glibc version suffix like ".2.17" from targets
// ---------------------------------------------------------------------------

mod validation;
pub use validation::*;

// ---------------------------------------------------------------------------
// check_workspace_package — validate --package flag for workspace crates
// ---------------------------------------------------------------------------

mod workspace;
pub(crate) use workspace::*;

// ---------------------------------------------------------------------------
// BuildStage
// ---------------------------------------------------------------------------

pub struct BuildStage;

mod run;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests;
