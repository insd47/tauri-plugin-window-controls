const COMMANDS: &[&str] = &["get_glyph_path", "window_command", "window_state", "enable_snap"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
