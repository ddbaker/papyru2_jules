// This file will contain the IME functionality.
// Based on https://github.com/8bitTD/bevy_egui_ime

use bevy_egui::egui;
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct ImeManager {
    pub count: usize, // Made public
    pub ime_texts: Vec<ImeText>, // Made public
}

impl Default for ImeManager {
    fn default() -> ImeManager {
        ImeManager {
            count: 0,
            ime_texts: Vec::new(),
        }
    }
}

// NOTE: The get_layoutjob that was here has been removed.
// It is now correctly placed within impl ImeText.

impl ImeText {
    // This is the correctly placed get_layoutjob
    fn get_layoutjob(
        &self,
        full_text_with_ime: &str,
        wrap_width: f32,
        style: &egui::Style,
        highlighter: &mut crate::easy_mark::easy_mark_highlighter::MemoizedEasymarkHighlighter,
        highlight_editor: bool,
    ) -> egui::text::LayoutJob {
        let font_id = egui::FontId::new(14.0, egui::FontFamily::Name("NotoSerifCJKjp-Medium".into()));
        let default_text_color = style.visuals.text_color(); // Store default text color

        if !self.is_ime || self.ime_string.is_empty() {
            // No active IME preedit, or preedit string is empty: use highlighter or simple
            if highlight_editor {
                let mut job = highlighter.highlight(style, full_text_with_ime);
                job.wrap.max_width = wrap_width;
                for section in &mut job.sections {
                    section.format.font_id = font_id.clone();
                }
                job.break_on_newline = matches!(self.edit_type, EditType::MultiLine);
                return job;
            } else {
                let mut job = egui::text::LayoutJob::simple(
                    full_text_with_ime.into(),
                    font_id,
                    default_text_color,
                    wrap_width,
                );
                job.break_on_newline = matches!(self.edit_type, EditType::MultiLine);
                return job;
            }
        }

        // Active IME preedit: construct LayoutJob section by section
        let mut job = egui::text::LayoutJob::default();
        job.text = full_text_with_ime.to_string(); // Set the full text for the job

        // Calculate byte indices for slicing based on char indices
        let text_before_char_end = self.cursor_index;
        let ime_char_end = self.cursor_index + self.ime_string.chars().count();

        let mut char_idx_counter = 0;
        let mut byte_idx_start = 0;

        // Part 1: Text before IME
        for (char_byte_idx, _char) in full_text_with_ime.char_indices() {
            if char_idx_counter == text_before_char_end {
                let text_slice = &full_text_with_ime[byte_idx_start..char_byte_idx];
                if !text_slice.is_empty() {
                    if highlight_editor {
                        let highlighted_segment = highlighter.highlight(style, text_slice); // Removed mut
                        for mut section in highlighted_segment.sections {
                            section.format.font_id = font_id.clone();
                            job.sections.push(section);
                        }
                    } else {
                        job.append(text_slice, 0.0, egui::TextFormat::simple(font_id.clone(), default_text_color));
                    }
                }
                byte_idx_start = char_byte_idx;
                break;
            }
            if char_idx_counter < text_before_char_end { // handle case where loop ends before index
                 char_idx_counter += 1;
            } else { // Should not happen if text_before_char_end is valid
                break;
            }
        }
         // If cursor_index is 0, the loop above won't run for "before" part.
        if text_before_char_end == 0 {
             byte_idx_start = 0; // Ensure byte_idx_start is 0 if no text before.
        } else if char_idx_counter < text_before_char_end { // If string ends before cursor_index
            let text_slice = &full_text_with_ime[byte_idx_start..];
             if !text_slice.is_empty() {
                if highlight_editor {
                    let highlighted_segment = highlighter.highlight(style, text_slice); // Removed mut
                    for mut section in highlighted_segment.sections {
                        section.format.font_id = font_id.clone();
                        job.sections.push(section);
                    }
                } else {
                    job.append(text_slice, 0.0, egui::TextFormat::simple(font_id.clone(), default_text_color));
                }
            }
            byte_idx_start = full_text_with_ime.len();
        }


        // Part 2: IME string
        char_idx_counter = 0; // Reset for IME string part relative to its start
        let ime_byte_start = byte_idx_start;
        for (char_byte_idx, _char) in full_text_with_ime[ime_byte_start..].char_indices() {
             let current_full_string_char_idx = text_before_char_end + char_idx_counter;
            if current_full_string_char_idx == ime_char_end {
                let ime_slice = &full_text_with_ime[ime_byte_start..(ime_byte_start + char_byte_idx)];
                 if !ime_slice.is_empty() {
                    job.append(
                        ime_slice,
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color: style.visuals.override_text_color.unwrap_or(egui::Color32::GREEN),
                            underline: egui::Stroke::new(1.0, style.visuals.override_text_color.unwrap_or(egui::Color32::GREEN)),
                            ..Default::default()
                        },
                    );
                }
                byte_idx_start = ime_byte_start + char_byte_idx;
                break;
            }
            if current_full_string_char_idx < ime_char_end {
                char_idx_counter +=1;
            } else {
                break;
            }
        }
        if text_before_char_end + char_idx_counter < ime_char_end { // If IME string goes to end of full_text_with_ime
            let ime_slice = &full_text_with_ime[ime_byte_start..];
            if !ime_slice.is_empty() {
                 job.append(
                    ime_slice,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: style.visuals.override_text_color.unwrap_or(egui::Color32::GREEN),
                        underline: egui::Stroke::new(1.0, style.visuals.override_text_color.unwrap_or(egui::Color32::GREEN)),
                        ..Default::default()
                    },
                );
            }
            byte_idx_start = full_text_with_ime.len();
        }


        // Part 3: Text after IME
        if byte_idx_start < full_text_with_ime.len() {
            let text_slice = &full_text_with_ime[byte_idx_start..];
            if !text_slice.is_empty() {
                if highlight_editor {
                    let highlighted_segment = highlighter.highlight(style, text_slice); // Removed mut
                    for mut section in highlighted_segment.sections {
                        section.format.font_id = font_id.clone();
                        job.sections.push(section);
                    }
                } else {
                    job.append(text_slice, 0.0, egui::TextFormat::simple(font_id.clone(), default_text_color));
                }
            }
        }

        job.wrap.max_width = wrap_width;
        job.break_on_newline = matches!(self.edit_type, EditType::MultiLine);
        // job.text is already set at the beginning
        job
    }
}

