use std::fs;
use std::path::{Path, PathBuf};
use std::{env, io};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let target_dir = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").unwrap());

    recursive_copy(&manifest_dir.join("assets"), &target_dir).expect("Failed to write ressources");
}

fn locate_target_dir_from_output_dir(dir: &Path) -> Option<&Path> {
    if dir.ends_with("target") {
        return Some(dir);
    }
    match dir.parent() {
        Some(path) => locate_target_dir_from_output_dir(path),
        None => None,
    }
}

fn recursive_copy(from: &Path, to: &Path) -> io::Result<()> {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.join(from_path.file_name().unwrap());
    eprintln!("from : {}", from_path.to_str().unwrap());
    if from.is_dir() {
        fs::create_dir_all(&to_path)?;
        for entry in fs::read_dir(from_path.clone())? {
            let entry = entry?;
            
            if entry.file_type()?.is_dir() {
                recursive_copy(&entry.path(), &to_path)?;
            } else {
                let target_path = to_path.join(entry.file_name());
                eprintln!("copying {}", target_path.to_str().unwrap());
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    } else {
        fs::copy(from, to_path).expect("failed to copy");
    }
    Ok(())
}
