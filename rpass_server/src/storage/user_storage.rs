use super::{Error, Result, Key, Record};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::fs;

#[cfg(test)]
use mockall::automock;

/// Password storage of concrete user
pub struct UserStorage {
    path: PathBuf,
    pub_key: Key
}

#[cfg_attr(test, automock, allow(dead_code))]
impl UserStorage {
    /// Initializes UserDir from given `path`
    ///
    /// # Errors
    ///
    /// * UserDoesNotExists - if `path` does not exist or isn't a directory
    /// * Io - if can't read key from *path/key.pub* file
    pub(super) fn new<P: 'static + AsRef<Path>>(path: P) -> Result<Self> {
        let real_path = path.as_ref();
        if !real_path.exists() || !real_path.is_dir() {
            return Err(Error::UserDoesNotExist(
                real_path.display().to_string()));
        }

        let pub_key = Key::from_bytes(&fs::read(real_path.join("key.pub"))?)?;
        Ok(UserStorage{path: real_path.to_path_buf(), pub_key})
    }

    /// Gets user pub key
    pub fn get_pub_key(&self) -> &Key {
        &self.pub_key
    }

    /// Writes `record` into user's directory with filename `record.resource`
    ///
    /// # Errors
    ///
    /// * Io - if some error occurred during record writing
    pub fn write_record(&mut self, record: &Record)
            -> Result<()> {
        let record_file = self.path.join(&record.resource);
        fs::write(record_file, record.to_string()).map_err(|err| err.into())
    }

    /// Gets record about `resource`
    ///
    /// # Errors
    ///
    /// * Io - if some error occurred during record file reading
    /// * CantParseRecord - if can't parse record
    pub fn get_record(&self, resource: &str) -> Result<Record> {
        let record_file = self.path.join(resource);
        let record_str = fs::read_to_string(record_file)?;
        Ok(Record {
            resource: resource.to_owned(),
            .. Record::from_str(&record_str)?
        })
    }

    /// Gets list of names of all records
    ///
    /// # Errors
    ///
    /// Io - if can't read items in user directory
    pub fn list_records(&self) -> Result<Vec<String>> {
        let mut records_names = vec![];
        for entry_res in fs::read_dir(self.path.clone())? {
            let entry = entry_res?;
            let file = entry.path();
            if !file.is_file() {
                continue;
            }

            match file.file_name() {
                Some(filename) if filename != "key.pub" =>
                    records_names.push(filename.to_string_lossy().into_owned()),
                _ => ()
            }
        }
        records_names.sort();

        Ok(records_names)
    }
}
