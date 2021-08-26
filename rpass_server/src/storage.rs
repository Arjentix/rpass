use std::path::{Path, PathBuf};
use std::fs;

/// Password storage
pub struct Storage {
    path: PathBuf
}

impl Storage {
    /// Initializes storage from given path to storage folder
    /// 
    /// # Panics
    /// Panics if can't open or create storage directory
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let real_path = path.as_ref();
        Self::open_storage(real_path);

        Storage {path: real_path.to_path_buf()}
    }

    /// Open storage directory
    /// 
    /// # Panics
    /// Panics if can't open or create storage directory
    fn open_storage(path: &Path) {
        const DIRECTORY_MESSAGE_PREFIX: &'static str =
            "Rpass storage directory";

        if !path.exists() {
            println!("{} {:?} does not exist. Creating...",
                DIRECTORY_MESSAGE_PREFIX, path);
            fs::create_dir(path).unwrap();
        } else if !path.is_dir() {
            panic!("{} {:?} is not a directory. Aborting...",
                DIRECTORY_MESSAGE_PREFIX, path);
        } else {
            println!("{} is {:?}", DIRECTORY_MESSAGE_PREFIX, path);
        }
    }
}
