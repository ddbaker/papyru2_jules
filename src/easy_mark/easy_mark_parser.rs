//! A parser for `EasyMark`: a very simple markup language.
//!
//! WARNING: `EasyMark` is subject to change.
//
//! # `EasyMark` design goals:
//! 1. easy to parse
//! 2. easy to learn
//! 3. similar to markdown

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Item<'a> {
    Newline,
    Text(Style, &'a str),
    Hyperlink(Style, &'a str, &'a str),
    Indentation(usize),
    QuoteIndent,
    BulletPoint,
    NumberedPoint(&'a str),
    Separator,
    CodeBlock(&'a str, &'a str),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Style {
    pub heading: bool,
    pub quoted: bool,
    pub code: bool,
    pub strong: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub italics: bool,
    pub small: bool,
    pub raised: bool,
}

pub struct Parser<'a> {
    s: &'a str,
    start_of_line: bool,
    style: Style,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            start_of_line: true,
            style: Style::default(),
        }
    }

    fn numbered_list(&mut self) -> Option<Item<'a>> {
        let n_digits = self.s.chars().take_while(|c| c.is_ascii_digit()).count();
        if n_digits > 0 && self.s.chars().skip(n_digits).take(2).eq(". ".chars()) {
            let number = &self.s[..n_digits];
            self.s = &self.s[(n_digits + 2)..];
            // self.start_of_line = false; // Will be set by caller or next text
            return Some(Item::NumberedPoint(number));
        }
        None
    }

    fn code_block(&mut self) -> Option<Item<'a>> {
        if let Some(language_start) = self.s.strip_prefix("```") {
            if let Some(newline) = language_start.find('\n') {
                let language = &language_start[..newline];
                let code_start = &language_start[newline + 1..];
                if let Some(end) = code_start.find("\n```") {
                    let code = &code_start[..end].trim();
                    self.s = &code_start[end + 4..];
                    // self.start_of_line = false; // Will be set by caller or next text
                    return Some(Item::CodeBlock(language, code));
                } else {
                    self.s = ""; // Consume rest of string
                    return Some(Item::CodeBlock(language, code_start.trim_end_matches('\n')));
                }
            }
        }
        None
    }

    fn inline_code(&mut self) -> Option<Item<'a>> {
        if let Some(rest) = self.s.strip_prefix('`') {
            self.s = rest;
            // self.start_of_line = false; // Will be set by caller or next text
            let mut temp_style = self.style; // Create a temporary style for this item
            temp_style.code = true;

            let rest_of_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(end) = rest_of_line.find('`') {
                let text_content = &self.s[..end];
                self.s = &self.s[end + 1..];
                return Some(Item::Text(temp_style, text_content));
            } else {
                // Unterminated inline code, treat as code until end of line or string
                let text_content = rest_of_line;
                self.s = &self.s[rest_of_line.len()..];
                return Some(Item::Text(temp_style, text_content));
            }
        }
        None
    }

    fn url(&mut self) -> Option<Item<'a>> {
        if self.s.starts_with('<') {
            let this_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(url_end) = this_line.find('>') {
                let url_text = &self.s[1..url_end];
                self.s = &self.s[url_end + 1..];
                // self.start_of_line = false; // Will be set by caller or next text
                return Some(Item::Hyperlink(self.style, url_text, url_text));
            }
        }
        if self.s.starts_with('[') {
            let this_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(bracket_end) = this_line.find(']') {
                let text = &this_line[1..bracket_end];
                if this_line.get(bracket_end + 1..).map_or(false, |r| r.starts_with('(')) {
                     if let Some(parens_end_offset) = this_line.get(bracket_end + 2..).and_then(|r| r.find(')')) {
                        let parens_end = bracket_end + 2 + parens_end_offset;
                        let url = &self.s[bracket_end + 2..parens_end];
                        self.s = &self.s[parens_end + 1..];
                        // self.start_of_line = false; // Will be set by caller or next text
                        return Some(Item::Hyperlink(self.style, text, url));
                    }
                }
            }
        }
        None
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // println!("[PARSER LoopTop] s: '{:?}', start_of_line: {}, style: {:?}", self.s.chars().take(30).collect::<String>(), self.start_of_line, self.style);

        if self.s.is_empty() {
            return None;
        }

        if self.s.starts_with('\n') {
            self.s = &self.s[1..];
            self.start_of_line = true;
            self.style = Style::default();
            return Some(Item::Newline);
        }

        if self.s.starts_with("\\\n") && self.s.len() >= 2 { // Escaped newline
            self.s = &self.s[2..];
            // start_of_line remains whatever it was, effectively continuing the line
            return self.next(); // Effectively a continue, re-evaluate new self.s
        }

        if self.s.starts_with('\\') && self.s.len() >= 2 { // Escaped char
            let text = &self.s[1..2];
            self.s = &self.s[2..];
            self.start_of_line = false;
            return Some(Item::Text(self.style, text));
        }

        if self.start_of_line {
            if self.s.starts_with(' ') { // Indentation
                let length = self.s.find(|c| c != ' ').unwrap_or(self.s.len());
                self.s = &self.s[length..];
                // start_of_line remains true, Indentation doesn't count as content for this flag
                return Some(Item::Indentation(length));
            }

            if let Some(s_after_hash) = self.s.strip_prefix('#') {
                if !s_after_hash.is_empty() && (s_after_hash.starts_with(' ') || s_after_hash.starts_with('\t')) {
                    let after_trimmed_whitespace = s_after_hash.trim_start();
                    // println!("[PARSER #] Detected heading. `s_after_hash`: '{:?}', `after_trimmed_whitespace`: '{:?}'",
                    //          s_after_hash.chars().take(30).collect::<String>(),
                    //          after_trimmed_whitespace.chars().take(30).collect::<String>());
                    self.s = after_trimmed_whitespace;
                    self.start_of_line = false; // Text of heading is not at start_of_line
                    self.style.heading = true;  // Style applies until newline or reset
                    return self.next(); // Re-evaluate to parse the heading text itself
                }
            }

            if let Some(after) = self.s.strip_prefix("> ") {
                self.s = after;
                self.style.quoted = true; // Style applies until newline or reset
                // start_of_line remains true for QuoteIndent itself, text after it will set it false
                return Some(Item::QuoteIndent);
            }
            if self.s.starts_with("- ") {
                self.s = &self.s[2..];
                self.start_of_line = false;
                return Some(Item::BulletPoint);
            }
            if let Some(item) = self.numbered_list() { // numbered_list updates self.s
                self.start_of_line = false;
                return Some(item);
            }
            if let Some(after) = self.s.strip_prefix("---") {
                self.s = after.trim_start_matches('-');
                self.s = self.s.strip_prefix('\n').unwrap_or(self.s); // Consume trailing newline if any
                self.start_of_line = false; // Separator itself makes it not start_of_line
                                           // If it was followed by \n, next will be start_of_line=true
                return Some(Item::Separator);
            }
            if let Some(item) = self.code_block() { // code_block updates self.s
                self.start_of_line = false;
                return Some(item);
            }
            // If still start_of_line and no specific token matched, it's a plain text line
            // This will be handled by the generic text swallowing logic below,
            // after self.start_of_line is set to false.
        }

        // If we reach here, either start_of_line was false, or it was true but no line-start token was found.
        // For the latter case, we now consider it as the start of a regular text segment.
        // This ensures that plain lines like "EasyMark is a markup language" are processed by text swallowing.
        // let mut just_set_start_of_line_false = false; // [JULES] Removed unused variable
        if self.start_of_line {
             self.start_of_line = false;
             // just_set_start_of_line_false = true; // [JULES] Removed unused variable assignment
        }


        // Inline elements or plain text
        if let Some(item) = self.inline_code() { return Some(item); }

        // Style toggles: only if not just_set_start_of_line_false OR if the char is not the first on line
        // This is to prevent a style char at line start from being ONLY a toggle if it's not.
        // However, current EasyMark spec implies style chars always toggle.
        let mut style_changed = false;
        if let Some(rest) = self.s.strip_prefix('*') { self.s = rest; self.style.strong = !self.style.strong; style_changed = true; }
        else if let Some(rest) = self.s.strip_prefix('_') { self.s = rest; self.style.underline = !self.style.underline; style_changed = true; }
        else if let Some(rest) = self.s.strip_prefix('~') { self.s = rest; self.style.strikethrough = !self.style.strikethrough; style_changed = true; }
        else if let Some(rest) = self.s.strip_prefix('/') { self.s = rest; self.style.italics = !self.style.italics; style_changed = true; }
        else if let Some(rest) = self.s.strip_prefix('$') { self.s = rest; self.style.small = !self.style.small; style_changed = true; }
        else if let Some(rest) = self.s.strip_prefix('^') { self.s = rest; self.style.raised = !self.style.raised; style_changed = true; }

        if style_changed {
            // If only a style toggle happened and it resulted in an empty self.s,
            // it implies the style char was at the end. We should yield nothing or handle this.
            // For now, just continue to re-evaluate. If self.s is empty, loop will terminate.
            // If self.s is not empty, it might be a text node or another style toggle.
            if self.s.is_empty() { return None; } // Or perhaps yield previous text if any? Complicated.
            return self.next(); // Re-evaluate with new style and remaining self.s
        }

        if let Some(item) = self.url() { return Some(item); }

        // Fallback: Swallow text up to next special char or end of line/string.
        let special_chars = &['*', '`', '~', '_', '/', '$', '^', '\\', '<', '[', '\n'][..];
        let find_result = self.s.find(special_chars);
        let end = find_result.unwrap_or(self.s.len());

        // println!("[PARSER Swallow] s: '{:?}', find_result: {:?}, calculated_end: {}",
        //          self.s.chars().take(30).collect::<String>(),
        //          find_result,
        //          end);

        if end == 0 { // Found special char at the start, but it wasn't handled by toggles/inline_code/url
                      // This implies it's a lone special char to be treated as text, or start of next structure.
                      // Example: `* ` should be BulletPoint, not Text("*").
                      // This case needs careful handling. If it's truly unhandled, consume 1 char as text.
            // However, this should ideally be caught by specific rules or style toggles.
            // If we are here, it means a single special char is to be output as text.
            // The previous logic `idx.max(1)` handled this.
            // Let's reinstate a similar logic: if end is 0 because a special char is at s[0],
            // and it wasn't a style toggle, then consume just that one char as text.
            let effective_end = if end == 0 && !self.s.is_empty() { 1 } else { end };

            if effective_end > 0 { // Ensure we have something to yield
                let text_slice = &self.s[..effective_end];
                self.s = &self.s[effective_end..];
                // self.start_of_line = false; // Already false or set by now
                // if let Item::Text(style, text_content) = Item::Text(self.style, text_slice) {
                //      println!("[PARSER Text] Yielding Text (fallback, end=0 case): content='{}', style={:?}", text_content, style);
                // }
                return Some(Item::Text(self.style, text_slice));
            } else { // end is 0 and s is empty - should be caught by s.is_empty() at top
                return None;
            }
        }

        // Normal text segment
        let text_slice = &self.s[..end];
        self.s = &self.s[end..];
        // self.start_of_line = false; // Already false or set by now
        // if let Item::Text(style, content) = Item::Text(self.style, text_slice) {
        //      println!("[PARSER Text] Yielding Text (default): content='{}', style={:?}", content, style);
        // }
        return Some(Item::Text(self.style, text_slice));
    }
}

#[test]
fn test_easy_mark_parser() {
    let items: Vec<_> = Parser::new("~strikethrough `code`~").collect();
    assert_eq!(
        items,
        vec![
            Item::Text(
                Style {
                    strikethrough: true,
                    ..Default::default()
                },
                "strikethrough "
            ),
            Item::Text(
                Style {
                    code: true,
                    strikethrough: true,
                    ..Default::default()
                },
                "code"
            ),
        ]
    );
}
