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

//! Simple markdown parser without nom dependency.
//! 
//! This module provides a simpler, more performant alternative to the nom-based parser
//! by working directly with `&[GCString]` without the overhead of virtual arrays.

const CHECKED_UPPER: &str = "[X]";

use crate::{
    BulletKind,
    GCString,
    List,
    MdDocument,
    MdElement,
    MdLineFragment,
    MdLineFragments,
    HeadingData,
    local_constants::{COMMA_CHAR},
    HeadingLevel,
    CodeBlockLine,
    CodeBlockLineContent,
    HyperlinkData,
    local_constants::*,
};

/// Parser state that tracks the current position in the lines array.
#[derive(Debug, Clone)]
pub struct ParserState<'a> {
    pub lines: &'a [GCString],
    pub current_line: usize,
}

impl<'a> ParserState<'a> {
    /// Create a new parser state.
    pub fn new(lines: &'a [GCString]) -> Self {
        Self {
            lines,
            current_line: 0,
        }
    }

    /// Check if there are more lines to parse.
    pub fn has_more_lines(&self) -> bool {
        self.current_line < self.lines.len()
    }

    /// Get the current line without advancing.
    pub fn current_line(&self) -> Option<&'a GCString> {
        self.lines.get(self.current_line)
    }

    /// Get the current line and advance to the next.
    pub fn consume_line(&mut self) -> Option<&'a GCString> {
        if self.has_more_lines() {
            let line = &self.lines[self.current_line];
            self.current_line += 1;
            Some(line)
        } else {
            None
        }
    }

    /// Peek at a line at a specific offset from current without advancing.
    pub fn peek_line(&self, offset: usize) -> Option<&'a GCString> {
        self.lines.get(self.current_line + offset)
    }

    /// Advance the parser by n lines.
    pub fn advance(&mut self, n: usize) {
        self.current_line = (self.current_line + n).min(self.lines.len());
    }
}

/// Result type for parsers.
pub type ParseResult<T> = Option<T>;

/// Main entry point for parsing markdown without nom.
pub fn parse_markdown_simple(lines: &[GCString]) -> MdDocument<'_> {
    let mut state = ParserState::new(lines);
    let mut elements = Vec::new();

    while state.has_more_lines() {
        if let Some(line) = state.current_line() {
            // Try metadata parsers first
            if let Some(title) = parse_line_kv(TITLE, line) {
                elements.push(MdElement::Title(title));
                state.advance(1);
            } else if let Some(date) = parse_line_kv(DATE, line) {
                elements.push(MdElement::Date(date));
                state.advance(1);
            } else if let Some(tags) = parse_line_csv(TAGS, line) {
                elements.push(MdElement::Tags(tags));
                state.advance(1);
            } else if let Some(authors) = parse_line_csv(AUTHORS, line) {
                elements.push(MdElement::Authors(authors));
                state.advance(1);
            } else if let Some(heading) = parse_heading(line) {
                elements.push(MdElement::Heading(heading));
                state.advance(1);
            } else if let Some(code_block) = try_parse_code_block(&mut state) {
                elements.push(MdElement::CodeBlock(code_block));
                // state already advanced by try_parse_code_block
            } else if let Some((list_lines, bullet_kind, indent)) = try_parse_smart_list(&mut state) {
                elements.push(MdElement::SmartList((list_lines, bullet_kind, indent)));
                // state already advanced by try_parse_smart_list
            } else if line.is_empty() {
                elements.push(MdElement::Text(List::from(vec![])));
                state.advance(1);
            } else {
                // Parse text with fragments
                let fragments = parse_text_fragments(line);
                elements.push(MdElement::Text(fragments));
                state.advance(1);
            }
        }
    }

    List::from(elements)
}

/// Parse a key-value metadata line like "@title: My Title".
pub fn parse_line_kv<'a>(tag_name: &str, line: &'a GCString) -> ParseResult<&'a str> {
    let prefix = format!("{}{}{}", tag_name, COLON, SPACE);
    let line_str: &str = line.as_ref();
    
    if line_str.starts_with(&prefix) {
        let value = &line_str[prefix.len()..];
        let value = value.trim();
        
        // Check for nested tag
        if value.contains(&prefix) {
            return None;
        }
        
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    } else {
        None
    }
}

/// Parse a comma-separated values metadata line like "@tags: rust, parser, markdown".
pub fn parse_line_csv<'a>(tag_name: &str, line: &'a GCString) -> ParseResult<List<&'a str>> {
    let prefix = format!("{}{}{}", tag_name, COLON, SPACE);
    let line_str: &str = line.as_ref();
    
    if line_str.starts_with(&prefix) {
        let values = &line_str[prefix.len()..];
        let items: Vec<&str> = values
            .split(COMMA_CHAR)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        
        if items.is_empty() {
            None
        } else {
            Some(List::from(items))
        }
    } else {
        None
    }
}

/// Parse a markdown heading line.
pub fn parse_heading(line: &GCString) -> ParseResult<HeadingData<'_>> {
    let line_str: &str = line.as_ref();
    
    if line_str.is_empty() || !line_str.starts_with(HEADING_CHAR) {
        return None;
    }
    
    let level = line_str
        .chars()
        .take_while(|&c| c == HEADING_CHAR)
        .count();
    
    if level > 6 || level == 0 {
        return None;
    }
    
    // Check if there's a space after the hashes
    if line_str.len() <= level || !line_str[level..].starts_with(SPACE_CHAR) {
        return None;
    }
    
    let text = line_str[level..].trim();
    
    Some(HeadingData {
        heading_level: HeadingLevel::from(level),
        text,
    })
}

