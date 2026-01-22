pub mod capsules;
pub mod instructions;
pub mod vm;

// --- HEAVY MODULES (CLI ONLY) ---
// These require Bincode, Tokio, or File I/O.
// We hide them from WASM to prevent crashes.

#[cfg(feature = "cli-mode")]
pub mod codec;

#[cfg(feature = "cli-mode")]
pub mod net;

pub fn version() -> &'static str {
    "0.1.0"
}
