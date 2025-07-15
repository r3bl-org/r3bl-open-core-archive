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

pub const COMMA_CHAR: char = ',';
pub const TAB_CHAR: char = '\t';
pub const COLON: &str = ":";
pub const COMMA: &str = ",";
pub const SPACE: &str = " ";
pub const NEW_LINE_CHAR: char = '\n';
pub const SPACE_CHAR: char = ' ';
pub const DATE: &str = "@date";
pub const AUTHORS: &str = "@authors";

// Additional constants from md_parser::constants
pub const TAGS: &str = "@tags";
pub const TITLE: &str = "@title";
pub const NEW_LINE: &str = "\n";
pub const UNDERSCORE: &str = "_";
pub const STAR: &str = "*";
pub const BACK_TICK: &str = "`";
pub const CHECKED: &str = "[x]";
pub const UNCHECKED: &str = "[ ]";
pub const LIST_PREFIX_BASE_WIDTH: usize = 2;
pub const ORDERED_LIST_PARTIAL_PREFIX: &str = ".";
pub const UNORDERED_LIST_PREFIX: &str = "-";
pub const CODE_BLOCK_START_PARTIAL: &str = "```";
pub const CODE_BLOCK_END: &str = "```";
pub const LEFT_BRACKET: &str = "[";
pub const RIGHT_BRACKET: &str = "]";
pub const LEFT_PARENTHESIS: &str = "(";
pub const RIGHT_PARENTHESIS: &str = ")";
pub const LEFT_IMAGE: &str = "![";
pub const RIGHT_IMAGE: &str = "](";
pub const HEADING_CHAR: char = '#';