// Copyright 2018-2021 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Module containing InternalError implementation.

use std::error;
use std::fmt;

struct Source {
    prefix: Option<String>,
    source: Box<dyn error::Error>,
}

/// An error which is returned for reasons internal to the function.
///
/// This error is produced when a failure occurred within the function but the failure is due to an
/// internal implementation detail of the function. This generally means that there is no specific
/// information which can be returned that would help the caller of the function recover or
/// otherwise take action.
pub struct InternalError {
    message: Option<String>,
    source: Option<Source>,
}

impl InternalError {
    /// Constructs a new `InternalError` from a specified source error.
    ///
    /// The implementation of `std::fmt::Display` for this error will simply pass through the
    /// display of the source message unmodified.
    ///
    /// # Examples
    ///
    /// ```
    /// use augrim::error::InternalError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    /// let internal_error = InternalError::from_source(Box::new(io_err));
    /// assert_eq!(format!("{}", internal_error), "io error");
    /// ```
    pub fn from_source(source: Box<dyn error::Error>) -> Self {
        Self {
            message: None,
            source: Some(Source {
                prefix: None,
                source,
            }),
        }
    }

    /// Constructs a new `InternalError` from a specified source error and message string.
    ///
    /// The implementation of `std::fmt::Display` for this error will be the message string
    /// provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use augrim::error::InternalError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    /// let internal_error = InternalError::from_source_with_message(Box::new(io_err), "oops".to_string());
    /// assert_eq!(format!("{}", internal_error), "oops");
    /// ```
    pub fn from_source_with_message(source: Box<dyn error::Error>, message: String) -> Self {
        Self {
            message: Some(message),
            source: Some(Source {
                prefix: None,
                source,
            }),
        }
    }

    /// Constructs a new `InternalError` from a specified source error and prefix string.
    ///
    /// The implementation of `std::fmt::Display` for this error will be constructed from the
    /// prefix and source message's display following the format of `format!("{}: {}", prefix,
    /// source)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use augrim::error::InternalError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    /// let internal_error = InternalError::from_source_with_prefix(Box::new(io_err), "Could not open file".to_string());
    /// assert_eq!(format!("{}", internal_error), "Could not open file: io error");
    /// ```
    pub fn from_source_with_prefix(source: Box<dyn error::Error>, prefix: String) -> Self {
        Self {
            message: None,
            source: Some(Source {
                prefix: Some(prefix),
                source,
            }),
        }
    }

    /// Constructs a new `InternalError` with a specified message string.
    ///
    /// The implementation of `std::fmt::Display` for this error will be the message string
    /// provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use augrim::error::InternalError;
    ///
    /// let internal_error = InternalError::with_message("oops".to_string());
    /// assert_eq!(format!("{}", internal_error), "oops");
    /// ```
    pub fn with_message(message: String) -> Self {
        Self {
            message: Some(message),
            source: None,
        }
    }

    /// Reduces the `InternalError` to the display string
    ///
    /// If the error includes a source, the debug format will be logged to provide
    /// information that may be lost on the conversion.
    pub fn reduce_to_string(self) -> String {
        if self.source.is_some() {
            debug!("{:?}", self);
        }

        self.to_string()
    }
}

impl error::Error for InternalError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.source {
            Some(s) => Some(s.source.as_ref()),
            None => None,
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{}", m),
            None => match &self.source {
                Some(s) => match &s.prefix {
                    Some(p) => write!(f, "{}: {}", p, s.source),
                    None => write!(f, "{}", s.source),
                },
                None => write!(f, "{}", std::any::type_name::<InternalError>()),
            },
        }
    }
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_struct = f.debug_struct("InternalError");

        if let Some(message) = &self.message {
            debug_struct.field("message", message);
        }

        if let Some(source) = &self.source {
            if let Some(prefix) = &source.prefix {
                debug_struct.field("prefix", prefix);
            }

            debug_struct.field("source", &source.source);
        }

        debug_struct.finish()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Tests that errors constructed with `InternalError::from_source` return a debug string of
    /// the form `format!("InternalError { {:?} }", source)`.
    #[test]
    fn test_debug_from_source() {
        let msg = "test message";
        let debug = "InternalError { source: InternalError { message: \"test message\" } }";
        let err =
            InternalError::from_source(Box::new(InternalError::with_message(msg.to_string())));
        assert_eq!(format!("{:?}", err), debug);
    }

    /// Tests that errors constructed with `InternalError::from_source_with_message` return a debug
    /// string of the form `format!("InternalError { message: {:?}, source: {:?} }", message,
    /// source)`.
    #[test]
    fn test_debug_from_source_with_message() {
        let msg = "test message";
        let debug = "InternalError { message: \"test message\", source: InternalError { message: \"unused\" } }";
        let err = InternalError::from_source_with_message(
            Box::new(InternalError::with_message("unused".to_string())),
            msg.to_string(),
        );
        assert_eq!(format!("{:?}", err), debug);
    }

    /// Tests that errors constructed with `InternalError::from_source_with_prefix` return a debug
    /// string of the form `format!("InternalError { prefix: {:?}, source: {:?} }", prefix,
    /// source)`.
    #[test]
    fn test_debug_from_source_with_prefix() {
        let prefix = "test prefix";
        let msg = "test message";
        let debug = "InternalError { prefix: \"test prefix\", source: InternalError { message: \"test message\" } }";
        let err = InternalError::from_source_with_prefix(
            Box::new(InternalError::with_message(msg.to_string())),
            prefix.to_string(),
        );
        assert_eq!(format!("{:?}", err), debug);
    }

    /// Tests that errors constructed with `InternalError::with_message` return a debug
    /// string of the form `format!("InternalError { message: {:?} }", message)`.
    #[test]
    fn test_debug_with_message() {
        let msg = "test message";
        let debug = "InternalError { message: \"test message\" }";
        let err = InternalError::with_message(msg.to_string());
        assert_eq!(format!("{:?}", err), debug);
    }

    /// Tests that error constructed with `InternalError::from_source` return a display
    /// string which is the same as the source's display string.
    #[test]
    fn test_display_from_source() {
        let msg = "test message";
        let err =
            InternalError::from_source(Box::new(InternalError::with_message(msg.to_string())));
        assert_eq!(format!("{}", err), msg);
    }

    /// Tests that error constructed with `InternalError::from_source_with_message` return
    /// message as the display string.
    #[test]
    fn test_display_from_source_with_message() {
        let msg = "test message";
        let err = InternalError::from_source_with_message(
            Box::new(InternalError::with_message("unused".to_string())),
            msg.to_string(),
        );
        assert_eq!(format!("{}", err), msg);
    }

    /// Tests that error constructed with `InternalError::from_source_with_message` return
    /// a display string of the form `format!("{}: {}", prefix, source)`.
    #[test]
    fn test_display_from_source_with_prefix() {
        let prefix = "test prefix";
        let msg = "test message";
        let err = InternalError::from_source_with_prefix(
            Box::new(InternalError::with_message(msg.to_string())),
            prefix.to_string(),
        );
        assert_eq!(format!("{}", err), format!("{}: {}", prefix, msg));
    }

    /// Tests that error constructed with `InternalError::with_message` return message as the
    /// display string.
    #[test]
    fn test_display_with_message() {
        let msg = "test message";
        let err = InternalError::with_message(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }
}