// Separate impl block for ImeManager methods
impl ImeManager {
    pub fn ui_for_editor(
        &mut self,
        text: &mut String,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        highlighter: &mut crate::easy_mark::easy_mark_highlighter::MemoizedEasymarkHighlighter,
        highlight_editor: bool,
    ) -> egui::text_edit::TextEditOutput {
        if self.count >= self.ime_texts.len() {
            self.add();
            self.ime_texts[self.count].text = text.to_string();
        }
        // Assuming EasyMarkEditor is always multiline
        let teo = self.ime_texts[self.count].get_text_edit_output(
            ui.available_width(), // Use available width for the editor
            text,
            EditType::MultiLine,
            ui,
            ctx,
            highlighter,
            highlight_editor,
        );
        self.ime_texts[self.count].id = teo.response.id.short_debug_format();
        self.count += 1;
        teo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui;

    // Helper to create a default ImeManager for tests
    fn create_test_ime_manager() -> ImeManager {
        ImeManager::default()
    }

    // Helper to create a default ImeText for tests
    fn create_test_ime_text() -> ImeText {
        ImeText::default()
    }

    #[test]
    fn test_ime_text_default() {
        let ime_text = create_test_ime_text();
        assert_eq!(ime_text.text, "");
        assert_eq!(ime_text.ime_string, "");
        assert_eq!(ime_text.cursor_index, 0);
        assert!(!ime_text.is_focus);
    }

    #[test]
    fn test_ime_manager_default() {
        let ime_manager = create_test_ime_manager();
        assert_eq!(ime_manager.count, 0);
        assert!(ime_manager.ime_texts.is_empty());
    }

    #[test]
    fn test_ime_text_listen_preedit() {
        let mut ime_text = create_test_ime_text();
        ime_text.is_focus = true; // Simulate focus

        let preedit_event = Ime::Preedit {
            value: "preedit".to_string(),
            cursor: Some((0, 1)), // Dummy cursor value
            window_id: bevy::window::WindowId::primary(), // Dummy window ID
        };
        ime_text.listen_ime_event(&preedit_event);

        assert_eq!(ime_text.ime_string, "preedit");
        assert_eq!(ime_text.ime_string.chars().count(), ime_text.ime_string_index);
    }

    #[test]
    fn test_ime_text_listen_commit_empty_text() {
        let mut ime_text = create_test_ime_text();
        ime_text.is_focus = true;

        let commit_event = Ime::Commit {
            value: "commit".to_string(),
            window_id: bevy::window::WindowId::primary(),
        };
        ime_text.listen_ime_event(&commit_event);

        assert_eq!(ime_text.text, "commit");
        assert!(ime_text.is_ime_input);
        assert_eq!(ime_text.ime_string, "");
    }

    #[test]
    fn test_ime_text_listen_commit_insert_middle() {
        let mut ime_text = create_test_ime_text();
        ime_text.is_focus = true;
        ime_text.text = "hello world".to_string();
        ime_text.cursor_index = 6; // Between "hello" and "world"

        let commit_event = Ime::Commit {
            value: "beautiful ".to_string(),
            window_id: bevy::window::WindowId::primary(),
        };
        ime_text.listen_ime_event(&commit_event);

        assert_eq!(ime_text.text, "hello beautiful world");
        assert!(ime_text.is_ime_input);
    }

    #[test]
    fn test_ime_text_listen_commit_append_end() {
        let mut ime_text = create_test_ime_text();
        ime_text.is_focus = true;
        ime_text.text = "hello".to_string();
        ime_text.cursor_index = ime_text.text.chars().count(); // At the end

        let commit_event = Ime::Commit {
            value: " world".to_string(),
            window_id: bevy::window::WindowId::primary(),
        };
        ime_text.listen_ime_event(&commit_event);

        assert_eq!(ime_text.text, "hello world");
        assert!(ime_text.is_ime_input);
    }


    #[test]
    fn test_ime_manager_add_text() {
        let mut ime_manager = create_test_ime_manager();
        ime_manager.add();
        assert_eq!(ime_manager.ime_texts.len(), 1);
    }

    #[test]
    fn test_ime_manager_listen_event_propagates() {
        let mut ime_manager = create_test_ime_manager();
        ime_manager.add();
        ime_manager.ime_texts[0].is_focus = true;

        let preedit_event = Ime::Preedit {
            value: "test".to_string(),
            cursor: Some((0, 1)),
            window_id: bevy::window::WindowId::primary(),
        };
        ime_manager.listen_ime_event(&preedit_event);

        assert_eq!(ime_manager.ime_texts[0].ime_string, "test");
    }

    // Note: Testing get_text_edit_output and get_layoutjob is more complex
    // as they depend on egui::Ui and egui::Context, and visual output.
    // These might be better suited for integration tests or visual regression tests.
    // For now, focusing on the logic of event handling and state changes.

    #[test]
    fn test_ime_text_enabled_disabled_events() {
        let mut ime_text = create_test_ime_text();
        ime_text.is_focus = true;

        let enabled_event = Ime::Enabled { window_id: bevy::window::WindowId::primary() };
        ime_text.listen_ime_event(&enabled_event);
        assert!(ime_text.is_ime);

        let disabled_event = Ime::Disabled { window_id: bevy::window::WindowId::primary() };
        ime_text.listen_ime_event(&disabled_event);
        assert!(!ime_text.is_ime);
    }
}

// Separate impl block for ImeManager methods
impl ImeManager {
    // ... (ui_for_editor remains here) ...

