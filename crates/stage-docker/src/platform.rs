// ---------------------------------------------------------------------------
// platform_to_arch
// ---------------------------------------------------------------------------

/// Extract the architecture component from a Docker platform string.
///
/// Handles three-component platform strings like `"linux/arm/v7"` by
/// concatenating the arch and variant (e.g. `"armv7"`), which matches
/// the output of [`map_target`] for armv7/armv6 Rust triples.
///
/// Examples:
/// - `"linux/amd64"` → `"amd64"`
/// - `"linux/arm64"` → `"arm64"`
/// - `"linux/arm/v7"` → `"armv7"`
/// - `"linux/arm/v6"` → `"armv6"`
pub fn platform_to_arch(platform: &str) -> &str {
    let parts: Vec<&str> = platform.split('/').collect();
    match parts.as_slice() {
        [_, arch, variant] => {
            // For "linux/arm/v7" → "armv7", "linux/arm/v6" → "armv6"
            // We need static strings since the return type is &str.
            match (*arch, *variant) {
                ("arm", "v6") => "armv6",
                ("arm", "v7") => "armv7",
                _ => variant,
            }
        }
        [_, arch] => arch,
        _ => platform,
    }
}

// ---------------------------------------------------------------------------
// tag_suffix
// ---------------------------------------------------------------------------

/// Extract the architecture portion of a platform string for use as a tag suffix.
///
/// Delegates to [`platform_to_arch`] since the logic is identical:
/// - `"linux/amd64"` → `"amd64"`
/// - `"linux/arm64"` → `"arm64"`
/// - `"linux/arm/v7"` → `"armv7"`
pub(crate) fn tag_suffix(platform: &str) -> String {
    platform_to_arch(platform).to_string()
}