/// Try to parse a code block starting at the current position.
pub fn try_parse_code_block<'a>(state: &mut ParserState<'a>) -> ParseResult<List<CodeBlockLine<'a>>> {
    let first_line = state.current_line()?;
    let first_line_str: &str = first_line.as_ref();
    
    // Code blocks must start at the beginning of the line (no indentation)
    // Indented code blocks are part of lists and handled separately
    if !first_line_str.starts_with(CODE_BLOCK_START_PARTIAL) {
        return None;
    }
    
    // Check if the line has leading whitespace (indented code blocks are not top-level)
    if first_line_str.trim_start() != first_line_str {
        return None;
    }
    
    // Extract language if present
    let lang = first_line_str[CODE_BLOCK_START_PARTIAL.len()..].trim();
    let lang = if lang.is_empty() { None } else { Some(lang) };
    
    // Save the starting position in case we need to backtrack
    let start_line = state.current_line;
    
    // Advance past the opening line
    state.advance(1);
    
    let mut lines = vec![CodeBlockLine {
        language: lang,
        content: CodeBlockLineContent::StartTag,
    }];
    
    // Collect lines until we find the closing ```
    while let Some(line) = state.current_line() {
        let line_str: &str = line.as_ref();
        
        if line_str.trim() == CODE_BLOCK_END {
            lines.push(CodeBlockLine {
                language: lang,
                content: CodeBlockLineContent::EndTag,
            });
            state.advance(1);
            return Some(List::from(lines));
        }
        
        lines.push(CodeBlockLine {
            language: lang,
            content: CodeBlockLineContent::Text(line_str),
        });
        state.advance(1);
    }
    
    // No closing ``` found - this is not a valid code block
    // Backtrack to where we started
    state.current_line = start_line;
    None
}

/// Try to parse a smart list starting at the current position.
pub fn try_parse_smart_list<'a>(state: &mut ParserState<'a>) -> ParseResult<(List<List<MdLineFragment<'a>>>, BulletKind, usize)> {
    let first_line = state.current_line()?;
    let first_line_str: &str = first_line.as_ref();
    
    // Determine indent level and bullet type
    let (indent, bullet_kind, content_start) = match parse_list_item_prefix(first_line_str) {
        Some(info) => info,
        None => return None,
    };
    
    // Ensure indent is a multiple of LIST_PREFIX_BASE_WIDTH (usually 2)
    if indent % LIST_PREFIX_BASE_WIDTH != 0 {
        return None;
    }
    
    // Calculate the bullet width (e.g., "1. " = 3, "- " = 2)
    let bullet_width = content_start - indent;
    
    let mut list_lines = Vec::new();
    
    // Parse the first line
    let first_content = &first_line_str[content_start..];
    let mut fragments = vec![
        match bullet_kind {
            BulletKind::Unordered => MdLineFragment::UnorderedListBullet { indent, is_first_line: true },
            BulletKind::Ordered(num) => MdLineFragment::OrderedListBullet { indent, number: num, is_first_line: true },
        }
    ];
    
    // Parse fragments in the content
    let content_fragments = parse_text_fragments_from_str(first_content);
    fragments.extend(content_fragments.into_iter());
    list_lines.push(List::from(fragments));
    
    state.advance(1);
    
    // Parse continuation lines
    while let Some(line) = state.current_line() {
        let line_str: &str = line.as_ref();
        
        // Check if this is another list item
        if parse_list_item_prefix(line_str).is_some() {
            // If we find another list item at any level, stop here
            // The legacy parser creates separate SmartList elements for each item
            break;
        } else if line_str.trim().is_empty() {
            // Empty line ends the list (legacy behavior)
            break;
        } else {
            // Check if it's a continuation line (properly indented)
            let line_indent = line_str.len() - line_str.trim_start().len();
            
            // Legacy parser requires exactly (indent + bullet_width) spaces for continuation lines
            if line_indent == indent + bullet_width {
                // Continuation line
                let mut continuation_fragments = vec![
                    match bullet_kind {
                        BulletKind::Unordered => MdLineFragment::UnorderedListBullet { indent, is_first_line: false },
                        BulletKind::Ordered(num) => MdLineFragment::OrderedListBullet { indent, number: num, is_first_line: false },
                    }
                ];
                
                // Parse the continuation line content
                let content = &line_str[line_indent..];
                
                // Special handling for code blocks - legacy parser keeps them as plain text
                if content.trim().starts_with(CODE_BLOCK_START_PARTIAL) || content.trim() == CODE_BLOCK_END {
                    continuation_fragments.push(MdLineFragment::Plain(content));
                } else {
                    // Parse regular text fragments
                    let content_fragments = parse_text_fragments_from_str(content);
                    continuation_fragments.extend(content_fragments.into_iter());
                }
                
                list_lines.push(List::from(continuation_fragments));
                
                state.advance(1);
            } else {
                // Not properly indented, end of list
                break;
            }
        }
    }
    
    if list_lines.is_empty() {
        None
    } else {
        Some((List::from(list_lines), bullet_kind, indent))
    }
}

