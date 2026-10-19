//! Startup breadcrumbs survive GUI-subsystem stderr loss and native driver aborts.
use std::io::Write;

pub fn log(message: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for path in crate::candidate_log_paths("springsharp_boot.log") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            if writeln!(file, "[{timestamp} pid={}] {message}", std::process::id()).is_ok() {
                let _ = file.flush();
                break;
            }
        }
    }
}

pub fn renderer() -> Result<Option<eframe::Renderer>, String> {
    parse_renderer(std::env::args().skip(1))
}

fn parse_renderer(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<eframe::Renderer>, String> {
    let mut renderer = None;
    while let Some(arg) = args.next() {
        if arg == "--renderer" {
            renderer = Some(match args.next().as_deref() {
                Some("glow") => eframe::Renderer::Glow,
                Some("wgpu") => eframe::Renderer::Wgpu,
                _ => return Err("Use --renderer glow or --renderer wgpu".into()),
            });
        } else {
            return Err(format!(
                "Unknown argument: {arg}. Use --renderer glow or --renderer wgpu"
            ));
        }
    }
    Ok(renderer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_override_bypasses_primary_backend() {
        let parse = |args: &[&str]| parse_renderer(args.iter().map(|s| s.to_string()));
        assert!(parse(&[]).unwrap().is_none());
        assert!(matches!(
            parse(&["--renderer", "glow"]).unwrap(),
            Some(eframe::Renderer::Glow)
        ));
        assert!(matches!(
            parse(&["--renderer", "wgpu"]).unwrap(),
            Some(eframe::Renderer::Wgpu)
        ));
        assert!(parse(&["--renderer"]).is_err());
        assert!(parse(&["--renderer", "invalid"]).is_err());
        assert!(parse(&["--invalid"]).is_err());
    }
}
