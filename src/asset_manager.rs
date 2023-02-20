/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NulError,
    FailedToGetExePath,
    FailedToGetExeDir,
    DirDoesNotExist,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

pub struct AssetsManager {
    root_dir: PathBuf,
}

impl AssetsManager {
    pub fn new(root_dir: &str) -> Result<AssetsManager, Error> {
        let exe_path = std::env::current_exe().map_err(|_| Error::FailedToGetExePath)?;
        let exe_dir = exe_path.parent().ok_or(Error::FailedToGetExeDir)?;
        let root_dir = exe_dir.join(root_dir);

        #[cfg(debug_assertions)]
        eprintln!("Creating AssetManager with root path : {}", root_dir.to_str().unwrap());

        if root_dir.exists() {
            Ok(AssetsManager {
                root_dir: root_dir })
        } else {
            Err(Error::DirDoesNotExist)
        }
    }

    pub fn read_cstring(&self, asset_path: &str) -> Result<ffi::CString, Error> {
        let asset_path = self.root_dir.join(asset_path);
        #[cfg(debug_assertions)]
        eprintln!("Reading {}", asset_path.to_str().unwrap());
        let mut file = fs::File::open(asset_path)?;
        let mut buffer = String::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_string(&mut buffer)?;

        ffi::CString::new(buffer).map_err(|_| Error::NulError)
    }
}