    fn add(&mut self) {
        let it = ImeText::new();
        self.ime_texts.push(it);
    }

    pub fn listen_ime_event(&mut self, event: &Ime) {
        for i in &mut self.ime_texts {
            i.listen_ime_event(event);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EditType { // Also make EditType public if it's used in public interfaces of ImeText, or if ImeText fields of this type are public
    SingleLine,
    MultiLine,
}

#[derive(Debug)]
pub struct ImeText { // Made public
    id: String, // Keeping fields private unless they need to be public for systems
    text: String,
    ime_string: String,
    ime_string_index: usize,
    cursor_index: usize,
    is_ime_input: bool,
    is_focus: bool,
    is_ime: bool,
    is_cursor_move: bool,
    edit_type: EditType,
    pub is_used: bool, // Made public
}

impl Default for ImeText {
    fn default() -> Self {
        ImeText {
            id: String::new(),
            text: String::new(),
            ime_string: String::new(),
            ime_string_index: 0,
            cursor_index: 0,
            is_ime_input: false,
            is_focus: false,
            is_ime: false,
            is_cursor_move: true,
            edit_type: EditType::SingleLine,
            is_used: false,
        }
    }
}

impl ImeText {
    fn new() -> ImeText {
        ImeText::default()
    }

    fn get_text_edit_output(
        &mut self,
        width: f32,
        text: &mut String,
        edit_type: EditType,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        highlighter: &mut crate::easy_mark::easy_mark_highlighter::MemoizedEasymarkHighlighter,
        highlight_editor: bool,
    ) -> egui::text_edit::TextEditOutput {
        self.edit_type = edit_type;
        self.is_used = true;
        let mut lyt = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let loj = self.get_layoutjob(string, wrap_width, ui.style(), highlighter, highlight_editor);
            ui.fonts(|f| f.layout_job(loj))
        };
        let mut tmp_text = match self.ime_string.len() {
            0 => self.text.to_string(),
            _ => {
                let mut front = String::new();
                let mut back = String::new();
                let mut cnt = 0;
                for c in self.text.chars() {
                    if cnt < self.cursor_index {
                        front.push_str(&c.to_string());
                    } else {
                        back.push_str(&c.to_string());
                    }
                    cnt += 1;
                }
                format!("{}{}{}", front, self.ime_string, back)
            }
        };

        let mut teo = match self.edit_type {
            EditType::SingleLine => egui::TextEdit::singleline(&mut tmp_text)
                .desired_width(width)
                .layouter(&mut lyt)
                .show(ui),
            EditType::MultiLine => egui::TextEdit::multiline(&mut tmp_text)
                .desired_width(width)
                .font(egui::TextStyle::Monospace) // Added Monospace font like in EasyMarkEditor
                .layouter(&mut lyt)
                .show(ui),
        };
        self.is_focus = teo.response.has_focus();
        if !self.is_ime {
            self.text = tmp_text.to_string();
        }
        if teo.cursor_range.is_some() {
            self.cursor_index = teo.cursor_range.unwrap().primary.ccursor.index;
        }
        if self.is_ime_input {
            //respose.changed()=true
            teo.response.mark_changed();
        }
        if self.is_ime_input {
            self.is_ime_input = false;
            if self.is_cursor_move {
                let mut res_cursor = teo.cursor_range.unwrap().primary.clone();
                for _ in 0..self.ime_string_index {
                    res_cursor = teo.galley.cursor_right_one_character(&res_cursor);
                }
                let cr = egui::text_selection::CursorRange {
                    primary: res_cursor,
                    secondary: res_cursor,
                };
                teo.state.cursor.set_range(Some(cr));
            }
        }
        if !self.is_cursor_move {
            self.is_cursor_move = true;
        }
        teo.state.clone().store(ctx, teo.response.id);
        *text = self.text.to_string();
        teo
    }

    fn listen_ime_event(&mut self, event: &Ime) {
        if !self.is_focus {
            return;
        }
        match event {
            Ime::Preedit { value, cursor, .. } if cursor.is_some() => {
                if self.is_focus {
                    self.ime_string = value.to_string();
                    self.ime_string_index = self.ime_string.chars().count();
                }
            }
            Ime::Commit { value, .. } => {
                if value.is_empty() {
                    self.is_cursor_move = false;
                }
                if self.is_focus {
                    let tmp = value.to_string();
                    if self.text.chars().count() == self.cursor_index {
                        self.text.push_str(&tmp);
                    } else {
                        let mut front = String::new();
                        let mut back = String::new();
                        let mut cnt = 0;
                        for c in self.text.chars() {
                            if cnt < self.cursor_index {
                                front.push_str(&c.to_string());
                            } else {
                                back.push_str(&c.to_string());
                            }
                            cnt += 1;
                        }
                        self.text = format!("{}{}{}", front, tmp, back);
                    }
                    self.is_ime_input = true;
                    self.ime_string = String::new();
                }
            }
            Ime::Enabled { .. } => {
                self.is_ime = true;
            }
            Ime::Disabled { .. } => {
                self.is_ime = false;
            }
            _ => (),
        }
    }
}
