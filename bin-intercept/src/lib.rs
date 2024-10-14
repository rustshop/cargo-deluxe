use std::io;
use std::path::PathBuf;
use std::process::Command;

use thiserror::Error;
use tracing::trace;

const LOG_TARGET: &str = "bin-intercept";

#[derive(Debug, Error)]
pub enum InterceptError {
    #[error("Base name missing in the current executed binary")]
    MissingBaseName,
    #[error("Next bin to call not found: {0}")]
    FindNextBin(FindNextBinError),
    #[error("Could not call `which`: {0}")]
    Which(which::Error),
    #[error("IO Error: {0}")]
    IO(io::Error),
}

#[derive(Error, Debug)]
pub enum FindNextBinError {
    #[error("Current bin not found in PATH: {}", current_exe.display())]
    NoMatch { current_exe: PathBuf },

    #[error("No more bin found after current_bin in the PATH: {}", current_exe.display() )]
    NextBinMissing { current_exe: PathBuf },
}

pub type InterceptResult<T> = std::result::Result<T, InterceptError>;
pub type FindNextBinResult<T> = std::result::Result<T, FindNextBinError>;

pub struct Intercept {
    next_bin: PathBuf,
}

impl Intercept {
    pub fn new() -> InterceptResult<Self> {
        let current_exe = std::env::current_exe().map_err(InterceptError::IO)?;

        trace!(target: LOG_TARGET, current_exe=%current_exe.display(), "Current exe");

        let bin_basename = current_exe
            .file_name()
            .ok_or(InterceptError::MissingBaseName)?;
        let all_bins = which::which_all(bin_basename).map_err(InterceptError::Which)?;

        let next_bin =
            find_next_bin(&current_exe, all_bins).map_err(InterceptError::FindNextBin)?;
        trace!(target: LOG_TARGET, current_exe=%next_bin.display(), "Next bin");

        Ok(Self { next_bin })
    }

    pub fn intercept<F, FE>(self, f: F) -> std::result::Result<Command, FE>
    where
        F: FnOnce(&mut Command) -> std::result::Result<(), FE>,
    {
        let mut command = Command::new(self.next_bin);

        f(&mut command)?;

        Ok(command)
    }
}

fn find_next_bin(
    our_bin_path: &PathBuf,
    all_bin_paths: impl Iterator<Item = PathBuf>,
) -> FindNextBinResult<PathBuf> {
    let mut found = false;

    for path in all_bin_paths {
        if found {
            return Ok(path);
        }
        if &path == our_bin_path {
            found = true;
        }
    }

    if found {
        Err(FindNextBinError::NextBinMissing {
            current_exe: our_bin_path.clone(),
        })
    } else {
        Err(FindNextBinError::NoMatch {
            current_exe: our_bin_path.clone(),
        })
    }
}
