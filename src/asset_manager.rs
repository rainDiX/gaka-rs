/*
* SPDX-License-Identifier: MIT
*/

use std::ffi;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NulError,
    FailedToGetCurrentDirPath,
    FailedToGetExePath,
    FailedToGetExeDir,
    DirDoesNotExist,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

pub struct AssetManager {
    root_dir: PathBuf,
}

impl AssetManager {
    pub fn new(root_dir: &str, working_directory: bool) -> Result<AssetManager, Error> {
        let root_dir = {
            if working_directory {
                let cwd_path =
                    std::env::current_dir().map_err(|_| Error::FailedToGetCurrentDirPath)?;
                log::debug!(
                    "Current working directory is {}",
                    cwd_path.to_str().unwrap()
                );
                cwd_path.join(root_dir)
            } else {
                let exe_path = std::env::current_exe().map_err(|_| Error::FailedToGetExePath)?;
                let exe_dir = exe_path.parent().ok_or(Error::FailedToGetExeDir)?;
                log::debug!("Executable directory is {}", exe_dir.to_str().unwrap());
                exe_dir.join(root_dir)
            }
        };

        #[cfg(debug_assertions)]
        log::info!(
            "Creating AssetManager with root path : {}",
            root_dir.to_str().unwrap()
        );

        if root_dir.exists() {
            Ok(AssetManager { root_dir })
        } else {
            Err(Error::DirDoesNotExist)
        }
    }

    pub fn read_string(&self, asset_path: &str) -> Result<String, Error> {
        let asset_path = self.root_dir.join(asset_path);
        #[cfg(debug_assertions)]
        log::debug!("Reading {}", asset_path.to_str().unwrap());
        let mut file = fs::File::open(asset_path)?;
        let mut buffer = String::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_cstring(&self, asset_path: &str) -> Result<ffi::CString, Error> {
        let buffer = self.read_string(asset_path)?;
        ffi::CString::new(buffer).map_err(|_| Error::NulError)
    }

    pub fn read_bytes(&self, asset_path: &str) -> Result<Vec<u8>, Error> {
        let asset_path = self.root_dir.join(asset_path);
        #[cfg(debug_assertions)]
        log::debug!("Reading {}", asset_path.to_str().unwrap());
        let mut file = fs::File::open(asset_path)?;
        let mut buffer = Vec::<u8>::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
