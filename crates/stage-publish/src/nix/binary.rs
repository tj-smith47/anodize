//! Binary-format detection helpers used by the Nix publisher.
//!
//! ELF program-header parsing is intentionally minimal: we only need to
//! detect a `PT_INTERP` entry (dynamic linker), which signals that the
//! binary will need `autoPatchelfHook` on Linux.

/// Read a `u64` from a byte slice at the given offset. The slice bounds
/// are trusted at each call site (headers are pre-sized); `unwrap_or` with
/// `[0u8; 8]` prevents a panic if that ever regresses — we just return 0,
/// which the caller's downstream checks treat as a malformed ELF.
fn read_u64(bytes: &[u8], little: bool) -> u64 {
    let arr: [u8; 8] = bytes.try_into().unwrap_or([0u8; 8]);
    if little {
        u64::from_le_bytes(arr)
    } else {
        u64::from_be_bytes(arr)
    }
}

/// Read a `u32` from a byte slice at the given offset. Same fallback
/// rationale as [`read_u64`].
fn read_u32(bytes: &[u8], little: bool) -> u32 {
    let arr: [u8; 4] = bytes.try_into().unwrap_or([0u8; 4]);
    if little {
        u32::from_le_bytes(arr)
    } else {
        u32::from_be_bytes(arr)
    }
}

/// Read a `u16` from a byte slice. Same fallback rationale as [`read_u64`].
fn read_u16(bytes: &[u8], little: bool) -> u16 {
    let arr: [u8; 2] = bytes.try_into().unwrap_or([0u8; 2]);
    if little {
        u16::from_le_bytes(arr)
    } else {
        u16::from_be_bytes(arr)
    }
}

// ---------------------------------------------------------------------------
// ELF dynamic linking detection
// ---------------------------------------------------------------------------

/// Check if a binary is dynamically linked by looking for ELF PT_INTERP header.
/// Returns false for non-ELF files (macOS Mach-O, Windows PE).
pub(super) fn is_dynamically_linked(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    // Read ELF header: magic (4 bytes), class (1), data (1), version (1),
    // osabi (1), padding (8), type (2), machine (2), version (4), then
    // entry/phoff/shoff vary by class.
    let mut header = [0u8; 64]; // Enough for 64-bit ELF header
    if file.read(&mut header).unwrap_or(0) < 52 {
        return false;
    }
    // Check ELF magic
    if &header[0..4] != b"\x7fELF" {
        return false;
    }
    let is_64bit = header[4] == 2;
    let is_little_endian = header[5] == 1;

    // Parse program header offset and count
    let (phoff, phentsize, phnum) = if is_64bit {
        let phoff = read_u64(&header[32..40], is_little_endian);
        let phentsize = read_u16(&header[54..56], is_little_endian);
        let phnum = read_u16(&header[56..58], is_little_endian);
        (phoff, phentsize, phnum)
    } else {
        let phoff = read_u32(&header[28..32], is_little_endian) as u64;
        let phentsize = read_u16(&header[42..44], is_little_endian);
        let phnum = read_u16(&header[44..46], is_little_endian);
        (phoff, phentsize, phnum)
    };

    // Read program headers and look for PT_INTERP (type 3)
    use std::io::Seek;
    if file.seek(std::io::SeekFrom::Start(phoff)).is_err() {
        return false;
    }
    let mut phdr_buf = vec![0u8; phentsize as usize];
    for _ in 0..phnum {
        if file.read_exact(&mut phdr_buf).is_err() {
            return false;
        }
        let p_type = read_u32(&phdr_buf[0..4], is_little_endian);
        if p_type == 3 {
            // PT_INTERP
            return true;
        }
    }
    false
}