/// Parse the prefix of a list item line and return (indent, bullet_kind, content_start_pos).
fn parse_list_item_prefix(line: &str) -> Option<(usize, BulletKind, usize)> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    
    // Check for unordered list
    if trimmed.starts_with(UNORDERED_LIST_PREFIX) {
        return Some((indent, BulletKind::Unordered, indent + UNORDERED_LIST_PREFIX.len()));
    }
    
    // Check for ordered list
    let bytes = trimmed.as_bytes();
    let mut num_end = 0;
    while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
        num_end += 1;
    }
    
    if num_end > 0 && trimmed[num_end..].starts_with(ORDERED_LIST_PARTIAL_PREFIX) {
        if let Ok(number) = trimmed[..num_end].parse::<usize>() {
            return Some((
                indent, 
                BulletKind::Ordered(number as usize), 
                indent + num_end + ORDERED_LIST_PARTIAL_PREFIX.len()
            ));
        }
    }
    
    None
}

/// Parse text fragments from a string slice (helper for smart list parsing).
fn parse_text_fragments_from_str(text: &str) -> Vec<MdLineFragment<'_>> {
    let mut fragments = Vec::new();
    let mut current_pos = 0;
    
    // Check for checkbox at the beginning
    if text.starts_with(UNCHECKED) && text.len() > UNCHECKED.len() && text.as_bytes()[UNCHECKED.len()] == b' ' {
        fragments.push(MdLineFragment::Checkbox(false));
        current_pos = UNCHECKED.len();
    } else if text.starts_with(CHECKED) && text.len() > CHECKED.len() && text.as_bytes()[CHECKED.len()] == b' ' {
        fragments.push(MdLineFragment::Checkbox(true));
        current_pos = CHECKED.len();
    }
    
    while current_pos < text.len() {
        // Try to parse a special fragment first
        if let Some((fragment, consumed)) = try_parse_fragment(text, current_pos) {
            fragments.push(fragment);
            current_pos += consumed;
        } else {
            // Start collecting plain text
            let plain_start = current_pos;
            let bytes = text.as_bytes();
            
            while current_pos < bytes.len() {
                if is_fragment_start_byte(bytes[current_pos]) {
                    // Check if it's triple backticks - if so, include them in plain text
                    if bytes[current_pos] == b'`' && is_triple_backtick(text, current_pos) {
                        // Skip all three backticks
                        current_pos += 3;
                        continue;
                    }
                    // Check if it's a checkbox pattern or [X] - if so, include it in plain text
                    if bytes[current_pos] == b'[' {
                        if is_checkbox_pattern(text, current_pos) {
                            // Skip the checkbox pattern
                            if text[current_pos..].starts_with(CHECKED) || text[current_pos..].starts_with(UNCHECKED) {
                                current_pos += 3;
                            } else {
                                break;
                            }
                            continue;
                        } else if text[current_pos..].starts_with(CHECKED_UPPER) {
                            // [X] is always plain text
                            current_pos += 3;
                            continue;
                        } else if !is_likely_link_start(text, current_pos) {
                            // Not a valid link pattern, continue collecting as plain text
                            current_pos += 1;
                            continue;
                        }
                    }
                    // For exclamation marks, check if it's an image pattern
                    if bytes[current_pos] == b'!' {
                        if current_pos + 1 < bytes.len() && bytes[current_pos + 1] == b'[' {
                            // This might be an image, break to check
                            break;
                        } else {
                            // Just a regular exclamation mark, include it
                            current_pos += 1;
                            continue;
                        }
                    }
                    // Might be a special fragment, break to check
                    break;
                }
                // Regular character, advance
                current_pos += 1;
                while current_pos < bytes.len() && !is_utf8_char_boundary(bytes[current_pos]) {
                    current_pos += 1;
                }
            }
            
            // Add the plain text we collected
            if current_pos > plain_start {
                fragments.push(MdLineFragment::Plain(&text[plain_start..current_pos]));
            } else {
                // No plain text collected, but we have a special character that failed to parse
                // For [ character that failed link parsing, collect the entire reference-style pattern
                if bytes[plain_start] == b'[' {
                    current_pos = plain_start + 1;
                    let mut bracket_depth = 1;
                    
                    // Collect through all brackets until we hit a non-bracket special character
                    while current_pos < bytes.len() && bracket_depth > 0 {
                        match bytes[current_pos] {
                            b'[' => bracket_depth += 1,
                            b']' => {
                                bracket_depth -= 1;
                                if bracket_depth == 0 {
                                    current_pos += 1;
                                    // Check if there's another [ immediately after (reference-style link)
                                    if current_pos < bytes.len() && bytes[current_pos] == b'[' {
                                        bracket_depth = 1;
                                        current_pos += 1;
                                        continue;
                                    }
                                    break;
                                }
                            }
                            b'\n' => break, // Links can't span lines
                            _ => {}
                        }
                        current_pos += 1;
                    }
                    fragments.push(MdLineFragment::Plain(&text[plain_start..current_pos]));
                } else {
                    // For other characters (*,_,`), add as single character (legacy behavior)
                    current_pos += 1;
                    while current_pos < bytes.len() && !is_utf8_char_boundary(bytes[current_pos]) {
                        current_pos += 1;
                    }
                    fragments.push(MdLineFragment::Plain(&text[plain_start..current_pos]));
                }
            }
        }
    }
    
    fragments
}

