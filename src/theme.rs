use zenthra::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

pub const ACCENT_COLOR_OPTIONS: &[(&str, &str)] = &[
    ("gray", "Gray"),
    ("yellow", "Yellow"),
    ("blue", "Blue"),
    ("green", "Green"),
    ("red", "Red"),
    ("orange", "Orange"),
    ("purple", "Purple"),
    ("pink", "Pink"),
    ("turquoise", "Turquoise"),
    ("violet", "Violet"),
    ("lime", "Lime"),
    ("white", "White"),
];

pub const HIGHLIGHT_COLOR_OPTIONS: &[(&str, &str)] = &[
    ("gray", "Gray"),
    ("yellow", "Yellow"),
    ("blue", "Blue"),
    ("green", "Green"),
    ("red", "Red"),
    ("orange", "Orange"),
    ("purple", "Purple"),
    ("pink", "Pink"),
    ("turquoise", "Turquoise"),
    ("violet", "Violet"),
    ("lime", "Lime"),
    ("white", "White"),
];

pub fn named_color(name: &str, mode: ThemeMode) -> Color {
    match name {
        "yellow" => Color::rgb(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0),
        "blue" => Color::rgb(52.0 / 255.0, 152.0 / 255.0, 219.0 / 255.0),
        "green" => Color::rgb(46.0 / 255.0, 204.0 / 255.0, 113.0 / 255.0),
        "red" => Color::rgb(252.0 / 255.0, 60.0 / 255.0, 56.0 / 255.0),
        "orange" => Color::rgb(253.0 / 255.0, 151.0 / 255.0, 31.0 / 255.0),
        "purple" => Color::rgb(155.0 / 255.0, 89.0 / 255.0, 182.0 / 255.0),
        "pink" => Color::rgb(236.0 / 255.0, 100.0 / 255.0, 165.0 / 255.0),
        "turquoise" => Color::rgb(26.0 / 255.0, 188.0 / 255.0, 156.0 / 255.0),
        "violet" => Color::rgb(142.0 / 255.0, 68.0 / 255.0, 173.0 / 255.0),
        "lime" => Color::rgb(132.0 / 255.0, 204.0 / 255.0, 22.0 / 255.0),
        "gray" => Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0),
        "white" => match mode {
            ThemeMode::Dark => Color::rgb(240.0 / 255.0, 240.0 / 255.0, 240.0 / 255.0),
            ThemeMode::Light => Color::rgb(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0),
        },
        _ => Color::rgb(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0),
    }
}

fn resolve_highlight(name: &str, mode: ThemeMode) -> Color {
    if name == "gray" {
        match mode {
            ThemeMode::Dark => Color::rgb(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0).with_alpha(0.08),
            ThemeMode::Light => Color::rgb(0.0, 0.0, 0.0).with_alpha(0.08),
        }
    } else {
        let alpha = match mode {
            ThemeMode::Dark => 0.22,
            ThemeMode::Light => 0.18,
        };
        named_color(name, mode).with_alpha(alpha)
    }
}

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_base: Color,
    pub bg_panel: Color,
    pub bg_sidebar: Color,
    pub bg_active: Color,
    pub border: Color,
    pub highlight: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_dim: Color,
}

impl ThemeColors {
    pub fn resolve(mode: ThemeMode, accent_name: &str, highlight_name: &str) -> Self {
        let accent = named_color(accent_name, mode);
        let highlight = resolve_highlight(highlight_name, mode);
        let bg_active = highlight;

        match mode {
            ThemeMode::Dark => Self {
                bg_base: Color::rgb(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0),
                bg_panel: Color::rgb(2.0 / 255.0, 2.0 / 255.0, 2.0 / 255.0),
                bg_sidebar: Color::rgb(1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0),
                bg_active,
                border: Color::rgb(3.0 / 255.0, 3.0 / 255.0, 3.0 / 255.0),
                highlight,
                accent,
                text_primary: Color::rgb(224.0 / 255.0, 224.0 / 255.0, 224.0 / 255.0),
                text_muted: Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0),
                text_dim: Color::rgb(68.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0),
            },
            ThemeMode::Light => Self {
                bg_base: Color::rgb(240.0 / 255.0, 240.0 / 255.0, 242.0 / 255.0),
                bg_panel: Color::WHITE,
                bg_sidebar: Color::rgb(230.0 / 255.0, 230.0 / 255.0, 234.0 / 255.0),
                bg_active,
                border: Color::rgb(210.0 / 255.0, 210.0 / 255.0, 215.0 / 255.0),
                highlight,
                accent,
                text_primary: Color::rgb(17.0 / 255.0, 17.0 / 255.0, 17.0 / 255.0),
                text_muted: Color::rgb(90.0 / 255.0, 90.0 / 255.0, 95.0 / 255.0),
                text_dim: Color::rgb(140.0 / 255.0, 140.0 / 255.0, 145.0 / 255.0),
            },
        }
    }
}

