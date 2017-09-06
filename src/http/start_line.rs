//! `start_line` is a module to handle the Start line of a HTTP message and their components.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

use std::string::String;
use super::HTTP_METHOD;

#[derive(Clone, PartialEq, Eq, Debug)]
/// A `StartLine` is the first line of a HTTP message defining how the message should be treated.
pub enum StartLine {
    /// A `RequestLine` defines some action to be taken by the recipient.
    RequestLine {
        /// The `method` denoted by the request.
        method: &'static str,
        /// The resource target to perform the `method` on.
        target: String,
        /// The HTTP version of this message.
        version: String
    },
    /// A `StatusLine` is a response to a request message.
    StatusLine {
        /// The HTTP version of this message.
        version: String,
        /// The response code associated with this message.
        code: u32,
        /// The optional reason given for the response.
        reason: Option<String>
    }
}

impl StartLine {
    /// Converts the passed `str` to a `StartLine`.
    ///
    /// # Params
    ///
    /// msg --- The `str` to convert to a `StartLine`.
    pub fn from(msg: &str) -> Result<StartLine, String> {
        // Get the parts of the string, attempting to divide by either spaces or quotes.
        let parts: Vec<&str> = {
            // Split the string on quotes.
            let quot_split: Vec<&str> = msg.trim().split("\"").collect();
            
            // If the string is divided into three parts then this is a valid split.
            if quot_split.len() == 3 {
                // Return the split message.
                quot_split
            } else {
                // Split the string on spaces.
                msg.trim().split(" ").collect::<Vec<&str>>()
            }
        };
        
        // The first_part of the line should always be uppercase.
        let first_part = parts[0].trim().to_uppercase();
        
        // Returns a `RequestLine`.
        macro_rules! get_request {
            () => {{
                let method = HTTP_METHOD[HTTP_METHOD.iter().position(|m| *m == first_part).unwrap()];
                let target = String::from(parts[1].trim());
                let version = String::from(parts[2].trim()).to_uppercase();
                
                Ok(
                    StartLine::RequestLine {
                        method,
                        target,
                        version
                    }
                )
            }}
        }
        
        // Returns a `StatusLine`.
        macro_rules! get_status {
            () => {{
                let version = first_part;
                
                // Try to convert the status code to an integer.
                let code = if let Ok(i) = parts[1].trim().parse::<u32>() {
                    i
                } else {
                    // The status code was not a valid integer.
                    return Err(format!("Bad code for Status line, not an unsigned integer: `{}`", parts[1]));
                };
                
                // Get the reason by folding the remaining parts of the message together.
                let reason = String::from(
                    parts.iter().skip(2)
                        .fold(
                            String::new(), 
                            |mut res, s| {
                                res.push(' ');
                                res.push_str(s);
                                res
                            }
                        ).trim()
                );
                
                // If the reason is empty then there is no reason given.
                let reason = if reason.is_empty() {
                    None
                } else {
                    // Otherwise there is some reason given
                    Some(reason)
                };
                
                Ok(
                    StartLine::StatusLine {
                        version,
                        code,
                        reason
                    }
                )
            }}
        }
        
        // If the first part is found to match a HTTP_METHOD string then it is a Request line.
        for m in HTTP_METHOD.iter() {
            if first_part == *m {
                return get_request!();
            }
        }
        // Otherwise it is a Status line.
        return get_status!();
    }
    /// Unwraps the `RequestLine` to its values.
    pub fn request<'a>(&'a self) -> (&'static str, &'a String, &'a String) {
        if let StartLine::RequestLine { method, ref target, ref version } = *self {
            (method, target, version)
        } else {
            panic!("Called `request` on a non `RequestLine`.");
        }
    }
    /// Unwraps the `RequestLine` to its values.
    pub fn status<'a>(&'a self) -> (&String, u32, &Option<String>) {
        if let StartLine::StatusLine { ref version, code, ref reason } = *self {
            (version, code, reason)
        } else {
            panic!("Called `status` on a non `StartLine`.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_line() {
        assert_eq!(
            StartLine::from("get / http/1.1").unwrap(),
            StartLine::RequestLine {
                method: "GET",
                target: String::from("/"),
                version: String::from("HTTP/1.1")
            },
            "Test RequestLine::from-1 failed."
        );
        
        assert_eq!(
            StartLine::from("GET \"/space test\" http/2.1").unwrap(),
            StartLine::RequestLine {
                method: "GET",
                target: String::from("/space test"),
                version: String::from("HTTP/2.1")
            },
            "Test RequestLine::from-2 failed."
        );
        
        assert!(
            StartLine::from("fail \"/space test\" http/2.1").is_err(),
            "Test RequestLine::from-3 failed."
        );
        
        assert!(
            StartLine::from("fail /space test http/2.1").is_err(),
            "Test RequestLine::from-4 failed."
        );
    }
    #[test]
    fn test_status_line() {
        assert_eq!(
            StartLine::from("http/1.1 000 OK").unwrap(),
            StartLine::StatusLine {
                version: String::from("HTTP/1.1"),
                code: 0,
                reason: Some(String::from("OK"))
            },
            "Test StatusLine::from-1 failed."
        );
        
        assert_eq!(
            StartLine::from("http/2.1 012 test").unwrap(),
            StartLine::StatusLine {
                version: String::from("HTTP/2.1"),
                code: 12,
                reason: Some(String::from("test"))
            },
            "Test StatusLine::from-2 failed."
        );
        
        assert_eq!(
            StartLine::from("http/2.1 012 testing with spaces in reason").unwrap(),
            StartLine::StatusLine {
                version: String::from("HTTP/2.1"),
                code: 12,
                reason: Some(String::from("testing with spaces in reason"))
            },
            "Test StatusLine::from-3 failed."
        );
        
        assert_eq!(
            StartLine::from("http/2.1 012").unwrap(),
            StartLine::StatusLine {
                version: String::from("HTTP/2.1"),
                code: 12,
                reason: None
            },
            "Test StatusLine::from-4 failed."
        );
    }
}
