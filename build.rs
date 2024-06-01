use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let proto_root = Path::new("proto");
    let proto_files = collect_proto_files(proto_root);

    let proto_paths: Vec<&str> = proto_files
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect();

    tonic_build::configure()
        .compile(&proto_paths, &[proto_root.to_str().unwrap()])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}

fn collect_proto_files(dir: &Path) -> Vec<PathBuf> {
    let mut proto_files = Vec::new();
    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            proto_files.extend(collect_proto_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            proto_files.push(path);
        }
    }
    proto_files
}
