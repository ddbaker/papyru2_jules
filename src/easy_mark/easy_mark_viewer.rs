use super::easy_mark_parser as easy_mark;
use egui::{
    vec2, Align, Align2, Hyperlink, Layout, Response, RichText, Sense, Separator, Shape, TextStyle,
    Ui,
};

/// Parse and display a VERY simple and small subset of Markdown.
pub fn easy_mark(ui: &mut Ui, easy_mark: &str) {
    easy_mark_it(ui, easy_mark::Parser::new(easy_mark));
}

pub fn easy_mark_it<'em>(ui: &mut Ui, items: impl Iterator<Item = easy_mark::Item<'em>>) {
    let initial_size = vec2(
        ui.available_width(),
        ui.spacing().interact_size.y, // Assume there will be
    );

    let layout = Layout::left_to_right(Align::BOTTOM).with_main_wrap(true);

    ui.allocate_ui_with_layout(initial_size, layout, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let row_height = ui.text_style_height(&TextStyle::Body);
        ui.set_row_height(row_height);

        for item in items {
            println!("Viewer: Received item: {:?}", item); // Re-enabled
            item_ui(ui, item);
        }
    });
}

pub fn item_ui(ui: &mut Ui, item: easy_mark::Item<'_>) {
    println!("Viewer: item_ui called for: {:?}", item);
    let row_height = ui.text_style_height(&TextStyle::Body);
    println!("Viewer: row_height calculated as: {}", row_height);
    // let one_indent = row_height / 2.0; // Keep this commented for now, not used by simplified items

    match item {
        easy_mark::Item::Newline => {
            ui.allocate_exact_size(egui::vec2(0.0, row_height), egui::Sense::hover()); // Minimal newline
            println!("Viewer: Processed Newline (using allocate_exact_size)");
        }
        easy_mark::Item::Text(_style, text) => { // Ignoring style for now
            ui.label(text);
            println!("Viewer: Processed Text: {}", text);
        }
        easy_mark::Item::Separator => {
            ui.add(Separator::default().horizontal());
            println!("Viewer: Processed Separator");
        }
        _ => {
            // ui.label(format!("Skipped: {:?}", item)); // Optional: for visual feedback
            println!("Viewer: Skipped item: {:?}", item);
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
