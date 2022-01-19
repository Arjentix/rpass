pub mod storage;

mod callbacks;
mod request_dispatcher;
mod server;
mod session;

use request_dispatcher::RequestDispatcher;
use server::Server;
use session::Session;
use std::borrow::Cow;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, RwLock};
#[mockall_double::double]
use storage::Storage;
#[macro_use]
extern crate lazy_static;

pub type AsyncStorage = Arc<RwLock<Storage>>;
pub type AsyncRequestDispatcher = Arc<RwLock<RequestDispatcher>>;

fn main() -> Result<(), anyhow::Error> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Can't open home directory"))?;
    let path = home_dir.join(".rpass_storage");

    let storage = Arc::new(RwLock::new(Storage::new(path)?));
    let pub_key = {
        let storage_read = storage.read().unwrap();
        storage_read.pub_key().to_string()
    };
    let request_dispatcher = build_request_dispatcher(storage);

    let server = Server::new("127.0.0.1:3747", pub_key, request_dispatcher)?;
    server.run();

    Ok(())
}

fn build_request_dispatcher(storage: Arc<RwLock<Storage>>) -> AsyncRequestDispatcher {
    let request_dispatcher = AsyncRequestDispatcher::default();

    {
        let register_storage = storage.clone();
        let login_storage = storage.clone();
        let confirm_login_storage = storage.clone();
        let delete_me_storage = storage;

        let mut dispatcher_write = request_dispatcher.write().unwrap();
        dispatcher_write
            .add_callback(Cow::from("register"), move |_, arg_iter| {
                callbacks::register(register_storage.clone(), arg_iter)
            })
            .add_callback(Cow::from("login"), move |session, arg_iter| {
                callbacks::login(login_storage.clone(), session, arg_iter)
            })
            .add_callback(Cow::from("confirm_login"), move |session, arg_iter| {
                callbacks::confirm_login(confirm_login_storage.clone(), session, arg_iter)
            })
            .add_callback(Cow::from("delete_me"), move |session, _| {
                callbacks::delete_me(delete_me_storage.clone(), session)
            })
            .add_callback(Cow::from("new_record"), move |session, arg_iter| {
                callbacks::new_record(session, arg_iter)
            })
            .add_callback(Cow::from("show_record"), move |session, arg_iter| {
                callbacks::show_record(session, arg_iter)
            })
            .add_callback(Cow::from("list_records"), move |session, _| {
                callbacks::list_records(session)
            })
            .add_callback(Cow::from("delete_record"), move |session, arg_iter| {
                callbacks::delete_record(session, arg_iter)
            });
    }

    request_dispatcher
}
