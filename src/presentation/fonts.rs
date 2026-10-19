//! Shared font manifest and language policy. No dependency from the numerical core.
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextLanguage {
    #[default]
    Germanic,
    Italic,
    Han,
}
impl TextLanguage {
    pub const ALL: [Self; 3] = [Self::Germanic, Self::Italic, Self::Han];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Germanic => "gem",
            Self::Italic => "und-Latn",
            Self::Han => "und-Hani",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Germanic => "Linguae Germanicae",
            Self::Italic => "Linguae Italicae",
            Self::Han => "汉字",
        }
    }
    pub fn parse(tag: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|l| l.tag() == tag)
            .unwrap_or_default()
    }
    pub fn sans(self) -> [&'static str; 2] {
        ["Noto Sans", "Noto Sans CJK SC"]
    }
}
pub const FONTS: [(&str, &str); 5] = [
    ("Noto Sans", "NotoSans-Regular.ttf"),
    ("Noto Serif", "NotoSerif-Regular.ttf"),
    ("Noto Sans Mono", "NotoSansMono-Regular.ttf"),
    ("Noto Sans CJK SC", "NotoSansCJKsc-Regular.otf"),
    ("Noto Serif CJK JP", "NotoSerifCJKjp-Regular.otf"),
];
pub fn assets_dir() -> PathBuf {
    let deployed = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("assets")));
    if let Some(path) = deployed {
        if path.is_dir() {
            return path;
        }
    }
    if cfg!(debug_assertions) {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    }
    // A missing installation stays a visible missing resource, never a cwd lookup.
    std::env::current_exe()
        .unwrap_or_default()
        .with_file_name("assets")
}
pub fn font_path(file: &str) -> Option<PathBuf> {
    FONTS
        .iter()
        .any(|(_, name)| *name == file)
        .then(|| assets_dir().join("fonts").join(file))
}
pub fn css() -> String {
    let mut css = String::new();
    for (name, file) in FONTS {
        css.push_str(&format!("@font-face{{font-family:'{name}';src:url('/assets/fonts/{file}');font-weight:400;font-display:swap;}}\n"));
    }
    css.push_str("html{--sans:'Noto Sans','Noto Sans CJK SC',sans-serif;--mono:'Noto Sans Mono','Noto Sans CJK SC',monospace;}\n.lang-text{font-family:var(--sans);} .serif-title{font-family:'Noto Serif','Noto Serif CJK JP','Noto Sans','Noto Sans CJK SC',serif;}\n");
    css
}
