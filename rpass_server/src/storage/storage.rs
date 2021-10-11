use super::Key;
use super::Record;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::string::ToString;
use std::str::FromStr;

#[cfg(test)]
use mockall::automock;

const PUB_KEY_FILENAME: &'static str = "key.pub";

/// Password storage
pub struct Storage {
    path: PathBuf,
    pub_key: Key,
    sec_key: Key
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),

    #[error("Storage path {0} is not a directory")]
    StoragePathIsNotADirectory(PathBuf),

    #[error("user {0} already exists")]
    UserAlreadyExists(String),

    #[error("user {0} doesn't exist")]
    UserDoesNotExist(String),

    #[error("record parsing error: {0}")]
    RecordParsingError(#[from] <Record as FromStr>::Err)
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg_attr(test, automock, allow(dead_code))]
impl Storage {
    /// Initializes storage from given path to storage folder
    /// 
    /// # Errors
    /// 
    /// Any possible error during file/directory opening/writing
    pub fn from_path<P: 'static + AsRef<Path>>(path: P) -> Result<Self> {
        let real_path = path.as_ref();
        Self::open_storage(real_path)?;

        let (pub_key, sec_key) = Self::read_keys(real_path)?;

        Ok(Storage {path: real_path.to_path_buf(), pub_key, sec_key})
    }

    /// Adds new user to the storage
    /// 
    /// Creates user folder with name `username` ans *key.pub* file with
    /// `pub_key` content. Makes no `username` validation
    /// 
    /// # Errors
    /// 
    /// Any errors during creating folder and writing file
    pub fn add_new_user(&mut self, username: &str, pub_key: &Key)
            -> Result<()> {
        let user_dir = self.path.join(username);
        let pub_key_file = user_dir.join(PUB_KEY_FILENAME);
        fs::create_dir(user_dir)
            .map_err(|_| Error::UserAlreadyExists(username.to_owned()))?;
        fs::write(pub_key_file, pub_key.as_bytes()).map_err(|err| err.into())
    }

    /// Deletes user's files and directory
    /// 
    /// # Errors
    /// 
    /// See [`std::fs::remove_dir_all()`]
    pub fn delete_user(&mut self, username: &str) -> Result<()> {
        fs::remove_dir_all(self.path.join(username)).map_err(|err| err.into())
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
        Key::from_bytes(&fs::read(pub_key_file)?).map_err(|err| err.into())
    }

    /// Writes `record` into `username` directory with filename
    /// `record.resource`
    /// 
    /// # Errors
    /// 
    /// Any error during file writing
    pub fn write_record(&mut self, username: &str, record: &Record)
            -> Result<()> {
        let user_dir = self.get_user_dir(username)?;

        let record_file = user_dir.join(&record.resource);
        fs::write(record_file, record.to_string()).map_err(|err| err.into())
    }

    /// Gets record about `resource` from `username` directory
    pub fn get_record(&self, username: &str, resource: &str) -> Result<Record> {
        let user_dir = self.get_user_dir(username)?;

        let record_file = user_dir.join(resource);
        let record_str = fs::read_to_string(record_file)?;
        Ok(Record {
            resource: resource.to_owned(),
            .. Record::from_str(&record_str)?
        })
    }

    /// Gets storage public key
    pub fn get_pub_key(&self) -> &Key {
        &self.pub_key
    }

    /// Gets storage secret key
    pub fn get_sec_key(&self) -> &Key {
        &self.sec_key
    }

    /// Gets user directory, performing checking
    fn get_user_dir(&self, username: &str) -> Result<PathBuf> {
        let user_dir = self.path.join(username);
        if !user_dir.is_dir() {
            return Err(Error::UserDoesNotExist(username.to_owned()));
        }
        Ok(user_dir)
    }

    /// Open storage directory
    /// 
    /// # Errors
    /// 
    /// Any possible error during file/directory opening/writing
    fn open_storage(path: &Path) -> Result<()> {
        const DIRECTORY_MESSAGE_PREFIX: &'static str =
            "Rpass storage directory";

        if !path.exists() {
            println!("{} {:?} does not exist. Creating...",
                DIRECTORY_MESSAGE_PREFIX, path);
            fs::create_dir(path)?;
            return Self::init_keys(path);
        } else if !path.is_dir() {
            return Err(
                Error::StoragePathIsNotADirectory(path.to_owned())
            );
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
        fs::write(path.join("key.pub"), pub_key.as_bytes())?;
        fs::write(path.join("key.sec"), sec_key.as_bytes())?;
        Ok(())
    }

    /// Reads public and secret keys from files *key.pub* and *key.sec*
    /// 
    /// # Errors
    /// 
    /// Any possible error during files reading and keys constructing
    fn read_keys(path: &Path) -> Result<(Key, Key)> {
        let pub_key = Key::from_bytes(&fs::read(path.join("key.pub"))?)?;
        let sec_key = Key::from_bytes(&fs::read(path.join("key.sec"))?)?;
        Ok((pub_key, sec_key))
    }
}