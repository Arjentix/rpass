use std::path::{Path, PathBuf};
use std::fs;
use rpass::key::Key;

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

    /// Adds new user to the storage
    /// 
    /// Creates user folder with name `username` ans *key.pub* file with
    /// `pub_key` content
    pub fn add_new_user(&mut self, username: &String, pub_key: &Key)
            -> std::io::Result<()> {
        let user_dir = self.path.join(username);
        let pub_key_file = user_dir.join("key.pub");
        fs::create_dir(user_dir)?;
        fs::write(pub_key_file, pub_key.as_bytes())
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
