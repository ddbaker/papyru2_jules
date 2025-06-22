use super::easy_mark_parser as easy_mark;
use egui::{
    vec2, Align, Align2, Hyperlink, Layout, Response, RichText, Sense, Separator, Shape, TextStyle,
    Ui,
};

/// Parse and display a VERY simple and small subset of Markdown.
pub fn easy_mark(ui: &mut Ui, easy_mark_str: &str) { // Renamed easy_mark to easy_mark_str for clarity
    easy_mark_it(ui, easy_mark::Parser::new(easy_mark_str));
}

pub fn easy_mark_it<'em>(ui: &mut Ui, items_iter: impl Iterator<Item = easy_mark::Item<'em>>) {
    let mut indent_level: u8 = 0;
    let mut quote_level: u8 = 0;
    let mut list_marker_next: Option<String> = None;

    let base_row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent_pixels = base_row_height * 1.5;
    // For quote drawing, direct drawing is complex with wrapping.
    // Using a simple text marker for quotes for now.
    let quote_marker = "> "; // Simple text marker for quotes

    let mut current_line_items: Vec<easy_mark::Item> = Vec::new();
    let mut items = items_iter.peekable();

    // Use a vertical layout to stack lines.
    // Each call to render_line will handle one "visual" line.
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = ui.spacing().item_spacing.y * 0.5; // Reduce vertical spacing between lines a bit

        while let Some(item) = items.next() {
            println!("Viewer (easy_mark_it): Processing Item: {:?}", item);
            match item {
                easy_mark::Item::Indentation(level) => indent_level = level,
                easy_mark::Item::QuoteIndent => quote_level += 1,
                easy_mark::Item::BulletPoint => list_marker_next = Some("•".to_string()),
                easy_mark::Item::NumberedPoint(s) => list_marker_next = Some(format!("{}.", s)),
                easy_mark::Item::Newline => {
                    render_line(
                        ui,
                        &current_line_items,
                        indent_level,
                        quote_level,
                        &list_marker_next,
                        one_indent_pixels,
                        quote_marker,
                        // base_row_height, // Not directly needed by render_line if it uses item_ui_content
                    );
                    current_line_items.clear();
                    // Newline implies starting a new block, reset states
                    indent_level = 0;
                    quote_level = 0;
                    list_marker_next = None;
                    // The vertical layout of easy_mark_it implicitly handles new lines.
                    // If an explicit space is needed for multiple newlines, the parser should yield multiple Newline items.
                    // A single Newline item just means "end current line, reset for next".
                }
                _ => { // Text, Separator, Hyperlink, CodeBlock
                    current_line_items.push(item);
                    // If next item is a newline, or it's the end, then render the accumulated line.
                    if items.peek().map_or(true, |next_item| matches!(next_item, easy_mark::Item::Newline)) {
                        render_line(
                            ui,
                            &current_line_items,
                            indent_level,
                            quote_level,
                            &list_marker_next,
                            one_indent_pixels,
                            quote_marker,
                            // base_row_height,
                        );
                        current_line_items.clear();
                        // If the line was ended by a an item that implies a newline (like separator, code block)
                        // and the *next* item is not a Newline, we might need to reset state here too.
                        // However, Newline item is the primary state reset trigger.
                        // List marker is consumed by render_line.
                        if list_marker_next.is_some() && !current_line_items.iter().any(|i| matches!(i, easy_mark::Item::Text(_, _))) {
                            // If there was a list marker but no text followed on this line before newline
                            list_marker_next = None; // Consume it as it was "rendered" with no text
                        }

                    }
                }
            }
        }
        // Render any remaining items on the last line
        if !current_line_items.is_empty() {
            render_line(
                ui,
                &current_line_items,
                indent_level,
                quote_level,
                &list_marker_next,
                one_indent_pixels,
                quote_marker,
                // base_row_height,
            );
        }
    });
}

// Helper function to render a single line of items with current context
fn render_line(
    ui: &mut Ui,
    line_items: &[easy_mark::Item],
    indent_level: u8,
    quote_level: u8,
    list_marker_text: &Option<String>, // Changed name for clarity
    one_indent_pixels: f32,
    quote_marker_str: &str, // Using string for quote marker
    // base_row_height: f32, // Not directly used if item_ui_content handles its own spacing
) {
    if line_items.is_empty() && list_marker_text.is_none() && quote_level == 0 {
        // Avoid rendering completely empty horizontal regions if there's nothing to show.
        // A line with just a quote or list marker should still render.
        return;
    }

    ui.horizontal_wrapped(|ui| {
        // Default spacing for items on the same line (e.g. styled text segments)
        // ui.spacing_mut().item_spacing.x = 0.0; // Handled by rich_text_from_style or individual items

        if quote_level > 0 {
            // Simple text-based quote marker
            ui.label(RichText::new(quote_marker_str.repeat(quote_level as usize)).weak());
            ui.add_space(2.0); // Small space after quote marker
        }

        if indent_level > 0 {
            ui.add_space(indent_level as f32 * one_indent_pixels);
        }

        if let Some(marker_text_val) = list_marker_text {
            ui.label(RichText::new(marker_text_val.as_str()).strong());
            ui.add_space(2.0); // Space after list marker
        }

        for item_content in line_items {
            // Since Item is Copy (due to &str), we can pass it directly.
            item_ui_content(ui, *item_content);
        }
    });
}