/// Parse text fragments within a line.
pub fn parse_text_fragments<'a>(line: &'a GCString) -> MdLineFragments<'a> {
    let line_str: &'a str = line.as_ref();
    let mut fragments = Vec::new();
    let mut current_pos = 0;
    
    // Special case for legacy compatibility: if line starts with whitespace followed by triple backticks,
    // split it into two fragments to match legacy parser behavior
    if line_str.trim_start() != line_str && line_str.trim_start().starts_with("```") {
        let whitespace_end = line_str.len() - line_str.trim_start().len();
        fragments.push(MdLineFragment::Plain(&line_str[..whitespace_end]));
        fragments.push(MdLineFragment::Plain(&line_str[whitespace_end..]));
        return List::from(fragments);
    }
    
    let mut skip_fragment_parse = false;
    
    while current_pos < line_str.len() {
        // Start collecting plain text
        let plain_start = current_pos;
        let bytes = line_str.as_bytes();
        
        while current_pos < bytes.len() {
            if !skip_fragment_parse && is_fragment_start_byte(bytes[current_pos]) {
                // Check if it's triple backticks - if so, include them in plain text
                if bytes[current_pos] == b'`' && is_triple_backtick(line_str, current_pos) {
                    // Skip all three backticks
                    current_pos += 3;
                    continue;
                }
                // Check if it's a checkbox pattern or [X] - if so, include it in plain text
                if bytes[current_pos] == b'[' {
                    if is_checkbox_pattern(line_str, current_pos) {
                        // Skip the checkbox pattern
                        if line_str[current_pos..].starts_with(CHECKED) || line_str[current_pos..].starts_with(UNCHECKED) {
                            current_pos += 3;
                        } else {
                            break;
                        }
                        continue;
                    } else if line_str[current_pos..].starts_with(CHECKED_UPPER) {
                        // [X] is always plain text
                        current_pos += 3;
                        continue;
                    }
                }
                // For exclamation marks, check if it's an image pattern
                if bytes[current_pos] == b'!' {
                    if current_pos + 1 < bytes.len() && bytes[current_pos + 1] == b'[' {
                        // This might be an image, break to check
                        break;
                    } else {
                        // Just a regular exclamation mark, include it
                        current_pos += 1;
                        continue;
                    }
                }
                // Might be a special fragment, break to check
                break;
            }
            // Regular character, advance
            current_pos += 1;
            while current_pos < bytes.len() && !is_utf8_char_boundary(bytes[current_pos]) {
                current_pos += 1;
            }
        }
        
        // Add any plain text we collected before the special character
        if current_pos > plain_start {
            fragments.push(MdLineFragment::Plain(&line_str[plain_start..current_pos]));
        }
        
        // Now try to parse a special fragment at current position
        if current_pos < line_str.len() {
            if let Some((fragment, consumed)) = try_parse_fragment(line_str, current_pos) {
                fragments.push(fragment);
                current_pos += consumed;
            } else {
                // Failed to parse as special fragment
                // For certain characters (*,_), create single character fragments to match legacy
                let char = bytes[current_pos];
                if char == b'*' || char == b'_' {
                    // Create single character fragment for these special chars
                    let char_start = current_pos;
                    current_pos += 1;
                    while current_pos < bytes.len() && !is_utf8_char_boundary(bytes[current_pos]) {
                        current_pos += 1;
                    }
                    fragments.push(MdLineFragment::Plain(&line_str[char_start..current_pos]));
                } else {
                    // For other characters like [, we need to include them in the next plain text run
                    // Set flag to skip fragment parsing on next iteration
                    skip_fragment_parse = true;
                    // Continue to next iteration of outer loop to collect this as plain text
                    continue;
                }
            }
        }
        
        // Reset skip flag for next iteration
        skip_fragment_parse = false;
    }
    
    List::from(fragments)
}

/// Check if a byte is a UTF-8 character boundary.
fn is_utf8_char_boundary(byte: u8) -> bool {
    // UTF-8 continuation bytes start with 10xxxxxx
    (byte & 0xC0) != 0x80
}

/// Check if a byte might be the start of a special fragment.
fn is_fragment_start_byte(byte: u8) -> bool {
    matches!(byte, b'*' | b'_' | b'`' | b'[' | b'!')
}

/// Check if text at position might be triple backticks.
fn is_triple_backtick(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    pos + 2 < bytes.len() && bytes[pos] == b'`' && bytes[pos + 1] == b'`' && bytes[pos + 2] == b'`'
}

/// Check if a `[` at the given position is likely part of a valid link pattern `[text](url)`.
fn is_likely_link_start(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'[' {
        return false;
    }
    
    // Look for the pattern [...]( 
    let mut check_pos = pos + 1;
    let mut bracket_depth = 1;
    
    while check_pos < bytes.len() && bracket_depth > 0 {
        match bytes[check_pos] {
            b'[' => bracket_depth += 1,
            b']' => {
                bracket_depth -= 1;
                if bracket_depth == 0 && check_pos + 1 < bytes.len() && bytes[check_pos + 1] == b'(' {
                    return true;
                }
            }
            b'\n' => return false, // Links can't span lines
            _ => {}
        }
        check_pos += 1;
    }
    
    false
}

