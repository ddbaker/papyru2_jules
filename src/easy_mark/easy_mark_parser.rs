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
    /// `\n`
    // TODO(emilk): add Style here so empty heading still uses up the right amount of space.
    Newline,

    /// Text
    Text(Style, &'a str),

    /// title, url
    Hyperlink(Style, &'a str, &'a str),

    /// leading space before e.g. a [`Self::BulletPoint`].
    Indentation(usize),

    /// >
    QuoteIndent,

    /// - a point well made.
    BulletPoint,

    /// 1. numbered list. The string is the number(s).
    NumberedPoint(&'a str),

    /// ---
    Separator,

    /// language, code
    CodeBlock(&'a str, &'a str),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Style {
    /// # heading (large text)
    pub heading: bool,

    /// > quoted (slightly dimmer color or other font style)
    pub quoted: bool,

    /// `code` (monospace, some other color)
    pub code: bool,

    /// self.strong* (emphasized, e.g. bold)
    pub strong: bool,

    /// _underline_
    pub underline: bool,

    /// ~strikethrough~
    pub strikethrough: bool,

    /// /italics/
    pub italics: bool,

    /// $small$
    pub small: bool,

    /// ^raised^
    pub raised: bool,
}

/// Parser for the `EasyMark` markup language.
///
/// See the module-level documentation for details.
///
/// # Example:
/// ```
/// # use egui_demo_lib::easy_mark::parser::Parser;
/// for item in Parser::new("Hello *world*!") {
/// }
///
/// ```
pub struct Parser<'a> {
    /// The remainder of the input text
    s: &'a str,

    /// Are we at the start of a line?
    start_of_line: bool,

    /// Current self.style. Reset after a newline.
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

    /// `1. `, `42. ` etc.
    fn numbered_list(&mut self) -> Option<Item<'a>> {
        let n_digits = self.s.chars().take_while(|c| c.is_ascii_digit()).count()
;
        if n_digits > 0 && self.s.chars().skip(n_digits).take(2).eq(". ".chars()
) {
            let number = &self.s[..n_digits];
            self.s = &self.s[(n_digits + 2)..];
            self.start_of_line = false;
            return Some(Item::NumberedPoint(number));
        }
        None
    }

    // ```{language}\n{code}``` <--- This comment describes the code block structure
    fn code_block(&mut self) -> Option<Item<'a>> {
        if let Some(language_start) = self.s.strip_prefix("```") {
            if let Some(newline) = language_start.find('\n') {
                let language = &language_start[..newline];
                let code_start = &language_start[newline + 1..];
                if let Some(end) = code_start.find("\n```") {
                    let code = &code_start[..end].trim();
                    self.s = &code_start[end + 4..];
                    self.start_of_line = false;
                    return Some(Item::CodeBlock(language, code));
                } else {
                    self.s = "";
                    return Some(Item::CodeBlock(language, code_start));
                }
            }
        }
        None
    }

    // `code`
    fn inline_code(&mut self) -> Option<Item<'a>> {
        if let Some(rest) = self.s.strip_prefix('`') {
            self.s = rest;
            self.start_of_line = false;
            self.style.code = true;
            let rest_of_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(end) = rest_of_line.find('`') {
                let item = Item::Text(self.style, &self.s[..end]);
                self.s = &self.s[end + 1..];
                self.style.code = false;
                return Some(item);
            } else {
                let end = rest_of_line.len();
                let item = Item::Text(self.style, rest_of_line);
                self.s = &self.s[end..];
                self.style.code = false;
                return Some(item);
            }
        }
        None
    }

    /// `<url>` or `[link](url)`
    fn url(&mut self) -> Option<Item<'a>> {
        if self.s.starts_with('<') {
            let this_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(url_end) = this_line.find('>') {
                let url = &self.s[1..url_end];
                self.s = &self.s[url_end + 1..];
                self.start_of_line = false;
                return Some(Item::Hyperlink(self.style, url, url));
            }
        }

        // [text](url)
        if self.s.starts_with('[') {
            let this_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(bracket_end) = this_line.find(']') {
                let text = &this_line[1..bracket_end];
                if this_line[bracket_end + 1..].starts_with('(') {
                    if let Some(parens_end) = this_line[bracket_end + 2..].find(
')') {
                        let parens_end = bracket_end + 2 + parens_end;
                        let url = &self.s[bracket_end + 2..parens_end];
                        self.s = &self.s[parens_end + 1..];
                        self.start_of_line = false;
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
        loop {
            println!("[PARSER LoopTop] s: '{:?}', start_of_line: {}", self.s.chars().take(30).collect::<String>(), self.start_of_line);
            if self.s.is_empty() {
                // println!("Parser: Reached end of input."); // Silenced for now
                return None;
            }
            // The following line was intended to be commented out but was active.
            // println!("Parser: Current input slice: {:?}", self.s.chars().take(30).collect::<String>());

            // The rest of the next() function logic will go here, assigning to item_to_return
            // For example, when a newline is found:
            // if self.s.starts_with('\n') {
            //     self.s = &self.s[1..];
            //     self.start_of_line = true;
            //     self.style = Style::default();
            //     item_to_return = Some(Item::Newline);
            // }
            // ... and so on for all other item types

            // Original logic continues here, eventually setting item_to_return
            // For brevity, I'm not copying the entire function content here.
            // We will wrap the `return Some(item)` with this logic.

            //

            if self.s.starts_with('\n') {
                self.s = &self.s[1..];
                self.start_of_line = true;
                self.style = Style::default();
                let item = Item::Newline;
                // println!("Parser: Yielding Newline: {:?}", item); // Silenced for now
                return Some(item);
            }

            // Ignore line break (continue on the same line)
            if self.s.starts_with("\
") && self.s.len() >= 2 {
                self.s = &self.s[2..];
                self.start_of_line = false;
                continue;
            }

            // \ escape (to show e.g. a backtick)
            if self.s.starts_with('\\') && self.s.len() >= 2 {
                let text = &self.s[1..2];
                self.s = &self.s[2..];
                self.start_of_line = false;
                // println!("Parser: Yielding Escaped Text: {:?}", text); // Silenced for now
                return Some(Item::Text(self.style, text));
            }

            if self.start_of_line {
                // leading space (indentation)
                if self.s.starts_with(' ') {
                    let length = self.s.find(|c| c != ' ').unwrap_or(self.s.len(
));
                    self.s = &self.s[length..];
                    self.start_of_line = true; // indentation doesn't count
                    let item = Item::Indentation(length);
                    // println!("Parser: Yielding Indentation: {:?}", item); // Silenced for now
                    return Some(item);
                }

                // # Heading
                if let Some(s_after_hash) = self.s.strip_prefix('#') {
                    // Check for common whitespace (space or tab)
                    if !s_after_hash.is_empty() && (s_after_hash.starts_with(' ') || s_after_hash.starts_with('\t')) {
                        let after_trimmed_whitespace = s_after_hash.trim_start();
                        println!("[PARSER #] Detected heading. `s_after_hash`: '{:?}', `after_trimmed_whitespace`: '{:?}'",
                                 s_after_hash.chars().take(30).collect::<String>(),
                                 after_trimmed_whitespace.chars().take(30).collect::<String>());
                        self.s = after_trimmed_whitespace;
                        self.start_of_line = false;
                        self.style.heading = true;
                        continue;
                    }
                }

                // > quote
                if let Some(after) = self.s.strip_prefix("> ") {
                    self.s = after;
                    self.start_of_line = true; // quote indentation doesn't count
                    self.style.quoted = true;
                    let item = Item::QuoteIndent;
                    // println!("Parser: Yielding QuoteIndent: {:?}", item); // Silenced for now
                    return Some(item);
                }

                // - bullet point
                if self.s.starts_with("- ") {
                    self.s = &self.s[2..];
                    self.start_of_line = false;
                    let item = Item::BulletPoint;
                    // println!("Parser: Yielding BulletPoint: {:?}", item); // Silenced for now
                    return Some(item);
                }

                // `1. `, `42. ` etc.
                if let Some(item) = self.numbered_list() {
                    // println!("Parser: Yielding NumberedPoint (from helper): {:?}", item); // Silenced for now
                    return Some(item);
                }

                // --- separator
                if let Some(after) = self.s.strip_prefix("---") {
                    self.s = after.trim_start_matches('-'); // remove extra dashes
                    self.s = self.s.strip_prefix('\n').unwrap_or(self.s); // remove trailing newline
                    self.start_of_line = false;
                    let item = Item::Separator;
                    // println!("Parser: Yielding Separator: {:?}", item); // Silenced for now
                    return Some(item);
                }

                // ```{language}\n{code}``` <--- This comment describes the code block structure
                if let Some(item) = self.code_block() {
                    // println!("Parser: Yielding CodeBlock (from helper): {:?}", item); // Silenced for now
                    return Some(item);
                }
            }

            // `code`
            if let Some(item) = self.inline_code() {
                // println!("Parser: Yielding Text (from inline_code helper): {:?}", item); // Silenced for now
                return Some(item);
            }

            if let Some(rest) = self.s.strip_prefix('*') {
                self.s = rest;
                self.start_of_line = false;
                self.style.strong = !self.style.strong;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('_') {
                self.s = rest;
                self.start_of_line = false;
                self.style.underline = !self.style.underline;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('~') {
                self.s = rest;
                self.start_of_line = false;
                self.style.strikethrough = !self.style.strikethrough;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('/') {
                self.s = rest;
                self.start_of_line = false;
                self.style.italics = !self.style.italics;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('$') {
                self.s = rest;
                self.start_of_line = false;
                self.style.small = !self.style.small;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('^') {
                self.s = rest;
                self.start_of_line = false;
                self.style.raised = !self.style.raised;
                continue;
            }

            // `<url>` or `[link](url)`
            if let Some(item) = self.url() {
                // println!("Parser: Yielding Hyperlink (from url helper): {:?}", item); // Silenced for now
                return Some(item);
            }

            // Swallow everything up to the next special character:
            let find_result = self.s.find(&['*', '`', '~', '_', '/', '$', '^', '\\', '<', '[', '\n'][..]);
            let end = find_result.unwrap_or(self.s.len()); // Simplified end calculation for now

            println!("[PARSER Swallow] s: '{:?}', find_result: {:?}, calculated_end: {}",
                     self.s.chars().take(30).collect::<String>(),
                     find_result,
                     end);

            let item = Item::Text(self.style, &self.s[..end]);
            self.s = &self.s[end..];
            self.start_of_line = false;
            if let Item::Text(style, text_content) = &item {
                 println!("[PARSER Text] Yielding Text (default): content='{}', style={:?}", text_content, style);
            }
            return Some(item);
        }
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
