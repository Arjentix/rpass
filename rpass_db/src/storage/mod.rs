pub use error::Error;
pub use rpass::key::{self, Key};
pub use rpass::record::*;
#[mockall_double::double]
pub use user_storage::UserStorage;

mod error;
mod user_storage;

pub type Result<T> = std::result::Result<T, Error>;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Weak};

#[cfg(test)]
use mockall::automock;

const PUB_KEY_FILENAME: &str = "key.pub";

type WeakUserStorage = Weak<RwLock<UserStorage>>;

/// Record storage of all users
pub struct Storage {
    path: PathBuf,
    pub_key: Key,
    sec_key: Key,
    username_to_user_storage: HashMap<String, WeakUserStorage>,
}

#[cfg_attr(test, automock, allow(dead_code))]
impl Storage {
    /// Initializes storage from given path to storage folder
    ///
    /// # Errors
    ///
    /// Any possible error during file/directory opening/writing
    pub fn new<P: 'static + AsRef<Path>>(path: P) -> Result<Self> {
        let real_path = path.as_ref();
        Self::open_storage(real_path)?;

        let (pub_key, sec_key) = Self::read_keys(real_path)?;

        Ok(Storage {
            path: real_path.to_path_buf(),
            pub_key,
            sec_key,
            username_to_user_storage: HashMap::new(),
        })
    }

    /// Adds new user to the storage
    ///
    /// Creates user folder with name `username` ans *key.pub* file with
    /// `pub_key` content. Makes no `username` validation
    ///
    /// # Errors
    ///
    /// Any errors during creating folder and writing file
    pub fn add_new_user(&mut self, username: &str, pub_key: &Key) -> Result<()> {
        let user_dir = self.path.join(username);
        let pub_key_file = user_dir.join(PUB_KEY_FILENAME);
        fs::create_dir(user_dir).map_err(|_| Error::UserAlreadyExists(username.to_owned()))?;
        pub_key
            .write_to_file(pub_key_file)
            .map_err(|err| err.into())
    }

    /// Deletes user's files and directory
    /// There should be no any Arc on `username` user storage
    ///
    /// # Errors
    ///
    /// * UnsupportedActionForMultiSession -- if there are some active sessions
    /// of given user
    /// * Io -- if any error occurred during [`std::fs::remove_dir_all()`]
    pub fn delete_user(&mut self, username: &str) -> Result<()> {
        if let Some(weak) = self.username_to_user_storage.get(username) {
            if weak.strong_count() > 0 {
                return Err(Error::UnsupportedActionForMultiSession);
            }
        };

        self.username_to_user_storage.remove(username);
        fs::remove_dir_all(self.path.join(username)).map_err(|err| err.into())
    }

    /// Gets UserStorage struct for user with name `username`
    ///
    /// # Errors
    ///
    /// See [`UserStorage::new()`]
    pub fn get_user_storage(&mut self, username: &str) -> Result<Arc<RwLock<UserStorage>>> {
        if let Some(weak) = self.username_to_user_storage.get(username) {
            if weak.strong_count() > 0 {
                return Ok(weak.upgrade().unwrap());
            }
        };

        let user_dir_path = self.path.join(username);
        let user_storage = Arc::new(RwLock::new(UserStorage::new(user_dir_path)?));
        self.username_to_user_storage
            .insert(username.to_owned(), Arc::downgrade(&user_storage));

        Ok(user_storage)
    }

    /// Reads and returns user public key
    ///
    /// # Errors
    ///
    /// Any error during file reading
    pub fn get_user_pub_key(&self, username: &str) -> Result<Key> {
        let pub_key_file = self.path.join(username).join(PUB_KEY_FILENAME);
        if !pub_key_file.exists() {
            return Err(Error::UserDoesNotExist(username.to_owned()));
        }
        Key::from_file(pub_key_file).map_err(|err| err.into())
    }

    /// Gets storage public key
    pub fn pub_key(&self) -> &Key {
        &self.pub_key
    }

    /// Gets storage secret key
    pub fn sec_key(&self) -> &Key {
        &self.sec_key
    }

    /// Open storage directory
    ///
    /// # Errors
    ///
    /// Any possible error during file/directory opening/writing
    fn open_storage(path: &Path) -> Result<()> {
        const DIRECTORY_MESSAGE_PREFIX: &str = "Rpass storage directory";

        if !path.exists() {
            println!(
                "{} {:?} does not exist. Creating...",
                DIRECTORY_MESSAGE_PREFIX, path
            );
            fs::create_dir(path)?;
            return Self::init_keys(path);
        } else if !path.is_dir() {
            return Err(Error::StoragePathIsNotADirectory(path.to_owned()));
        }

        println!("{} is {:?}", DIRECTORY_MESSAGE_PREFIX, path);
        Ok(())
    }

    /// Creates public and secret keys and write them to the files *key.pub*
    /// and *key.sec*
    ///
    /// # Errors
    ///
    /// Any possible error during files writing
    fn init_keys(path: &Path) -> Result<()> {
        let (pub_key, sec_key) = Key::generate_pair();
        pub_key.write_to_file(path.join("key.pub"))?;
        sec_key
            .write_to_file(path.join("key.sec"))
            .map_err(|err| err.into())
    }

    /// Reads public and secret keys from files *key.pub* and *key.sec*
    ///
    /// # Errors
    ///
    /// Any possible error during files reading and keys constructing
    fn read_keys(path: &Path) -> Result<(Key, Key)> {
        let pub_key = Key::from_file(path.join("key.pub"))?;
        let sec_key = Key::from_file(path.join("key.sec"))?;
        Ok((pub_key, sec_key))
    }
}
