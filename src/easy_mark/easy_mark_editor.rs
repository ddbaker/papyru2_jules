use bevy_egui::{egui};
use egui::{
    text::CCursorRange, Key, KeyboardShortcut, Modifiers, ScrollArea, TextBuffer, Ui,
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EasyMarkEditor {
    pub code: String,
    highlight_editor: bool,
    show_rendered: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub highlighter: super::easy_mark_highlighter::MemoizedEasymarkHighlighter,
}

impl PartialEq for EasyMarkEditor {
    fn eq(&self, other: &Self) -> bool {
        (&self.code, self.highlight_editor, self.show_rendered)
            == (&other.code, other.highlight_editor, other.show_rendered)
    }
}

impl Default for EasyMarkEditor {
    fn default() -> Self {
        Self {
            code: DEFAULT_CODE.trim().to_owned(),
            highlight_editor: true,
            show_rendered: true,
            highlighter: Default::default(),
        }
    }
}

impl EasyMarkEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("controls").show(ui, |ui| {
            let _response = ui.button("Hotkeys").on_hover_ui(nested_hotkeys_ui);
            ui.checkbox(&mut self.show_rendered, "Show rendered"); // Label might need update
            ui.checkbox(&mut self.highlight_editor, "Highlight editor");
            if ui.button("Reset").clicked() {
                *self = Default::default();
            }
            ui.end_row();
        });
        ui.separator();

        if self.show_rendered {
            let available_height = ui.available_height(); // Calculate *before* columns closure
            ui.columns(2, |columns| {
                // Column 0: Editor (Left)
                ScrollArea::vertical()
                    .id_salt(egui::Id::new("editor_scroll_area_side"))  // Swapped content
                    .min_scrolled_height(available_height)
                    .auto_shrink([false, false])
                    .show(&mut columns[0], |ui_editor| { // columns[0] is now editor
                        self.editor_ui(ui_editor);
                    });

                // Column 1: Rendered View (Right)
                ScrollArea::vertical()
                    .id_salt(egui::Id::new("rendered_scroll_area_side")) // Swapped content
                    .min_scrolled_width(100.0)
                    .min_scrolled_height(available_height)
                    .auto_shrink([false, false])
                    .show(&mut columns[1], |ui_viewer| { // columns[1] is now viewer
                        super::easy_mark_viewer::easy_mark(ui_viewer, &self.code);
                    });
            });
        } else {
            // Single panel for editor only
            ScrollArea::vertical()
                .id_salt(egui::Id::new("editor_scroll_area_single"))
                .auto_shrink([false, false]) // Ensure it expands fully
                .show(ui, |ui_editor| {
                    self.editor_ui(ui_editor);
                });
        }
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            code, highlighter, ..
        } = self;

        let _response = if self.highlight_editor {
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = highlighter.highlight(ui.style(), string);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            let text_edit = egui::TextEdit::multiline(code)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .layouter(&mut layouter);
            ui.add(text_edit)
        } else {
            let text_edit = egui::TextEdit::multiline(code)
                .desired_width(f32::INFINITY);
            ui.add(text_edit)
        };
    }
}

pub const SHORTCUT_BOLD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::B);
pub const SHORTCUT_CODE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const SHORTCUT_ITALICS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
pub const SHORTCUT_SUBSCRIPT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::L); // Assuming $ for subscript
pub const SHORTCUT_SUPERSCRIPT: KeyboardShortcut = // Assuming ^ for superscript
    KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);
pub const SHORTCUT_STRIKETHROUGH: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Q); // Assuming ~
pub const SHORTCUT_UNDERLINE: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::W); // Assuming _
pub const SHORTCUT_INDENT: KeyboardShortcut = // Placeholder, actual indent/outdent might be different
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::E);

fn nested_hotkeys_ui(ui: &mut egui::Ui) {
    egui::Grid::new("shortcuts").striped(true).show(ui, |ui| {
        let mut label = |shortcut, what| {
            ui.label(what);
            ui.weak(ui.ctx().format_shortcut(&shortcut));
            ui.end_row();
        };

        label(SHORTCUT_BOLD, "*bold*");
        label(SHORTCUT_CODE, "`code`");
        label(SHORTCUT_ITALICS, "/italics/");
        // Assuming $ for subscript based on EasyMark example, adjust if different
        label(SHORTCUT_SUBSCRIPT, "$small$"); // Or what $ represents if not subscript
        // Assuming ^ for superscript
        label(SHORTCUT_SUPERSCRIPT, "^raised^"); // Or what ^ represents if not superscript
        label(SHORTCUT_STRIKETHROUGH, "~strikethrough~");
        label(SHORTCUT_UNDERLINE, "_underline_");
        label(SHORTCUT_INDENT, "two spaces");
    });
}

