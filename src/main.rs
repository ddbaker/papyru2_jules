use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiContextPass};

// This module should exist at src/easy_mark/mod.rs
// and contain the easy_mark_editor, etc.
mod easy_mark;

// Wrapper struct for the EasyMarkEditor to be used as a Bezy resource.
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true, })
        .init_resource::<EasyMarkEditorState>() // Initialize the editor state as a resource
        .add_systems(EguiContextPass, ui_system)
        .run();
}

// Bevy system to render the Egui UI
fn ui_system(mut contexts: EguiContexts, mut editor_state: ResMut<EasyMarkEditorState>) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        // Access the editor through the editor_state resource
        // The EasyMarkEditor::ui method will be called here.
        // Note: The original EasyMarkEditor::ui takes &mut self and &mut egui::Ui.
        // We get &mut editor_state.editor and the ui from the CentralPanel.
        editor_state.editor.ui(ui);
    });
}
