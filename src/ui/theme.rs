use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    pub background: Color,
    #[allow(dead_code)]
    pub surface: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::minimal_dark()
    }
}

impl Theme {
    pub fn minimal_dark() -> Self {
        Self {
            primary: Color::Rgb(135, 206, 250),   // Light blue
            secondary: Color::Rgb(100, 149, 237), // Cornflower blue
            accent: Color::Rgb(255, 99, 132),     // Red accent
            success: Color::Rgb(144, 238, 144),   // Light green
            warning: Color::Rgb(255, 165, 0),     // Orange
            danger: Color::Rgb(220, 20, 60),      // Crimson red
            info: Color::Rgb(135, 206, 250),      // Light blue
            background: Color::Rgb(16, 24, 32),   // Dark background
            surface: Color::Rgb(24, 32, 40),      // Slightly lighter
            text: Color::Rgb(255, 255, 255),      // Pure white
            text_dim: Color::Rgb(176, 196, 222),  // Light steel blue
            border: Color::Rgb(135, 206, 250),    // Light blue borders
            highlight: Color::Rgb(255, 99, 132),  // Red highlight
        }
    }

    #[allow(dead_code)]
    pub fn cyberpunk() -> Self {
        Self {
            primary: Color::Rgb(0, 255, 255),    // Cyan
            secondary: Color::Rgb(255, 0, 255),  // Magenta
            accent: Color::Rgb(0, 255, 127),     // Spring Green
            success: Color::Rgb(0, 255, 0),      // Lime
            warning: Color::Rgb(255, 255, 0),    // Yellow
            danger: Color::Rgb(255, 0, 0),       // Red
            info: Color::Rgb(135, 206, 235),     // Sky Blue
            background: Color::Rgb(16, 20, 31),  // Dark Blue
            surface: Color::Rgb(30, 35, 48),     // Darker Blue
            text: Color::Rgb(255, 255, 255),     // White
            text_dim: Color::Rgb(160, 160, 160), // Gray
            border: Color::Rgb(64, 224, 208),    // Turquoise
            highlight: Color::Rgb(255, 215, 0),  // Gold
        }
    }

    #[allow(dead_code)]
    pub fn matrix() -> Self {
        Self {
            primary: Color::Rgb(0, 255, 0),     // Matrix Green
            secondary: Color::Rgb(0, 200, 0),   // Dark Green
            accent: Color::Rgb(0, 255, 127),    // Light Green
            success: Color::Rgb(0, 255, 0),     // Green
            warning: Color::Rgb(255, 255, 0),   // Yellow
            danger: Color::Rgb(255, 0, 0),      // Red
            info: Color::Rgb(0, 255, 255),      // Cyan
            background: Color::Rgb(0, 0, 0),    // Black
            surface: Color::Rgb(10, 20, 10),    // Dark Green
            text: Color::Rgb(0, 255, 0),        // Green
            text_dim: Color::Rgb(0, 150, 0),    // Dim Green
            border: Color::Rgb(0, 200, 0),      // Green Border
            highlight: Color::Rgb(0, 255, 127), // Bright Green
        }
    }

