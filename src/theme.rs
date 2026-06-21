use zenthra::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_base: Color,
    pub bg_panel: Color,
    pub bg_sidebar: Color,
    pub bg_active: Color,
    pub border: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_dim: Color,
}

impl ThemeColors {
    pub fn get(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self {
                bg_base:      Color::rgb(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0), // pure black window bg
                bg_panel:     Color::rgb(2.0 / 255.0, 2.0 / 255.0, 2.0 / 255.0), // menu bar / toolbar
                bg_sidebar:   Color::rgb(1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0), // sidebar / list bg
                bg_active:    Color::rgba(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0, 0.15), // selected item bg (accent tint)
                border:       Color::rgb(3.0 / 255.0, 3.0 / 255.0, 3.0 / 255.0), // border / divider
                accent:       Color::rgb(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0), // vivid yellow accent
                text_primary: Color::rgb(224.0 / 255.0, 224.0 / 255.0, 224.0 / 255.0), // primary text
                text_muted:   Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0), // muted text
                text_dim:     Color::rgb(68.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0),    // dim/disabled text
            },
            ThemeMode::Light => Self {
                bg_base:      Color::rgb(248.0 / 255.0, 248.0 / 255.0, 246.0 / 255.0),
                bg_panel:     Color::WHITE,
                bg_sidebar:   Color::rgb(242.0 / 255.0, 242.0 / 255.0, 240.0 / 255.0),
                bg_active:    Color::rgba(200.0 / 255.0, 160.0 / 255.0, 0.0 / 255.0, 0.15),
                border:       Color::rgb(224.0 / 255.0, 224.0 / 255.0, 224.0 / 255.0),
                accent:       Color::rgb(200.0 / 255.0, 160.0 / 255.0, 0.0 / 255.0),
                text_primary: Color::rgb(26.0 / 255.0, 26.0 / 255.0, 26.0 / 255.0),
                text_muted:   Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0),
                text_dim:     Color::rgb(170.0 / 255.0, 170.0 / 255.0, 170.0 / 255.0),
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
            // Default to gray folder icon as requested
            return folders_base.join("gray.png");
        }
        
        let filename = match extension {
            // Exact Developer Manifests / Configuration files
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
            
            // Scripts and Executables
            "sh" | "bash" | "zsh" | "bat" | "cmd" => "application-x-executable-script.svg",
            "ps1" => "application-x-powershell.svg",
            "exe" | "bin" | "msi" => "application-x-executable.svg",
            
            // Documents & spreadsheets
            "txt" | "md" => "text-x-generic.svg",
            "csv" => "text-csv.svg",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet.svg",
            "doc" | "docx" | "odt" => "x-office-document.svg",
            
            // Dev languages
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
            
            // Dotfiles & Config
            "gitignore" | "dockerignore" | "env" | "editorconfig" | "babelrc" | "eslintrc" | "prettierrc" => "text-x-script.svg",
            "toml" | "json" | "yaml" | "yml" => "text-x-json.svg",
            "log" => "text-x-log.svg",
            
            // Java & Compiled Objects / Classes / Libraries
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
