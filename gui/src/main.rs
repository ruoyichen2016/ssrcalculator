//! Entry point for the optional `springsharp-gui` binary.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod fonts;
mod i18n;
mod theme;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn candidate_log_paths(filename: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join(filename));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(filename));
    }
    paths.push(std::env::temp_dir().join(filename));
    paths
}

fn write_log(filename: &str, content: &str) -> Option<PathBuf> {
    for path in candidate_log_paths(filename) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
        {
            if file.write_all(content.as_bytes()).is_ok() {
                return Some(path);
            }
        }
    }
    None
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any> (unknown panic payload)".to_string()
        };

        let backtrace = std::backtrace::Backtrace::capture();
        let log_content = format!(
            "=== SpringSharp Fatal Crash ===\nLocation: {}\nMessage: {}\n\nBacktrace:\n{}\n",
            location, payload, backtrace
        );

        let saved_to = write_log("springsharp_crash.log", &log_content);
        let dialog_msg = match saved_to {
            Some(path) => format!(
                "SpringSharp encountered an unexpected fatal error:\n\n{}\n\nCrash log written to:\n{}",
                payload,
                path.display()
            ),
            None => format!(
                "SpringSharp encountered an unexpected fatal error:\n\n{}\n\n(Could not write crash log to disk)",
                payload
            ),
        };

        rfd::MessageDialog::new()
            .set_title("SpringSharp Fatal Error")
            .set_description(&dialog_msg)
            .set_level(rfd::MessageLevel::Error)
            .show();

        default_hook(info);
    }));
}

fn main() {
    write_log(
        "springsharp_boot.log",
        "SpringSharp process initiated. Installing panic hook...\n",
    );
    install_panic_hook();

    match app::run() {
        Ok(()) => {
            let _ = write_log(
                "springsharp_boot.log",
                "SpringSharp process exited cleanly.\n",
            );
        }
        Err(error) => {
            let log_content = format!("=== SpringSharp Startup Error ===\nError: {}\n", error);
            let saved_to = write_log("springsharp_startup_error.log", &log_content);
            let dialog_msg = match saved_to {
                Some(path) => format!(
                    "SpringSharp failed to start:\n\n{}\n\nDetails saved to:\n{}",
                    error,
                    path.display()
                ),
                None => format!("SpringSharp failed to start:\n\n{}", error),
            };

            rfd::MessageDialog::new()
                .set_title("SpringSharp Startup Error")
                .set_description(&dialog_msg)
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
    }
}

