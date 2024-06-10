use candid::Principal;
use ic_cdk::caller;

use crate::types::error::Error;

use super::store::Store;

pub fn is_not_anonymous() -> Result<(), String> {
    match caller() == Principal::anonymous() {
        true => Err(Error::unauthorized()
            .add_message("Anonymous caller not allowed.")
            .to_string()),
        false => Ok(()),
    }
}

pub fn is_dev() -> Result<(), String> {
    let dev =
        Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
            .unwrap();
    match caller() == dev {
        true => Err(Error::unauthorized()
            .add_message("unknown caller")
            .to_string()),
        false => Ok(()),
    }
}

pub fn is_known_wallet() -> Result<(), String> {
    match Store::get_wallet(caller()).is_err() {
        true => Err(Error::unauthorized()
            .add_message("Unknown wallet")
            .to_string()),
        false => Ok(()),
    }
}
