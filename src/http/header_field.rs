//! `header_field` is a module to handle the Header Fields of a HTTP message and their components.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

use std::string::String;

#[derive(Clone, PartialEq, Eq, Debug)]
/// A `HeaderField` defines a `name:value` association in the header section of a HTTP message.
pub struct HeaderField {
    /// The `name` associated with the `HeaderField`.
    pub name: String,
    /// The `value` associated with the `HeaderField`.
    pub value: String
}

impl HeaderField {
    /// Converts the passed `str` into a `HeaderField`.
    ///
    /// # Params
    ///
    /// msg --- The `str` to convert.
    pub fn from(msg: &str) -> Result<HeaderField, String> {
        // Split the string on the colon.
        let parts: Vec<&str> = msg.split(":").collect();
        
        // Make sure the split worked properly.
        if parts.len() >= 2 {
            // Return the HeaderField.
            Ok(
                HeaderField {
                    // The first part is the name of the field.
                    name: String::from(parts[0].trim()),
                    // All the remaining parts make up the value string.
                    value: String::from(parts.iter().skip(2).fold(
                        // The second part has no colon at the front.
                        String::from(parts[1]),
                        // All other parts have a colon at the front.
                        |mut res, s| {
                            res.push_str(":");
                            res.push_str(s);
                            res
                        }
                    ).trim())
                }
            )
        } else {
            // The split was not successful and it is a bad header field.
            Err(format!("Bad Header Field: `{}`", msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_field() {
        assert_eq!(
            HeaderField::from("header1:field1").unwrap(),
            HeaderField {
                name: String::from("header1"),
                value: String::from("field1")
            },
            "Test HeaderField::from-1 failed."
        );
        
        assert_eq!(
            HeaderField::from(" header1:field1 ").unwrap(),
            HeaderField {
                name: String::from("header1"),
                value: String::from("field1")
            },
            "Test HeaderField::from-2 failed."
        );
        assert_eq!(
            HeaderField::from(" header1 : field1 ").unwrap(),
            HeaderField {
                name: String::from("header1"),
                value: String::from("field1")
            },
            "Test HeaderField::from-3 failed."
        );
    }
}
