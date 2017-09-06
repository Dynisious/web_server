//! `http` is a module which handles the building and conversion of HTTP messages and their components.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

mod message;
pub mod start_line;
pub mod header_field;

pub use std::string::String;
pub use self::message::*;

/// The methods recognised by a HTTP [message](struct.MessageHTTP.html).
pub static HTTP_METHOD: [&'static str; 1] = ["GET"];
