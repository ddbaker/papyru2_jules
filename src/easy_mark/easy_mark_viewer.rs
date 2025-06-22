use super::easy_mark_parser as easy_mark;
use egui::{
    vec2, Align, Align2, Hyperlink, Layout, Response, RichText, Sense, Separator, Shape, TextStyle,
    Ui,
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
    let quote_marker = "> ";

    let mut current_line_items: Vec<easy_mark::Item> = Vec::new();
    let mut items = items_iter.peekable();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = ui.spacing().item_spacing.y * 0.5;

        while let Some(item) = items.next() {
            // println!("Viewer (easy_mark_it): Processing Item: {:?}", item); // [JULES] Commented out
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
                    );
                    current_line_items.clear();
                    indent_level = 0;
                    quote_level = 0;
                    list_marker_next = None;
                }
                easy_mark::Item::Separator => {
                    if !current_line_items.is_empty() {
                        render_line(
                            ui,
                            &current_line_items,
                            indent_level,
                            quote_level,
                            &list_marker_next,
                            one_indent_pixels,
                            quote_marker,
                        );
                        current_line_items.clear();
                    }
                    list_marker_next = None;

                    println!("[DEBUG] easy_mark_it: Matched Item::Separator. About to add egui::Separator.");
                    ui.add_space(ui.spacing().item_spacing.y / 2.0);
                    ui.add(egui::Separator::default());
                    ui.add_space(ui.spacing().item_spacing.y / 2.0);
                    println!("[DEBUG] easy_mark_it: egui::Separator added.");

                    indent_level = 0;
                    quote_level = 0;
                }
                _ => { // Text, Hyperlink, CodeBlock
                    current_line_items.push(item);
                    if items.peek().map_or(true, |next_item| {
                        matches!(next_item, easy_mark::Item::Newline | easy_mark::Item::Separator)
                    }) {
                        render_line(
                            ui,
                            &current_line_items,
                            indent_level,
                            quote_level,
                            &list_marker_next,
                            one_indent_pixels,
                            quote_marker,
                        );
                        current_line_items.clear();
                        if list_marker_next.is_some() && !current_line_items.iter().any(|i| matches!(i, easy_mark::Item::Text(_, _))) {
                            list_marker_next = None;
                        }
                    }
                }
            }
        }
        if !current_line_items.is_empty() {
            render_line(
                ui,
                &current_line_items,
                indent_level,
                quote_level,
                &list_marker_next,
                one_indent_pixels,
                quote_marker,
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
    quote_marker_str: &str,
) {
    if line_items.is_empty() && list_marker_text.is_none() && quote_level == 0 {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        if quote_level > 0 {
            ui.label(RichText::new(quote_marker_str.repeat(quote_level as usize)).weak());
            ui.add_space(2.0);
        }

        if indent_level > 0 {
            ui.add_space(indent_level as f32 * one_indent_pixels);
        }

        if let Some(marker_text_val) = list_marker_text {
            ui.label(RichText::new(marker_text_val.as_str()).strong());
            ui.add_space(2.0);
        }

        for item_content in line_items {
            item_ui_content(ui, *item_content);
        }
    });
}

pub fn item_ui_content(ui: &mut Ui, item: easy_mark::Item<'_>) {
    // println!("Viewer (item_ui_content): Processing Item: {:?}", item); // [JULES] Commented out
    match item {
        easy_mark::Item::Text(style, text) => {
            // println!("---Viewer: Item is Text--- style: {:?}, content: '{}', len: {}", style, text, text.len()); // [JULES] Commented out
            if text.trim().is_empty() && !text.contains('\n') {
                 ui.allocate_exact_size(vec2(0.0, 0.0), Sense::hover());
            } else {
                let label = rich_text_from_style(text, &style);
                ui.label(label);
            }
        }
        // Separator is handled by easy_mark_it now
        easy_mark::Item::Hyperlink(style, text, url) => {
            // println!("---Viewer: Item is Hyperlink--- Text: '{}', URL: '{}', Style: {:?}", text, url, style); // [JULES] Commented out
            let mut rich_text = rich_text_from_style(text, &style);
            if !style.underline {
                rich_text = rich_text.underline();
            }
            ui.add(egui::Hyperlink::from_label_and_url(rich_text, url));
        }
        easy_mark::Item::CodeBlock(_language, code) => {
            // println!("---Viewer: Item is CodeBlock--- Code: '{}'", code); // [JULES] Commented out
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
    );
    response
}
