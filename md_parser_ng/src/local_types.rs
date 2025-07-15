/*
 *   Copyright (c) 2025 R3BL LLC
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! Local type definitions to avoid using deprecated types from r3bl_tui
//! Pinned to specific commit for compatibility. This commit does not have `AsStrSlice` or any
//! other files that are in this `md_parser_ng` crate. However, by the time this crate was
//! archived, the `r3bl_tui` crate had diverged significantly, and the missing code is in
//! `local_types.rs` and `local_constants.rs`.
//!
//! ```toml
//! r3bl_tui = { git = "https://github.com/r3bl-org/r3bl-open-core.git", rev = "fe1182a0f6c40f38852f2204b9895bef546aeed7" }
//! ```

use crate::{AsStrSlice, InlineString, List, MdLineFragment};
use std::fmt::Display;

/// Represents a parsed line in a smart list with metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartListLineAlt<'a> {
    pub indent: usize,
    pub bullet_str: InlineString,
    pub content: AsStrSlice<'a>,
}

/// Type alias for a list of fragments in one line.
pub type FragmentsInOneLine<'a> = List<MdLineFragment<'a>>;

/// Type alias for a list of lines, where each line is a list of fragments.
pub type Lines<'a> = List<FragmentsInOneLine<'a>>;

/// Intermediate representation for smart lists during parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartListIRAlt<'a> {
    pub indent: usize,
    pub bullet_kind: r3bl_tui::BulletKind,
    pub content_lines: List<SmartListLineAlt<'a>>,
}

/// Replacement for [`std::borrow::Cow`] that uses [`InlineString`] if it is owned.
/// And `&str` if it is borrowed.
#[derive(Clone, Debug, PartialEq)]
pub enum InlineStringCow<'a> {
    Borrowed(&'a str),
    Owned(InlineString),
}

impl<'a> InlineStringCow<'a> {
    #[must_use]
    pub fn new_empty_borrowed() -> Self {
        InlineStringCow::Borrowed("")
    }
    #[must_use]
    pub fn new_borrowed(arg: &'a str) -> Self {
        InlineStringCow::Borrowed(arg)
    }
    #[must_use]
    pub fn new_owned(arg: InlineString) -> Self {
        InlineStringCow::Owned(arg)
    }
}

impl AsRef<str> for InlineStringCow<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str(),
        }
    }
}

impl Display for InlineStringCow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Borrowed(as_ref_str) => write!(f, "{as_ref_str}"),
            Self::Owned(as_ref_str) => write!(f, "{as_ref_str}"),
        }
    }
}

/// Represents the position status of an index relative to content boundaries.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PositionStatus {
    /// The index is within valid content boundaries.
    Within,
    /// The index is exactly at the content boundary.
    Boundary,
    /// The index exceeds the content boundaries.
    Beyond,
}
