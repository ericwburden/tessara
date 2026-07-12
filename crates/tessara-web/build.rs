use std::{
    fs,
    path::{Path, PathBuf},
};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is available"),
    );
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("tessara-web lives beneath the workspace crates directory");

    let inputs = [
        workspace_root.join("style/main.css"),
        manifest_dir.join("assets"),
    ];
    let mut source_roots = fs::read_dir(workspace_root.join("crates"))
        .expect("workspace crates directory is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("tessara-web"))
        })
        .collect::<Vec<_>>();
    source_roots.extend(inputs);
    source_roots.sort();

    let mut files = Vec::new();
    for root in &source_roots {
        println!("cargo:rerun-if-changed={}", root.display());
        collect_files(root, &mut files);
    }
    files.sort();

    let mut hash = FNV_OFFSET;
    for file in files {
        hash_bytes(&mut hash, file.to_string_lossy().as_bytes());
        hash_bytes(
            &mut hash,
            &fs::read(&file).unwrap_or_else(|error| {
                panic!(
                    "failed to read asset fingerprint input {}: {error}",
                    file.display()
                )
            }),
        );
    }

    println!("cargo:rustc-env=TESSARA_ASSET_VERSION={hash:016x}");
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        files.push(path.to_path_buf());
        return;
    }

    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        collect_files(&entry, files);
    }
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}