// Renamed item_ui to item_ui_content, and it only handles "content" items.
pub fn item_ui_content(ui: &mut Ui, item: easy_mark::Item<'_>) {
    println!("Viewer (item_ui_content): Processing Item: {:?}", item);
    match item {
        easy_mark::Item::Text(style, text) => {
            println!("---Viewer: Item is Text--- style: {:?}, content: '{}', len: {}", style, text, text.len());
            if text.trim().is_empty() && !text.contains('\n') { // Don't add massive space for empty strings
                 ui.allocate_exact_size(vec2(0.0, 0.0), Sense::hover()); // Minimal space for empty text
            } else {
                let label = rich_text_from_style(text, &style);
                ui.label(label); // ui.label will wrap text by default.
            }
        }
        easy_mark::Item::Separator => {
            println!("---Viewer: Item is Separator--- Adding a separator widget.");
            // Separator should take full width and imply a line break of its own.
            // The horizontal_wrapped layout might constrain it.
            // Forcing it to be on its own "line visually" is tricky here.
            // The current render_line calls horizontal_wrapped. A separator should break this.
            // This suggests Separator should be handled by easy_mark_it like Newline.
            // For now, let it render, but it might look weird.
            // A quick fix: make separator consume available width and add newline after.
            let available_width = ui.available_width(); // This is width *within* current horizontal_wrapped.
            ui.add_space(ui.spacing().item_spacing.y / 2.0);
            ui.add(Separator::default().grow(available_width));
            // ui.add(egui::Label::new("\n").wrap(false)); // No, let Newline item handle visual newlines.
        }
        easy_mark::Item::Hyperlink(style, text, url) => {
            println!("---Viewer: Item is Hyperlink--- Text: '{}', URL: '{}', Style: {:?}", text, url, style);
            let mut rich_text = rich_text_from_style(text, &style);
            if !style.underline {
                rich_text = rich_text.underline();
            }
            ui.add(egui::Hyperlink::from_label_and_url(rich_text, url));
        }
        easy_mark::Item::CodeBlock(_language, code) => {
            println!("---Viewer: Item is CodeBlock--- Code: '{}'", code);
            // CodeBlock should also be a block element.
            // Similar to Separator, it should ideally break the horizontal_wrapped flow.
            // For now, render it within.
            egui::Frame::group(ui.style())
                .fill(ui.visuals().code_bg_color)
                .show(ui, |ui| {
                    ui.label(RichText::new(code).code());
                });
            // ui.add(egui::Label::new("\n").wrap(false)); // No, let Newline item handle newlines.
        }
        // These should ideally not be passed to item_ui_content if easy_mark_it is correct
        easy_mark::Item::Newline |
        easy_mark::Item::Indentation(_) |
        easy_mark::Item::QuoteIndent |
        easy_mark::Item::BulletPoint |
        easy_mark::Item::NumberedPoint(_) => {
             eprintln!("ERROR: Layout item {:?} unexpectedly passed to item_ui_content.", item);
        }
    }
}

fn rich_text_from_style(text: &str, style: &easy_mark::Style) -> RichText {
    let easy_mark::Style {
        heading,
        quoted,
        code,
        strong,
        underline,
        strikethrough,
        italics,
        small,
        raised,
    } = *style;

    let small = small || raised; // Raised text is also smaller

    let mut rich_text = RichText::new(text);
    if heading && !small {
        rich_text = rich_text.heading().strong();
    }
    if small && !heading {
        rich_text = rich_text.small();
    }
    if code {
        rich_text = rich_text.code();
    }
    if strong {
        rich_text = rich_text.strong();
    } else if quoted {
        rich_text = rich_text.weak();
    }
    if underline {
        rich_text = rich_text.underline();
    }
    if strikethrough {
        rich_text = rich_text.strikethrough();
    }
    if italics {
        rich_text = rich_text.italics();
    }
    if raised {
        rich_text = rich_text.raised();
    }
    rich_text
}

fn bullet_point(ui: &mut Ui, width: f32) -> Response {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let (rect, response) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
    ui.painter().circle_filled(
        rect.center(),
        rect.height() / 8.0,
        ui.visuals().strong_text_color(),
    );
    response
}

fn numbered_point(ui: &mut Ui, width: f32, number: &str) -> Response {
    let font_id = TextStyle::Body.resolve(ui.style());
    let row_height = ui.fonts(|f| f.row_height(&font_id));
    let (rect, response) = ui.allocate_exact_size(vec2(width, row_height), Sense::hover());
    let text = format!("{number}.");
    let text_color = ui.visuals().strong_text_color();
    ui.painter().text(
        rect.right_center(),
        Align2::RIGHT_CENTER,
        text,
        font_id,
        text_color,
    ); // Semicolon added here
    response
}
