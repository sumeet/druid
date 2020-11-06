// Copyright 2020 The Druid Authors.
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

//! Creating, interpreting, and validating textual representations of values.

use super::Selection;
use std::str::FromStr;

/// A trait for types that create, interpret, and validate textual representations
/// of values.
///
/// A formatter has two responsiblities: converting a value into an appropriate
/// string representation, and attempting to convert a string back into the
/// appropriate value.
///
/// In addition, a formatter performs validation on *partial* strings; that is,
/// it determines whether or not a string represents a potentially valid value,
/// even if it is not currently valid.
pub trait Formatter<T> {
    /// Return the string representation of this value.
    fn format(&self, value: &T) -> String;
    /// Return the string representation of this value, to be used during editing.
    ///
    /// This can be used if you want the text to differ based on whether or not
    /// it is being edited; for instance you might display a dollar sign when
    /// not editing, but not display it during editing.
    fn format_for_editing(&self, value: &T) -> String {
        self.format(value)
    }

    /// Determine whether the newly edited text is valid for this value type.
    ///
    /// This always returns a [`Validation`] object which indicates if
    /// validation was successful or not, and which can also optionally,
    /// regardless of success or failure, include new text and selection values
    /// that should replace the current ones.
    ///
    ///
    /// # Replacing the text or selection during validation
    ///
    /// Your `Formatter` may wish to change the current text or selection during
    /// editing for a number of reasons. For instance if validation fails, you
    /// may wish to allow editing to continue, but select the invalid region;
    /// alternatively you may consider input valid but want to transform it,
    /// such as by changing case or inserting spaces.
    ///
    /// If you do *not* explicitly set replacement text, and validation is not
    /// successful, the edit will be ignored.
    ///
    /// [`Validation`]: Validation
    fn validate_partial_input(&self, input: &str, sel: &Selection) -> Validation;

    /// The value represented by the input, or an error if the input is invalid.
    ///
    /// This must return `Ok()` for any string created by [`format`].
    ///
    /// [`format`]: #tymethod.format
    fn value(&self, input: &str) -> Result<T, ValidationError>;
}

/// A naive [`Formatter`] for types that implement [`FromStr`].
///
/// [`Formatter`]: Formatter
/// [`FromStr`]: std::str::FromStr
pub struct ParseFormatter;

/// The result of a [`Formatter`] attempting to validate some partial input.
pub struct Validation {
    result: Result<(), ValidationError>,
    /// A manual selection override.
    ///
    /// This will be set as the new selection (regardless of whether or not
    /// validation succeeded or failed)
    pub selection_change: Option<Selection>,
    /// A manual text override.
    ///
    /// This will be set as the new text, regardless of whether or not
    /// validation failed.
    pub text_change: Option<String>,
}

/// An error that occurs when attempting to parse text input.
//FIXME: remove this 'message' stuff and force people to use a real error type
//like FromStr does
#[derive(Debug)]
pub enum ValidationError {
    /// An error describing the failure.
    Err(Box<dyn std::error::Error>),
    /// A String describing the failure.
    Message(String),
}

impl ValidationError {
    /// Construct a `ValidationError` from some other [`Error`] type.
    ///
    /// [`Error`]: std::error::Error
    pub fn from_err(err: impl std::error::Error + 'static) -> Self {
        ValidationError::Err(Box::new(err))
    }

    /// Construct a `ValidationError` with a `String`.
    pub fn with_message(msg: String) -> Self {
        ValidationError::Message(msg)
    }
}

impl Validation {
    /// Create a `Validation` indicating succes.
    pub fn success() -> Self {
        Validation {
            result: Ok(()),
            selection_change: None,
            text_change: None,
        }
    }

    /// Create a `Validation` with an error indicating the failure reason.
    pub fn failure_with_err(err: impl std::error::Error + 'static) -> Self {
        Validation {
            result: Err(ValidationError::Err(Box::new(err))),
            ..Validation::success()
        }
    }

    /// Create a `Validation` with a String indicating the failure reason.
    pub fn failure_with_message(message: impl Into<String>) -> Self {
        Validation {
            result: Err(ValidationError::Message(message.into())),
            ..Validation::success()
        }
    }

    /// Optionally set a `String` that will replace the current contents.
    pub fn change_text(mut self, text: String) -> Self {
        self.text_change = Some(text);
        self
    }

    /// Optionally set a [`Selection`] that will replace the current one.
    pub fn change_selection(mut self, sel: Selection) -> Self {
        self.selection_change = Some(sel);
        self
    }

    /// Returns `true` if this `Validation` indicates success.
    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }

    /// If validation failed, return the underlying [`ValidationError`].
    ///
    /// [`ValidationError`]: ValidationError
    pub fn error(&self) -> Option<&ValidationError> {
        self.result.as_ref().err()
    }
}

impl<T> Formatter<T> for ParseFormatter
where
    T: FromStr + std::fmt::Display,
    <T as FromStr>::Err: std::error::Error + 'static,
{
    fn format(&self, value: &T) -> String {
        value.to_string()
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        match input.parse::<T>() {
            Ok(_) => Validation::success(),
            Err(e) => Validation::failure_with_err(e),
        }
    }

    fn value(&self, input: &str) -> Result<T, ValidationError> {
        input.parse().map_err(ValidationError::from_err)
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Err(err) => err.fmt(f),
            ValidationError::Message(s) => s.fmt(f),
        }
    }
}
impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let ValidationError::Err(e) = self {
            Some(e.as_ref())
        } else {
            None
        }
    }
}