/// Check if text at position is a checkbox pattern that should be plain text.
fn is_checkbox_pattern(text: &str, pos: usize) -> bool {
    let remaining = &text[pos..];
    // Check for [x] or [ ] patterns (not [X] which is always plain text)
    (remaining.starts_with(CHECKED) || remaining.starts_with(UNCHECKED)) &&
    // But only if they're not followed by a space (which would make them valid checkboxes in lists)
    (remaining.len() <= 3 || remaining.as_bytes()[3] != b' ')
}

/// Try to parse a fragment starting at the given byte position.
/// Priority order matches NG parser:
/// 1. Italic (_text_)
/// 2. Bold (*text*)  
/// 3. Inline code (`text`)
/// 4. Image (![alt](url))
/// 5. Link ([text](url))
fn try_parse_fragment(text: &str, start: usize) -> Option<(MdLineFragment<'_>, usize)> {
    let bytes = text.as_bytes();
    if start >= bytes.len() {
        return None;
    }
    
    // Try parsers in priority order
    
    // 1. Italic (_text_)
    if bytes[start] == b'_' {
        if let Some(result) = parse_italic_fragment(text, start, b'_') {
            return Some(result);
        }
    }
    
    // 2. Bold (*text*)
    if bytes[start] == b'*' {
        if let Some(result) = parse_bold_fragment(text, start) {
            return Some(result);
        }
    }
    
    // 3. Inline code (`text`)
    if bytes[start] == b'`' {
        if let Some(result) = parse_inline_code_fragment(text, start) {
            return Some(result);
        }
    }
    
    // 4. Image (![alt](url))
    if bytes[start] == b'!' && start + 1 < bytes.len() && bytes[start + 1] == b'[' {
        if let Some(result) = parse_image_fragment(text, start) {
            return Some(result);
        }
    }
    
    // 5. Link ([text](url))
    if bytes[start] == b'[' {
        if let Some(result) = parse_link_fragment(text, start) {
            return Some(result);
        }
    }
    
    None
}

/// Parse bold text (*text*).
/// Note: R3BL uses single * for bold (non-standard). Standard markdown uses **.
/// This allows empty bold fragments (e.g., **) to match legacy parser behavior.
/// TODO: Consider whether empty formatting should produce Bold("") or Plain("**").
/// TODO: Consider supporting standard markdown ** for bold and * for italic.
fn parse_bold_fragment(text: &str, start: usize) -> Option<(MdLineFragment<'_>, usize)> {
    let bytes = text.as_bytes();
    
    // Skip opening *
    let content_start = start + 1;
    
    // Find closing *
    let mut pos = content_start;
    while pos < bytes.len() {
        if bytes[pos] == b'*' {
            let content = &text[content_start..pos];
            return Some((MdLineFragment::Bold(content), pos + 1 - start));
        }
        pos += 1;
    }
    
    None
}

/// Parse italic text (_text_).
/// Note: R3BL uses single _ for italic and * for bold (non-standard). 
/// Standard markdown uses * or _ for italic and ** for bold.
fn parse_italic_fragment(text: &str, start: usize, delimiter: u8) -> Option<(MdLineFragment<'_>, usize)> {
    let bytes = text.as_bytes();
    
    // Skip opening delimiter
    let content_start = start + 1;
    
    // Find closing delimiter
    let mut pos = content_start;
    while pos < bytes.len() {
        if bytes[pos] == delimiter {
            let content = &text[content_start..pos];
            return Some((MdLineFragment::Italic(content), pos + 1 - start));
        }
        pos += 1;
    }
    
    None
}

/// Parse inline code (`code`).
fn parse_inline_code_fragment(text: &str, start: usize) -> Option<(MdLineFragment<'_>, usize)> {
    let bytes = text.as_bytes();
    
    // Don't parse if it's a code block (```)
    if start + 2 < bytes.len() && bytes[start + 1] == b'`' && bytes[start + 2] == b'`' {
        return None;
    }
    
    // Skip opening `
    let content_start = start + 1;
    
    // Find closing ` (but not if it's immediately after, which would be empty inline code)
    let mut pos = content_start;
    while pos < bytes.len() {
        if bytes[pos] == b'`' {
            // Don't return empty inline code
            if pos == content_start {
                return None;
            }
            let content = &text[content_start..pos];
            return Some((MdLineFragment::InlineCode(content), pos + 1 - start));
        }
        pos += 1;
    }
    
    None
}

/// Parse link [text](url).
fn parse_link_fragment(text: &str, start: usize) -> Option<(MdLineFragment<'_>, usize)> {
    let bytes = text.as_bytes();
    
    // Need at least [x](y) = 6 characters
    if start + 5 >= bytes.len() {
        return None;
    }
    
    // Find closing ]
    let mut bracket_end = None;
    let mut pos = start + 1;
    while pos < bytes.len() {
        if bytes[pos] == b']' {
            bracket_end = Some(pos);
            break;
        }
        pos += 1;
    }
    
    let bracket_end = bracket_end?;
    
    // Check for (url) immediately after ]
    if bracket_end + 1 >= bytes.len() || bytes[bracket_end + 1] != b'(' {
        return None;
    }
    
    // Find closing )
    pos = bracket_end + 2;
    while pos < bytes.len() {
        if bytes[pos] == b')' {
            let link_text = &text[start + 1..bracket_end];
            let url = &text[bracket_end + 2..pos];
            
            return Some((
                MdLineFragment::Link(HyperlinkData::new(link_text, url)),
                pos + 1 - start
            ));
        }
        pos += 1;
    }
    
    None
}

