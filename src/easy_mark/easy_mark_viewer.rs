use super::easy_mark_parser as easy_mark;
use bevy_egui::{egui};
use egui::{
    vec2, Align, Layout, RichText, Sense, TextStyle, Ui, // Removed Hyperlink, Separator, Shape
};

/// Parse and display a VERY simple and small subset of Markdown.
pub fn easy_mark(ui: &mut Ui, easy_mark_str: &str) {
    easy_mark_it(ui, easy_mark::Parser::new(easy_mark_str));
}

pub fn easy_mark_it<'em>(ui: &mut Ui, items_iter: impl Iterator<Item = easy_mark::Item<'em>>) {
    let mut indent_level: usize = 0;
    let mut quote_level: u8 = 0;
    let mut list_marker_next: Option<String> = None;

    let base_row_height = ui.text_style_height(&TextStyle::Body);
    let one_indent_pixels = base_row_height * 1.5;

    let mut current_line_items: Vec<easy_mark::Item> = Vec::new();
    // let mut items = items_iter.peekable(); // Not peeking anymore in this revised structure

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = ui.spacing().item_spacing.y * 0.5;

        for item in items_iter { // Iterate directly, no peek needed for this simplified logic
            // println!("Viewer (easy_mark_it): Processing Item: {:?}", item);
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
                        base_row_height,
                    );
                    current_line_items.clear();
                    indent_level = 0;
                    quote_level = 0;
                    list_marker_next = None;
                }
                easy_mark::Item::Separator => {
                    // Render any pending line items before the separator
                    render_line(
                        ui,
                        &current_line_items,
                        indent_level,
                        quote_level,
                        &list_marker_next,
                        one_indent_pixels,
                        base_row_height,
                    );
                    current_line_items.clear();
                    list_marker_next = None;

                    // println!("[DEBUG] easy_mark_it: Matched Item::Separator. About to add egui::Separator."); // [JULES] Commented out
                    ui.add_space(ui.spacing().item_spacing.y / 2.0);
                    ui.add(egui::Separator::default());
                    ui.add_space(ui.spacing().item_spacing.y / 2.0);
                    // println!("[DEBUG] easy_mark_it: egui::Separator added."); // [JULES] Commented out

                    indent_level = 0;
                    quote_level = 0;
                    // list_marker_next is already None
                }
                _ => { // Text, Hyperlink, CodeBlock - ONLY ACCUMULATE
                    current_line_items.push(item);
                }
            }
        }
        // After loop, render any remaining current_line_items
        // This condition ensures we render if there are items, or if it's an empty line that was quoted or had a list marker.
        if !current_line_items.is_empty() || list_marker_next.is_some() || quote_level > 0 {
            render_line(
                ui,
                &current_line_items,
                indent_level,
                quote_level,
                &list_marker_next,
                one_indent_pixels,
                base_row_height,
            );
        }
    });
}

fn render_line(
    ui: &mut Ui,
    line_items: &[easy_mark::Item],
    indent_level: usize,
    quote_level: u8,
    list_marker_text: &Option<String>,
    one_indent_pixels: f32,
    base_row_height: f32,
) {
    if line_items.is_empty() && list_marker_text.is_none() && quote_level == 0 {
        // If there are no items, no list marker, and not in a quote, then it's a truly empty line.
        // Add a small space to represent the blank line visually if desired,
        // or just return to draw nothing. The vertical layout in easy_mark_it might handle spacing.
        // For now, let's ensure it doesn't draw a horizontal ui if completely empty.
        // If we want blank lines to take up space, easy_mark_it's Newline arm could add ui.add_space().
        // The current Newline arm implicitly does this by ending a line and letting vertical layout space.
        return;
    }

    // If there's any actual content OR if it's a quoted line (even if otherwise empty) OR a line with just a list marker
    // The check above already handles the "truly empty and unstyled" case.
    // So, if we are here, we draw.

    ui.horizontal(|ui_line| {
        ui_line.spacing_mut().item_spacing.x = 0.0;

        if quote_level > 0 {
            let bar_color = ui_line.visuals().widgets.noninteractive.fg_stroke.color.gamma_multiply(0.4);
            let quote_bar_width = 2.0;
            let quote_bar_total_spacing_per_level = quote_bar_width + 4.0;

            let total_quote_indent_width = quote_level as f32 * quote_bar_total_spacing_per_level;

            let (quote_bar_area_rect, _) = ui_line.allocate_exact_size(
                egui::vec2(total_quote_indent_width, base_row_height),
                Sense::hover()
            );

            for i in 0..quote_level {
                let bar_x_start = quote_bar_area_rect.min.x + i as f32 * quote_bar_total_spacing_per_level;
                let rect = egui::Rect::from_min_max(
                    egui::pos2(bar_x_start, quote_bar_area_rect.min.y),
                    egui::pos2(bar_x_start + quote_bar_width, quote_bar_area_rect.max.y)
                );
                ui_line.painter().rect_filled(rect, 0.0, bar_color);
            }
        }

        let content_available_width = ui_line.available_width();
        ui_line.allocate_ui_with_layout(
            egui::vec2(content_available_width, 0.0),
            Layout::left_to_right(Align::TOP).with_main_wrap(true),
            |ui_content| {
                ui_content.spacing_mut().item_spacing.x = 2.0;

                if indent_level > 0 {
                    ui_content.add_space(indent_level as f32 * one_indent_pixels);
                }

                if let Some(marker_text_val) = list_marker_text {
                    ui_content.label(RichText::new(marker_text_val.as_str()).strong());
                }

                for item_content in line_items {
                    item_ui_content(ui_content, *item_content);
                }
            }
        );
    });
}

pub fn item_ui_content(ui: &mut Ui, item: easy_mark::Item<'_>) {
    // println!("Viewer (item_ui_content): Processing Item: {:?}", item);
    match item {
        easy_mark::Item::Text(style, text) => {
            // println!("---Viewer: Item is Text--- style: {:?}, content: '{}', len: {}", style, text, text.len());
            if text.trim().is_empty() && !text.contains('\n') {
                 ui.allocate_exact_size(vec2(0.0, 0.0), Sense::hover());
            } else {
                let label = rich_text_from_style(text, &style);
                ui.label(label);
            }
        }
        easy_mark::Item::Hyperlink(style, text, url) => {
            // println!("---Viewer: Item is Hyperlink--- Text: '{}', URL: '{}', Style: {:?}", text, url, style);
            let mut rich_text = rich_text_from_style(text, &style);
            if !style.underline {
                rich_text = rich_text.underline();
            }
            ui.add(egui::Hyperlink::from_label_and_url(rich_text, url));
        }
        easy_mark::Item::CodeBlock(_language, code) => {
            // println!("---Viewer: Item is CodeBlock--- Code: '{}'", code);
            egui::Frame::group(ui.style())
                .fill(ui.visuals().code_bg_color)
                .show(ui, |ui| {
                    ui.label(RichText::new(code).code());
                });
        }
        easy_mark::Item::Newline |
        easy_mark::Item::Indentation(_) |
        easy_mark::Item::QuoteIndent |
        easy_mark::Item::BulletPoint |
        easy_mark::Item::NumberedPoint(_) |
        easy_mark::Item::Separator => {
             eprintln!("ERROR: Layout/Block item {:?} unexpectedly passed to item_ui_content.", item);
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

    let small = small || raised;

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

// [JULES] fn bullet_point and fn numbered_point removed as they are unused.
