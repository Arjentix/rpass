use std::sync::{Arc, RwLock};

use crate::storage::{Storage, Key};
use crate::request_dispatcher::ArgIter;

use std::str::FromStr;

pub fn register(storage: Arc<RwLock<Storage>>, arg_iter: ArgIter)
        -> String {
    let username = match arg_iter.next() {
        Some(username) => username,
        None => return "Error: empty username".to_string()
    };
    println!("username = \"{}\"", username);
    let key_string = match arg_iter.next() {
        Some(key_string) => key_string,
        None => return "Error: empty key".to_string()
    };
    let key = match Key::from_str(key_string) {
        Ok(key) => key,
        Err(err) => return err.to_string()
    };

    let mut storage_write = storage.write().unwrap();
    match storage_write.add_new_user(&username, &key) {
        Ok(()) => "Ok".to_string(),
        Err(err) => err.to_string()
    }
}
