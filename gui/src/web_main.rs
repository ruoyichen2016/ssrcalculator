//! Visible lifetime owner for the local zero-JavaScript service.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod fonts;
use eframe::egui;
use springsharp::{presentation::fonts::TextLanguage, LocalServer};

struct WebLauncher {
    server: Option<LocalServer>,
    open_on_start: bool,
}
impl eframe::App for WebLauncher {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("SpringSharp Web");
            if let Some(server) = &self.server {
                ui.label("Local service is running. Closing this window stops it.");
                ui.hyperlink_to(&server.url, &server.url);
                if self.open_on_start || ui.button("Open workbench in browser").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(&server.url));
                    self.open_on_start = false;
                }
                ui.separator();
                ui.label("HTML");
                if ui.button("Stop service and close").clicked() {
                    self.server.take();
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
    }
}
fn main() {
    if let Err(error) = run() {
        rfd::MessageDialog::new()
            .set_title("SpringSharp Web startup error")
            .set_description(error)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}
fn run() -> Result<(), String> {
    let server = LocalServer::start().map_err(|e| e.to_string())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SpringSharp Web",
        options,
        Box::new(move |cc| {
            fonts::install(&cc.egui_ctx, TextLanguage::Germanic);
            Ok(Box::new(WebLauncher {
                server: Some(server),
                open_on_start: true,
            }))
        }),
    )
    .map_err(|e| e.to_string())
}
