use std::result::Result;
use std::str::FromStr;

/// User record with password
#[derive(Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    pub resource: String, // Resource to store password from
    pub password: String, // Password, encrypted with user public key
    pub notes: String,    // Additional notes, encrypted with user public key
}

impl Record {
    /// Creates new `Record` with all fields default-initialized
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseRecordError {
    #[error("empty string")]
    EmptyString,
}

impl FromStr for Record {
    type Err = ParseRecordError;

    /// Constructs new record from string. Expects password and notes delimited
    /// by new line character
    ///
    /// *resource* field will be set to default
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (password, notes) = s.split_once('\n').ok_or(ParseRecordError::EmptyString)?;
        Ok(Record {
            resource: String::default(),
            password: password.to_owned(),
            notes: notes.to_owned(),
        })
    }
}

impl ToString for Record {
    /// Converts record to string **without** *resource* field
    ///
    /// Password will be placed at the first line. The next lines is notes
    fn to_string(&self) -> String {
        self.password.clone() + "\n" + &self.notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let record = Record::new();
        assert_eq!(record.resource, "");
        assert_eq!(record.password, "");
        assert_eq!(record.notes, "");
    }

    #[test]
    fn test_from_str() {
        assert!(matches!(
            Record::from_str(""),
            Err(ParseRecordError::EmptyString)
        ));
        assert_eq!(
            Record::from_str("secret\nnotes\nanother notes").unwrap(),
            Record {
                resource: String::default(),
                password: "secret".to_owned(),
                notes: "notes\nanother notes".to_owned()
            }
        );
    }

    #[test]
    fn test_to_string() {
        let record = Record {
            resource: "example.com".to_owned(),
            password: "secret".to_owned(),
            notes: "some notes\nvery useful".to_owned(),
        };
        assert_eq!(record.to_string(), "secret\nsome notes\nvery useful");
    }
}
