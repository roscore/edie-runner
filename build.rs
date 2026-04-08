//! Compile-time asset bundler with light obfuscation.
//!
//! Reads every file from `assets/gen/`, XOR-scrambles them with a rolling
//! key, concatenates into a single binary blob, and emits an index file
//! mapping filenames to (offset, length) pairs.
//!
//! At runtime, `src/assets.rs` `include_bytes!`s the blob and includes the
//! index, then decrypts on demand before handing bytes to macroquad.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Obfuscation key. NOT cryptographic — exists only to defeat casual
/// extractors that scan the wasm for recognizable PNG / WAV headers.
const KEY: &[u8] = b"EDIE_RUNNER_v1_AeiROBOT_virus_scramble_2026_PANGYO";

fn scramble(src: &[u8], offset: usize) -> Vec<u8> {
    src.iter()
        .enumerate()
        .map(|(i, b)| {
            let idx = offset + i;
            let k = KEY[idx % KEY.len()];
            b ^ k ^ ((idx as u8).wrapping_mul(31))
        })
        .collect()
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_dir = PathBuf::from("assets/gen");
    println!("cargo:rerun-if-changed=assets/gen");
    println!("cargo:rerun-if-changed=build.rs");

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    if gen_dir.exists() {
        for entry in fs::read_dir(&gen_dir).expect("read assets/gen") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if name.starts_with('.') {
                    continue;
                }
                let bytes = fs::read(&path).expect("read asset");
                entries.push((name, bytes));
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Build scrambled blob + index
    let mut blob: Vec<u8> = Vec::new();
    let mut index_src = String::from(
        "pub const ASSET_INDEX: &[(&str, usize, usize)] = &[\n",
    );
    for (name, raw) in &entries {
        let offset = blob.len();
        let scrambled = scramble(raw, offset);
        blob.extend_from_slice(&scrambled);
        index_src.push_str(&format!(
            "    (\"{}\", {}, {}),\n",
            name,
            offset,
            raw.len()
        ));
    }
    index_src.push_str("];\n");

    let blob_path = out_dir.join("asset_blob.bin");
    let index_path = out_dir.join("asset_index.rs");
    fs::write(&blob_path, &blob).expect("write blob");
    fs::write(&index_path, &index_src).expect("write index");

    println!(
        "cargo:warning=Bundled {} assets into {} bytes ({} KB)",
        entries.len(),
        blob.len(),
        blob.len() / 1024
    );
}
