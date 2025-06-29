use bevy::prelude::*;
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};

// This module should exist at src/easy_mark/mod.rs
// and contain the easy_mark_editor, etc.
mod easy_mark;
mod ime; // Added ime module

// Wrapper struct for the EasyMarkEditor to be used as a Bevy resource.
#[derive(Resource)]
struct EasyMarkEditorState {
    editor: easy_mark::easy_mark_editor::EasyMarkEditor,
}

// Implement Default for the resource state.
// The EasyMarkEditor itself has a Default implementation.
impl Default for EasyMarkEditorState {
    fn default() -> Self {
        Self {
            editor: easy_mark::easy_mark_editor::EasyMarkEditor::default(),
        }
    }
}

fn main() {
    let default_window_width = 1156.0;
    let default_window_height = 612.0;
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "EasyMark Editor".into(),
                resolution: (default_window_width, default_window_height).into(),
                ime_enabled: true, // Enable IME events for the window
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(Startup, setup_egui_font) // Added system to setup egui font
        .init_resource::<EasyMarkEditorState>() // Initialize the editor state as a resource
        .init_resource::<ime::ImeManager>()      // Initialize ImeManager as a resource
        .add_systems(PreUpdate, reset_unused_ime_system) // Added PreUpdate system
        .add_systems(Update, listen_ime_events_system)  // Added Update system for IME events
        .add_systems(PostUpdate, clear_unused_ime_system) // Added PostUpdate system
        .add_systems(EguiContextPass, ui_system)
        .run();
}

// System to setup egui font with NotoSerifCJKjp-Medium.otf
fn setup_egui_font(mut contexts: EguiContexts) {
    let mut fonts = egui::FontDefinitions::default();

    // Install NotoSerifCJKjp-Medium font
    // Make sure the path to the font file is correct
    fonts.font_data.insert(
        "NotoSerifCJKjp-Medium".to_owned(),
        // egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSerifCJKjp-Medium.otf")), // Old
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSerifCJKjp-Medium.otf"))), // Wrapped with Arc::new
    );

    // Put NotoSerifCJKjp-Medium first (highest priority) for proportional text
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "NotoSerifCJKjp-Medium".to_owned());

    // Put NotoSerifCJKjp-Medium first (highest priority) for monospace text
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "NotoSerifCJKjp-Medium".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
}

// Bevy system to render the Egui UI
fn ui_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EasyMarkEditorState>,
    mut ime_manager: ResMut<ime::ImeManager>, // Added ImeManager resource
) {
    if let Some(mut ctx) = contexts.try_ctx_mut() {
        ctx.set_theme(egui::Theme::Light);
        let inner_ctx = ctx.clone(); // Clone the context to pass immutably
        egui::CentralPanel::default().show(&mut ctx, |ui| {
            // Pass ImeManager and the cloned context to the editor's ui method
            // highlighter and highlight_editor are accessed within EasyMarkEditor::editor_ui
            editor_state.editor.ui(ui, &mut ime_manager, &inner_ctx);
        });
    }
}

// System to reset IME state at the beginning of the frame
fn reset_unused_ime_system(mut ime_manager: ResMut<ime::ImeManager>) {
    for ime_text in &mut ime_manager.ime_texts { // Made public in ImeManager for this system
        ime_text.is_used = false;
    }
    ime_manager.count = 0; // Made public in ImeManager
}

// System to listen to IME events from Bevy
fn listen_ime_events_system(
    mut events: EventReader<Ime>,
    mut ime_manager: ResMut<ime::ImeManager>,
    mut windows: Query<&mut Window>,
) {
    for event in events.read() {
        ime_manager.listen_ime_event(event);
    }
    if let Ok(mut window) = windows.single_mut() { // Changed to single_mut()
        if let Some(cursor_pos) = window.cursor_position() {
            window.ime_position = cursor_pos;
        }
    }
}

// System to clear unused IME text instances at the end of the frame
fn clear_unused_ime_system(mut ime_manager: ResMut<ime::ImeManager>) {
    ime_manager.ime_texts.retain(|i| i.is_used); // Made public in ImeManager
}