#[allow(dead_code)]
fn shortcuts(ui: &Ui, code: &mut dyn TextBuffer, ccursor_range: &mut CCursorRange) -> bool {
    let mut any_change = false;

    if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_INDENT)) {
        any_change = true;
        let primary = ccursor_range.primary;
        let advance = code.insert_text("  ", primary.index);
        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }

    for (shortcut, surrounding) in [
        (SHORTCUT_BOLD, "*"),
        (SHORTCUT_CODE, "`"),
        (SHORTCUT_ITALICS, "/"),
        (SHORTCUT_SUBSCRIPT, "$"), // Assuming $
        (SHORTCUT_SUPERSCRIPT, "^"), // Assuming ^
        (SHORTCUT_STRIKETHROUGH, "~"),
        (SHORTCUT_UNDERLINE, "_"),
    ] {
        if ui.input_mut(|i| i.consume_shortcut(&shortcut)) {
            any_change = true;
            toggle_surrounding(code, ccursor_range, surrounding);
        };
    }

    any_change
}

#[allow(dead_code)]
fn toggle_surrounding(
    code: &mut dyn TextBuffer,
    ccursor_range: &mut CCursorRange,
    surrounding: &str,
) {
    let primary = ccursor_range.primary;
    let secondary = ccursor_range.secondary;

    let surrounding_ccount = surrounding.chars().count();

    let prefix_crange = primary.index.saturating_sub(surrounding_ccount)..primary.index;
    let suffix_crange = secondary.index..secondary.index.saturating_add(surrounding_ccount);
    let already_surrounded = code.char_range(prefix_crange.clone()) == surrounding
        && code.char_range(suffix_crange.clone()) == surrounding;

    if already_surrounded {
        code.delete_char_range(suffix_crange);
        code.delete_char_range(prefix_crange);
        ccursor_range.primary.index -= surrounding_ccount;
        ccursor_range.secondary.index -= surrounding_ccount;
    } else {
        code.insert_text(surrounding, secondary.index);
        let advance = code.insert_text(surrounding, primary.index);

        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
}

const DEFAULT_CODE: &str = r#"
# EasyMark
EasyMark is a markup language, designed for extreme simplicity.
```
WARNING: EasyMark is still an evolving specification,
and is also missing some features.
```
----------------

# At a glance
- inline text:
  - normal, `code`, *strong*, ~strikethrough~, _underline_, /italics/, ^raised^, $small$
  - `\` escapes the next character
  - [hyperlink](https://github.com/emilk/egui)
  - Embedded URL: <https://github.com/emilk/egui>
- `# ` header
- `---` separator (horizontal line)
- `> ` quote
- `- ` bullet list
- `1. ` numbered list
- ``` code fence
- a^2^ + b^2^ = c^2^
- $Remember to read the small print$
# Design
> /"Why do what everyone else is doing, when everyone else is already doing it?"
>   \- Emil
Goals:
1. easy to parse
2. easy to learn
3. similar to markdown
[The reference parser](https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/easy_mark/easy_mark_parser.rs) is \~250 lines of code, using only the Rust standard library. The parser uses no look-ahead or recursion.
There is never more than one way to accomplish the same thing, and each special character is only used for one thing. For instance `*` is used for *strong* and `-` is used for bullet lists. There is no alternative way to specify the *strong * style or getting a bullet list.
Similarity to markdown is kept when possible, but with much less ambiguity and s ome improvements (like _underlining_).
# Details
All style changes are single characters, so it is `*strong*`, NOT `**strong**`. Style is reset by a matching character, or at the end of the line.
Style change characters and escapes (`\`) work everywhere except for in inline code, code blocks and in URLs.
You can mix styles. For instance: /italics _underline_/ and *strong `code`*.
You can use styles on URLs: ~my webpage is at <http://www.example.com>~.
Newlines are preserved. If you want to continue text on the same line, just do so. Alternatively, escape the newline by ending the line with a backslash (`\`). Escaping the newline effectively ignores it.
The style characters are chosen to be similar to what they are representing:
  `_` = _underline_
  `~` = ~strikethrough~ (`-` is used for bullet points)
  `/` = /italics/
  `*` = *strong*
  `$` = $small$
  `^` = ^raised^
# To do
- Sub-headers (`## h2`, `### h3` etc)
- Hotkey Editor
- International keyboard algorithm for non-letter keys
- ALT+SHIFT+Num1 is not a functioning hotkey
- Tab Indent Increment/Decrement CTRL+], CTRL+[
- Images
  - we want to be able to optionally specify size (width and\/or height)
  - centering of images is very desirable
  - captioning (image with a text underneath it)
  - `![caption=My image][width=200][center](url)` ?
- Nicer URL:s
  - `<url>` and `[url](url)` do the same thing yet look completely different.
  - let's keep similarity with images
- Tables
- Inspiration: <https://mycorrhiza.wiki/help/en/mycomarkup>
"#;
