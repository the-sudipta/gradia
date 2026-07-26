// Prevents an extra console window from appearing beside Gradia on Windows
// release builds. Debug builds keep the console for developer diagnostics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gradia_lib::run();
}
