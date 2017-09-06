//! `message` is a module to handle complete HTTP messages.
//! It is responsible for building and converting the components of a HTTP message as one element.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

use std::string::String;
use super::header_field::*;
use super::start_line::*;

#[derive(Clone, PartialEq, Eq, Debug)]
/// A `MessageHTTP` is a representation of a HTTP message.
pub struct MessageHTTP {
    /// The first line of a HTTP message, either a `RequestLine` or a `StatusLine`. [Read more](start_line/enum.StartLine.html)
    pub start_line: StartLine,
    /// The fields of the HTTP message.
    pub header_fields: Vec<HeaderField>,
    /// The bytes making up the body of the HTTP message.
    pub message_body: Vec<u8>
}

impl MessageHTTP {
    /// Returns a new `MessageHTTP` built from the given parts.
    ///
    /// # Params
    ///
    /// start_line --- The `StartLine` for the message.</br>
    /// header_fields --- The `HeaderField`s to modify the message.</br>
    /// message_body --- The bytes which make up the message.
    pub fn new(start_line: StartLine, header_fields: Vec<HeaderField>, message_body: Vec<u8>) -> MessageHTTP {
        MessageHTTP { start_line, header_fields, message_body }
    }
    /// Returns a new `MessageHTTP` from the passed `str`.
    ///
    /// # Params
    ///
    /// msg --- The message string to convert.
    pub fn from(msg: &str) -> Result<MessageHTTP, String> {
        // Split the message based on the line termination for HTTP messages.
        let mut lines = msg.split("\r\n");
        
        // Get the start_line as the first line in the message.
        let start_line = if let Some(line) = lines.next() {
            // Convert the first line to a `StartLine`.
            match StartLine::from(line) {
                Ok(line) => line,
                Err(e) => return Err(e)
            }
        } else {
            // There was no first line in lines.
            return Err(format!("Bad Message string, no Start line: `{}`", msg));
        };
        
        // Get all the header fields from the message and convert them all.
        let fields = lines
            .clone()
            .take_while(
                |s| {
                    *s != ""
                }
            ).map(HeaderField::from);
        
        // The `Vec` of Header fields for the message.
        let mut header_fields = Vec::new();
        // Read in each of the fields.
        for field in fields {
            // If the field raised an error when getting passed raise it again.
            match field {
                Ok(hf) => header_fields.push(hf),
                Err(e) => return Err(e)
            }
        }
        
        // Skip the lines which where used for the Header fields.
        let mut lines = lines.skip(header_fields.len() + 1);
        // The `init_string` is the first part of the message body, following lines need to be appended again.
        let init_string = String::from(
            // If there is no next line then there is no message body.
            match lines.next() {
                Some(line) => line,
                None => ""
            }
        );
        // If there is no next line then there is no message body.
        let message_body = if init_string != "" {
            // Append each of the remaining lines with there seperators restored as the bytes are part of the message.
            lines.fold(
                init_string,
                |mut res, s| {
                    res.push_str("\r\n");
                    res.push_str(s);
                    res
                }
            ).into_bytes()
        } else {
            // There is no body and therefore there is no bytes.
            init_string.into_bytes()
        };
        
        Ok(MessageHTTP::new(start_line, header_fields, message_body))
    }
    /// Returns a new `MessageHTTP` from the passed bytes.
    ///
    /// # Params
    ///
    /// msg --- The message string to convert.
    pub fn from_utf8(msg: Vec<u8>) -> Result<MessageHTTP, String> {
        match String::from_utf8(msg) {
            Ok(s) => MessageHTTP::from(s.as_str()),
            Err(_) => Err(String::from("Bad bytes for utf8 encoded message."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_http() {
        assert_eq!(
            MessageHTTP::from("http/1.1 200 OK\r\n name : value \r\n taste : smell \r\n\r\n The red fox jumped\r\nover the lazy dog").unwrap(),
            MessageHTTP {
                start_line: StartLine::StatusLine {
                    version: String::from("HTTP/1.1"),
                    code: 200,
                    reason: Some(String::from("OK"))
                },
                header_fields: vec![
                    HeaderField {
                        name: String::from("name"),
                        value: String::from("value")
                    },
                    HeaderField {
                        name: String::from("taste"),
                        value: String::from("smell")
                    }
                ],
                message_body: String::from(" The red fox jumped\r\nover the lazy dog").into_bytes()
            },
            "Test MessageHTTP::from-1 failed."
        );
        
        assert_eq!(
            MessageHTTP::from("http/1.1 200 OK\r\n name : value \r\n taste : smell \r\n\r\n").unwrap(),
            MessageHTTP {
                start_line: StartLine::StatusLine {
                    version: String::from("HTTP/1.1"),
                    code: 200,
                    reason: Some(String::from("OK"))
                },
                header_fields: vec![
                    HeaderField {
                        name: String::from("name"),
                        value: String::from("value")
                    },
                    HeaderField {
                        name: String::from("taste"),
                        value: String::from("smell")
                    }
                ],
                message_body: String::from("").into_bytes()
            },
            "Test MessageHTTP::from-2 failed."
        );
        
        assert_eq!(
            MessageHTTP::from("http/1.1 200\r\n name : value \r\n taste : smell \r\n\r\n").unwrap(),
            MessageHTTP {
                start_line: StartLine::StatusLine {
                    version: String::from("HTTP/1.1"),
                    code: 200,
                    reason: None
                },
                header_fields: vec![
                    HeaderField {
                        name: String::from("name"),
                        value: String::from("value")
                    },
                    HeaderField {
                        name: String::from("taste"),
                        value: String::from("smell")
                    }
                ],
                message_body: String::from("").into_bytes()
            },
            "Test MessageHTTP::from-3 failed."
        );
        
        assert_eq!(
            MessageHTTP::from("get / http/1.1\r\n name : value \r\n taste : smell \r\n\r\n").unwrap(),
            MessageHTTP {
                start_line: StartLine::RequestLine {
                    method: "GET",
                    target: String::from("/"),
                    version: String::from("HTTP/1.1")
                },
                header_fields: vec![
                    HeaderField {
                        name: String::from("name"),
                        value: String::from("value")
                    },
                    HeaderField {
                        name: String::from("taste"),
                        value: String::from("smell")
                    }
                ],
                message_body: String::from("").into_bytes()
            },
            "Test MessageHTTP::from-4 failed."
        );
        
        assert_eq!(
            MessageHTTP::from("get / http/1.1\r\n name : value \r\n taste : smell \r\n\r\n The quick brown fox\r\njumped over the lazy dog.").unwrap(),
            MessageHTTP {
                start_line: StartLine::RequestLine {
                    method: "GET",
                    target: String::from("/"),
                    version: String::from("HTTP/1.1")
                },
                header_fields: vec![
                    HeaderField {
                        name: String::from("name"),
                        value: String::from("value")
                    },
                    HeaderField {
                        name: String::from("taste"),
                        value: String::from("smell")
                    }
                ],
                message_body: String::from(" The quick brown fox\r\njumped over the lazy dog.").into_bytes()
            },
            "Test MessageHTTP::from-5 failed."
        );
    }
}
