#[cfg(feature = "gui")]
fn main() -> eframe::Result<()> {
    springsharp::gui::run()
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!(
        "This binary requires the `gui` feature: cargo run --features gui --bin springsharp-gui"
    );
}
