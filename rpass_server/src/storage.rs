use std::path::Path;
use std::path::PathBuf;

pub struct Storage {
    path: PathBuf
}

impl Storage {
    pub fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Self {
        Storage {path: path.as_ref().to_path_buf()}
    }
}