/// Parse image ![alt](url).
fn parse_image_fragment(text: &str, start: usize) -> Option<(MdLineFragment<'_>, usize)> {
    // Skip the ! and parse as if it's a link starting with [
    if let Some((fragment, consumed)) = parse_link_fragment(text, start + 1) {
        match fragment {
            MdLineFragment::Link(hyperlink) => {
                return Some((MdLineFragment::Image(hyperlink), consumed + 1));
            }
            _ => {}
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_kv() {
        let line = GCString::from("@title: My Great Title");
        assert_eq!(parse_line_kv(TITLE, &line), Some("My Great Title"));

        let line = GCString::from("@date: 2025-01-01");
        assert_eq!(parse_line_kv(DATE, &line), Some("2025-01-01"));

        let line = GCString::from("@title: ");
        assert_eq!(parse_line_kv(TITLE, &line), None);

        let line = GCString::from("Not a title");
        assert_eq!(parse_line_kv(TITLE, &line), None);
    }

    #[test]
    fn test_parse_line_csv() {
        let line = GCString::from("@tags: rust, parser, markdown");
        let result = parse_line_csv(TAGS, &line);
        assert!(result.is_some());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], "rust");
        assert_eq!(tags[1], "parser");
        assert_eq!(tags[2], "markdown");

        let line = GCString::from("@authors: Alice, Bob");
        let result = parse_line_csv(AUTHORS, &line);
        assert!(result.is_some());
        let authors = result.unwrap();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0], "Alice");
        assert_eq!(authors[1], "Bob");
    }

    #[test]
    fn test_parse_heading() {
        let line = GCString::from("# Heading 1");
        let result = parse_heading(&line);
        assert!(result.is_some());
        let heading = result.unwrap();
        assert_eq!(heading.heading_level.level, 1);
        assert_eq!(heading.text, "Heading 1");

        let line = GCString::from("### Heading 3");
        let result = parse_heading(&line);
        assert!(result.is_some());
        let heading = result.unwrap();
        assert_eq!(heading.heading_level.level, 3);
        assert_eq!(heading.text, "Heading 3");

        let line = GCString::from("#NoSpace");
        assert!(parse_heading(&line).is_none());

        let line = GCString::from("####### Too many");
        assert!(parse_heading(&line).is_none());
    }
    
    #[test]
    fn test_parse_text_fragments() {
        // Test plain text
        let line = GCString::from("Just plain text");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 1);
        match &fragments[0] {
            MdLineFragment::Plain(text) => assert_eq!(*text, "Just plain text"),
            _ => panic!("Expected plain text"),
        }
        
        // Test bold text
        let line = GCString::from("This is **bold** text");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 3);
        match &fragments[0] {
            MdLineFragment::Plain(text) => assert_eq!(*text, "This is "),
            _ => panic!("Expected plain text"),
        }
        match &fragments[1] {
            MdLineFragment::Bold(text) => assert_eq!(*text, "bold"),
            _ => panic!("Expected bold text"),
        }
        match &fragments[2] {
            MdLineFragment::Plain(text) => assert_eq!(*text, " text"),
            _ => panic!("Expected plain text"),
        }
        
        // Test italic with underscore
        let line = GCString::from("This is _italic_ text");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 3);
        match &fragments[1] {
            MdLineFragment::Italic(text) => assert_eq!(*text, "italic"),
            _ => panic!("Expected italic text"),
        }
        
        // Test italic with asterisk
        let line = GCString::from("This is *italic* text");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 3);
        match &fragments[1] {
            MdLineFragment::Italic(text) => assert_eq!(*text, "italic"),
            _ => panic!("Expected italic text"),
        }
        
        // Test inline code
        let line = GCString::from("Use `code` here");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 3);
        match &fragments[1] {
            MdLineFragment::InlineCode(text) => assert_eq!(*text, "code"),
            _ => panic!("Expected inline code"),
        }
        
        // Test link
        let line = GCString::from("Check [this link](https://example.com)");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 2);
        match &fragments[1] {
            MdLineFragment::Link(link) => {
                assert_eq!(link.text, "this link");
                assert_eq!(link.url, "https://example.com");
            }
            _ => panic!("Expected link"),
        }
        
        // Test image
        let line = GCString::from("![alt text](image.png)");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 1);
        match &fragments[0] {
            MdLineFragment::Image(image) => {
                assert_eq!(image.text, "alt text");
                assert_eq!(image.url, "image.png");
            }
            _ => panic!("Expected image"),
        }
    }
    
    #[test]
    fn test_parse_checkbox_fragments() {
        // Test unchecked checkbox
        let text = "[ ] todo item";
        let fragments = parse_text_fragments_from_str(text);
        assert_eq!(fragments.len(), 2);
        match &fragments[0] {
            MdLineFragment::Checkbox(checked) => assert_eq!(*checked, false),
            _ => panic!("Expected unchecked checkbox"),
        }
        match &fragments[1] {
            MdLineFragment::Plain(text) => assert_eq!(*text, " todo item"),
            _ => panic!("Expected plain text"),
        }
        
        // Test checked checkbox
        let text = "[x] done item";
        let fragments = parse_text_fragments_from_str(text);
        assert_eq!(fragments.len(), 2);
        match &fragments[0] {
            MdLineFragment::Checkbox(checked) => assert_eq!(*checked, true),
            _ => panic!("Expected checked checkbox"),
        }
        match &fragments[1] {
            MdLineFragment::Plain(text) => assert_eq!(*text, " done item"),
            _ => panic!("Expected plain text"),
        }
        
        // Test checkbox with fragments
        let text = "[ ] this is **bold** text";
        let fragments = parse_text_fragments_from_str(text);
        assert_eq!(fragments.len(), 4);
        match &fragments[0] {
            MdLineFragment::Checkbox(checked) => assert_eq!(*checked, false),
            _ => panic!("Expected unchecked checkbox"),
        }
        match &fragments[2] {
            MdLineFragment::Bold(text) => assert_eq!(*text, "bold"),
            _ => panic!("Expected bold text"),
        }
        
        // Test mixed content
        let line = GCString::from("**Bold** and *italic* with `code`");
        let fragments = parse_text_fragments(&line);
        assert_eq!(fragments.len(), 5);
        match &fragments[0] {
            MdLineFragment::Bold(text) => assert_eq!(*text, "Bold"),
            _ => panic!("Expected bold"),
        }
        match &fragments[2] {
            MdLineFragment::Italic(text) => assert_eq!(*text, "italic"),
            _ => panic!("Expected italic"),
        }
        match &fragments[4] {
            MdLineFragment::InlineCode(text) => assert_eq!(*text, "code"),
            _ => panic!("Expected code"),
        }
    }
    
    #[test]
    fn test_parse_smart_list() {
        // Test unordered list
        let lines = vec![
            GCString::from("- First item"),
            GCString::from("- Second item"),
            GCString::from("  with continuation"),
            GCString::from("- Third item"),
        ];
        
        let mut state = ParserState::new(&lines);
        let result = try_parse_smart_list(&mut state);
        assert!(result.is_some());
        
        let (list_lines, bullet_kind, indent) = result.unwrap();
        assert_eq!(indent, 0);
        assert!(matches!(bullet_kind, BulletKind::Unordered));
        assert_eq!(list_lines.len(), 4);
        
        // Check first item
        assert!(matches!(&list_lines[0][0], MdLineFragment::UnorderedListBullet { indent: 0, is_first_line: true }));
        assert!(matches!(&list_lines[0][1], MdLineFragment::Plain("First item")));
        
        // Check continuation line
        assert!(matches!(&list_lines[2][0], MdLineFragment::UnorderedListBullet { indent: 0, is_first_line: false }));
        
        // Test ordered list
        let lines = vec![
            GCString::from("1. First item"),
            GCString::from("2. Second item"),
            GCString::from("3. Third item"),
        ];
        
        let mut state = ParserState::new(&lines);
        let result = try_parse_smart_list(&mut state);
        assert!(result.is_some());
        
        let (list_lines, bullet_kind, indent) = result.unwrap();
        assert_eq!(indent, 0);
        assert!(matches!(bullet_kind, BulletKind::Ordered(1)));
        assert_eq!(list_lines.len(), 3);
        
        // Check ordered list items
        assert!(matches!(&list_lines[0][0], MdLineFragment::OrderedListBullet { indent: 0, number: 1, is_first_line: true }));
        assert!(matches!(&list_lines[1][0], MdLineFragment::OrderedListBullet { indent: 0, number: 2, is_first_line: true }));
        
        // Test indented list
        let lines = vec![
            GCString::from("  - Indented item"),
            GCString::from("  - Another item"),
        ];
        
        let mut state = ParserState::new(&lines);
        let result = try_parse_smart_list(&mut state);
        assert!(result.is_some());
        
        let (list_lines, bullet_kind, indent) = result.unwrap();
        assert_eq!(indent, 2);
        assert!(matches!(bullet_kind, BulletKind::Unordered));
        assert_eq!(list_lines.len(), 2);
    }
    
    #[test]
    fn test_parse_markdown_simple_integration() {
        let lines = vec![
            GCString::from("@title: Test Document"),
            GCString::from("@date: 2025-01-01"),
            GCString::from("@tags: test, markdown, parser"),
            GCString::from("@authors: Alice, Bob"),
            GCString::from(""),
            GCString::from("# Main Heading"),
            GCString::from(""),
            GCString::from("Some **bold** and *italic* text."),
            GCString::from(""),
            GCString::from("## Sub Heading"),
            GCString::from("More text with `code` and [link](https://example.com)."),
            GCString::from(""),
            GCString::from("- List item 1"),
            GCString::from("- List item 2"),
            GCString::from("  with continuation"),
            GCString::from(""),
            GCString::from("```rust"),
            GCString::from("fn main() {"),
            GCString::from("    println!(\"Hello\");"),
            GCString::from("}"),
            GCString::from("```"),
            GCString::from("Final line."),
        ];
        
        let document = parse_markdown_simple(&lines);
        
        // Debug: print document count
        println!("Document has {} elements", document.len());
        for (i, element) in document.iter().enumerate() {
            println!("{}: {:?}", i, match element {
                MdElement::Title(_) => "Title",
                MdElement::Date(_) => "Date",
                MdElement::Tags(_) => "Tags",
                MdElement::Authors(_) => "Authors",
                MdElement::Text(_) => "Text",
                MdElement::Heading(_) => "Heading",
                MdElement::CodeBlock(_) => "CodeBlock",
                MdElement::SmartList(_) => "SmartList",
            });
        }
        // for (i, elem) in document.iter().enumerate() {
        //     println!("{}: {:?}", i, std::mem::discriminant(elem));
        // }
        
        assert_eq!(document.len(), 16); // Updated for new list with 3 lines
        
        // Check metadata
        match &document[0] {
            MdElement::Title(title) => assert_eq!(*title, "Test Document"),
            _ => panic!("Expected Title"),
        }
        
        match &document[1] {
            MdElement::Date(date) => assert_eq!(*date, "2025-01-01"),
            _ => panic!("Expected Date"),
        }
        
        match &document[2] {
            MdElement::Tags(tags) => {
                assert_eq!(tags.len(), 3);
                assert_eq!(tags[0], "test");
                assert_eq!(tags[1], "markdown");
                assert_eq!(tags[2], "parser");
            }
            _ => panic!("Expected Tags"),
        }
        
        match &document[3] {
            MdElement::Authors(authors) => {
                assert_eq!(authors.len(), 2);
                assert_eq!(authors[0], "Alice");
                assert_eq!(authors[1], "Bob");
            }
            _ => panic!("Expected Authors"),
        }
        
        // Check empty line
        match &document[4] {
            MdElement::Text(fragments) => assert_eq!(fragments.len(), 0),
            _ => panic!("Expected empty Text"),
        }
        
        // Check heading
        match &document[5] {
            MdElement::Heading(heading) => {
                assert_eq!(heading.heading_level.level, 1);
                assert_eq!(heading.text, "Main Heading");
            }
            _ => panic!("Expected Heading"),
        }
        
        // Check text with fragments
        match &document[7] {
            MdElement::Text(fragments) => {
                assert_eq!(fragments.len(), 5);
                match &fragments[1] {
                    MdLineFragment::Bold(text) => assert_eq!(*text, "bold"),
                    _ => panic!("Expected bold fragment"),
                }
                match &fragments[3] {
                    MdLineFragment::Italic(text) => assert_eq!(*text, "italic"),
                    _ => panic!("Expected italic fragment"),
                }
            }
            _ => panic!("Expected Text with fragments"),
        }
        
        // Check smart list at index 12
        match &document[12] {
            MdElement::SmartList((lines, bullet_kind, indent)) => {
                assert_eq!(*indent, 0);
                assert!(matches!(bullet_kind, r3bl_tui::BulletKind::Unordered));
                assert_eq!(lines.len(), 3);
            }
            _ => panic!("Expected SmartList at index 12"),
        }
        
        // Check code block at index 14
        match &document[14] {
            MdElement::CodeBlock(lines) => {
                assert_eq!(lines.len(), 5); // Start tag, 3 content lines, end tag
                match &lines[0].content {
                    CodeBlockLineContent::StartTag => {},
                    _ => panic!("Expected StartTag"),
                }
                match &lines[1].content {
                    CodeBlockLineContent::Text(text) => assert_eq!(*text, "fn main() {"),
                    _ => panic!("Expected Text"),
                }
                match &lines[4].content {
                    CodeBlockLineContent::EndTag => {},
                    _ => panic!("Expected EndTag"),
                }
            }
            _ => panic!("Expected CodeBlock at index 14"),
        }
    }
    
    #[test]
    fn test_parse_smart_list_with_checkboxes() {
        let lines = vec![
            GCString::from("- [ ] First unchecked item"),
            GCString::from("- [x] Second checked item"),
            GCString::from("  Continuation line"),
            GCString::from("- [ ] Third item with **bold** text"),
        ];
        
        let document = parse_markdown_simple(&lines);
        assert_eq!(document.len(), 1);
        
        match &document[0] {
            MdElement::SmartList((lines, bullet_kind, indent)) => {
                assert_eq!(*indent, 0);
                assert!(matches!(bullet_kind, r3bl_tui::BulletKind::Unordered));
                assert_eq!(lines.len(), 4);
                
                // Check first line with unchecked checkbox
                let first_line = &lines[0];
                assert_eq!(first_line.len(), 3);
                match &first_line[1] {
                    MdLineFragment::Checkbox(checked) => assert_eq!(*checked, false),
                    _ => panic!("Expected unchecked checkbox"),
                }
                
                // Check second line with checked checkbox
                let second_line = &lines[1];
                match &second_line[1] {
                    MdLineFragment::Checkbox(checked) => assert_eq!(*checked, true),
                    _ => panic!("Expected checked checkbox"),
                }
                
                // Check fourth line with checkbox and bold text
                let fourth_line = &lines[3];
                match &fourth_line[1] {
                    MdLineFragment::Checkbox(checked) => assert_eq!(*checked, false),
                    _ => panic!("Expected unchecked checkbox"),
                }
                let mut found_bold = false;
                for fragment in fourth_line.iter() {
                    if let MdLineFragment::Bold(text) = fragment {
                        assert_eq!(*text, "bold");
                        found_bold = true;
                    }
                }
                assert!(found_bold, "Expected to find bold text");
            }
            _ => panic!("Expected SmartList"),
        }
    }
}