pub struct IconTheme {
    pub name: String,
}

impl IconTheme {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn get_icon_path(&self, category: &str, extension: &str) -> std::path::PathBuf {
        let prefix = if std::path::Path::new("apps/file_manager").exists() {
            "apps/file_manager/"
        } else {
            ""
        };
        let base = std::path::PathBuf::from(format!("{}assets/themes/{}/Gruvbox-Plus-Dark/mimetypes/scalable", prefix, self.name));
        let folders_base = std::path::PathBuf::from(format!("{}assets/themes/{}/folders", prefix, self.name));
        
        if category == "folder" {
            return folders_base.join("gray.png");
        }
        
        let filename = match extension {
            "cargo.toml" => "text-x-rust.svg",
            "cargo.lock" => "package-x-generic.svg",
            "pom.xml" => "text-x-maven+xml.svg",
            "build.gradle" | "settings.gradle" => "text-x-groovy.svg",
            "package.json" => "text-x-json.svg",
            "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "pnpm-lock.yml" | "bun.lock" | "bun.lockb" => "package-x-generic.svg",
            "makefile" => "text-x-script.svg",

            "pdf" => "application-pdf.svg",
            "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => "application-x-archive.svg",
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => "image-x-generic.svg",
            "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => "audio-x-generic.svg",
            "mp4" | "mkv" | "avi" | "webm" | "mov" | "flv" => "video-x-generic.svg",
            
            "sh" | "bash" | "zsh" | "bat" | "cmd" => "application-x-executable-script.svg",
            "ps1" => "application-x-powershell.svg",
            "exe" | "bin" | "msi" => "application-x-executable.svg",
            
            "txt" | "md" => "text-x-generic.svg",
            "csv" => "text-csv.svg",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet.svg",
            "doc" | "docx" | "odt" => "x-office-document.svg",
            
            "rs" => "text-x-rust.svg",
            "py" => "text-x-python.svg",
            "js" | "jsx" => "text-javascript.svg",
            "ts" | "tsx" => "text-typescript.svg",
            "go" => "text-x-go.svg",
            "java" => "text-x-java.svg",
            "kt" | "kts" => "text-x-kotlin.svg",
            "scala" => "text-x-scala.svg",
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => "text-x-cpp.svg",
            "html" => "text-html.svg",
            "css" | "scss" | "sass" | "less" => "text-css.svg",
            "php" => "application-x-php.svg",
            "rb" => "text-x-ruby.svg",
            "sql" => "text-x-sql.svg",
            "asm" | "s" | "assembly" => "text-x-asm.svg",
            
            "gitignore" | "dockerignore" | "env" | "editorconfig" | "babelrc" | "eslintrc" | "prettierrc" => "text-x-script.svg",
            "toml" | "json" | "yaml" | "yml" => "text-x-json.svg",
            "log" => "text-x-log.svg",
            
            "class" => "application-x-class-file.svg",
            "jar" | "war" => "application-x-java-archive.svg",
            "o" | "obj" | "a" | "lib" | "so" | "dll" | "dylib" => "application-x-shared-library-la.svg",

            _ => match category {
                "image" => "image-x-generic.svg",
                "text" => "text-x-generic.svg",
                "archive" => "application-x-archive.svg",
                "document" => "x-office-document.svg",
                "executable" => "application-x-executable.svg",
                "audio" => "audio-x-generic.svg",
                "video" => "video-x-generic.svg",
                _ => "unknown.svg",
            }
        };
        
        base.join(filename)
    }
}
