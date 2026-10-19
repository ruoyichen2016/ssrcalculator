//! Entry point for the optional `springsharp-gui` binary.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod fonts;
mod i18n;
mod theme;

fn main() {
    if let Err(error) = app::run() {
        rfd::MessageDialog::new()
            .set_title("SpringSharp startup error")
            .set_description(error.to_string())
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}