    pub fn style_primary(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn style_secondary(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn style_warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn style_danger(&self) -> Style {
        Style::default().fg(self.danger)
    }

    pub fn style_info(&self) -> Style {
        Style::default().fg(self.info)
    }

    pub fn style_text(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn style_text_dim(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn style_highlight(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_selected(&self) -> Style {
        Style::default()
            .bg(self.primary)
            .fg(self.background)
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}

// Nerd Font Icons
pub struct Icons;

#[allow(dead_code)]
impl Icons {
    // General
    pub const WHISKER: &'static str = "🐱";
    pub const TERMINAL: &'static str = "";
    pub const COMMAND: &'static str = "";
    pub const SEARCH: &'static str = "";
    pub const TIME: &'static str = "";
    pub const CALENDAR: &'static str = "";

    // Status
    pub const SUCCESS: &'static str = "";
    pub const ERROR: &'static str = "";
    pub const WARNING: &'static str = "";
    pub const INFO: &'static str = "";
    pub const QUESTION: &'static str = "";

    // Navigation
    pub const ARROW_RIGHT: &'static str = "";
    pub const ARROW_LEFT: &'static str = "";
    pub const ARROW_UP: &'static str = "";
    pub const ARROW_DOWN: &'static str = "";

    // Tabs
    pub const SUMMARY: &'static str = "";
    pub const COMMANDS: &'static str = "";
    pub const SESSIONS: &'static str = "";
    pub const HOSTS: &'static str = "";
    pub const HEATMAP: &'static str = "";
    pub const ALIASES: &'static str = "";
    pub const DANGEROUS: &'static str = "";
    pub const NETWORK: &'static str = "";
    pub const PACKAGES: &'static str = "";
    pub const EXPERIMENTS: &'static str = "";

    // Hosts
    pub const LOCAL: &'static str = "";
    pub const SSH: &'static str = "";
    pub const DOCKER: &'static str = "";
    pub const KUBERNETES: &'static str = "☸";
    pub const CLOUD: &'static str = "";

    // Network
    pub const SECURE: &'static str = "";
    pub const INSECURE: &'static str = "";
    pub const HTTP: &'static str = "";
    pub const HTTPS: &'static str = "";
    pub const DATABASE: &'static str = "";

    // Packages
    pub const NPM: &'static str = "";
    pub const PYTHON: &'static str = "";
    pub const RUST: &'static str = "";
    pub const LINUX: &'static str = "";
    pub const APPLE: &'static str = "";
    pub const PACKAGE: &'static str = "";

    // Activity
    pub const FIRE: &'static str = "";
    pub const LIGHTNING: &'static str = "";
    pub const STAR: &'static str = "";
    pub const HEART: &'static str = "";
    pub const BRAIN: &'static str = "";

    // Misc
    pub const FOLDER: &'static str = "";
    pub const FILE: &'static str = "";
    pub const GEAR: &'static str = "";
    pub const CHART: &'static str = "";
    pub const GRAPH: &'static str = "";
}

pub fn get_host_icon(host_id: &str) -> &'static str {
    if host_id == "local" {
        Icons::LOCAL
    } else if host_id.starts_with("ssh:") {
        Icons::SSH
    } else if host_id.starts_with("docker:") {
        Icons::DOCKER
    } else if host_id.starts_with("k8s:") {
        Icons::KUBERNETES
    } else {
        Icons::CLOUD
    }
}

#[allow(dead_code)]
pub fn get_package_icon(manager: &str) -> &'static str {
    match manager {
        "npm" | "yarn" | "pnpm" => Icons::NPM,
        "pip" | "pip3" | "pipenv" | "poetry" => Icons::PYTHON,
        "cargo" => Icons::RUST,
        "apt" | "apt-get" | "yum" | "dnf" | "pacman" => Icons::LINUX,
        "brew" => Icons::APPLE,
        _ => Icons::PACKAGE,
    }
}

#[allow(dead_code)]
pub fn get_danger_icon(score: f32) -> (&'static str, Color) {
    if score > 0.8 {
        ("", Color::Red)
    } else if score > 0.5 {
        ("", Color::Yellow)
    } else if score > 0.2 {
        ("", Color::Blue)
    } else {
        ("", Color::Green)
    }
}

pub fn get_activity_icon(level: f32) -> &'static str {
    if level > 0.8 {
        Icons::FIRE
    } else if level > 0.6 {
        Icons::LIGHTNING
    } else if level > 0.4 {
        Icons::STAR
    } else if level > 0.2 {
        Icons::HEART
    } else {
        ""
    }
}

pub fn get_manager_info(manager: &str) -> (&'static str, ratatui::style::Color) {
    match manager {
        "npm" | "yarn" | "pnpm" => (Icons::NPM, Color::Red),
        "pip" | "pip3" | "pipenv" | "poetry" => (Icons::PYTHON, Color::Blue),
        "cargo" => (Icons::RUST, Color::Yellow),
        "apt" | "apt-get" | "yum" | "dnf" | "pacman" => (Icons::LINUX, Color::Green),
        "brew" => (Icons::APPLE, Color::White),
        "docker" => (Icons::DOCKER, Color::Cyan),
        _ => (Icons::PACKAGE, Color::Gray),
    }
}
