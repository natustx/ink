mod app;
mod browser;
mod config;
mod image;
mod input;
mod layout;
mod parser;
mod render;
mod search;
mod stats;
mod theme;
mod toc;
mod watch;
mod wikilink;

use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(
    name = "ink",
    about = "The most advanced terminal markdown reader",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Markdown file path(s) or URL (reads stdin if omitted)
    #[arg(value_name = "FILE|URL")]
    pub input: Vec<String>,

    /// Color theme
    #[arg(short, long, default_value = "auto")]
    pub theme: String,

    /// Max rendering width in columns (or: narrow, wide, full)
    #[arg(short, long)]
    pub width: Option<String>,

    /// Presentation mode (split on ---)
    #[arg(short, long)]
    pub slides: bool,

    /// Plain output mode (no TUI, pipe-friendly)
    #[arg(short, long)]
    pub plain: bool,

    /// Watch file for changes and re-render
    #[arg(long)]
    pub watch: bool,

    /// Show table of contents on startup
    #[arg(long)]
    pub toc: bool,

    /// Disable image rendering
    #[arg(long)]
    pub no_images: bool,

    /// Show YAML/TOML frontmatter
    #[arg(long)]
    pub frontmatter: bool,

    /// Line spacing: compact, normal, relaxed
    #[arg(long, default_value = "normal")]
    pub spacing: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show document outline (heading structure)
    Outline {
        /// File to analyze
        file: String,
    },
    /// Show document statistics
    Stats {
        /// File to analyze
        file: String,
    },
    /// Show diff between two markdown files
    Diff {
        /// First file
        file_a: String,
        /// Second file
        file_b: String,
    },
    /// Print shell integration snippets (bash, zsh, fish)
    ShellSetup {
        /// Shell name: bash, zsh, or fish
        shell: String,
    },
}

/// Resolved arguments for the app.
#[derive(Clone)]
pub struct Args {
    pub inputs: Vec<String>,
    pub theme: String,
    pub width: Option<u16>,
    pub slides: bool,
    pub plain: bool,
    pub watch: bool,
    pub toc: bool,
    pub no_images: bool,
    pub frontmatter: bool,
    pub spacing: Spacing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spacing {
    Compact,
    Normal,
    Relaxed,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        return match cmd {
            Commands::Outline { file } => {
                let source = std::fs::read_to_string(file)?;
                stats::print_outline(&source);
                Ok(())
            }
            Commands::Stats { file } => {
                let source = std::fs::read_to_string(file)?;
                stats::print_stats(&source, file);
                Ok(())
            }
            Commands::Diff { file_a, file_b } => {
                let source_a = std::fs::read_to_string(file_a)?;
                let source_b = std::fs::read_to_string(file_b)?;
                stats::print_diff(&source_a, &source_b, file_a, file_b);
                Ok(())
            }
            Commands::ShellSetup { shell } => {
                print_shell_setup(shell);
                Ok(())
            }
        };
    }

    // Load config
    let user_config = config::load_config();

    let width = resolve_width(&cli.width, &user_config);
    let spacing = match cli.spacing.as_str() {
        "compact" => Spacing::Compact,
        "relaxed" => Spacing::Relaxed,
        _ => Spacing::Normal,
    };

    let theme = if cli.theme == "auto" {
        user_config
            .as_ref()
            .and_then(|c| c.theme.clone())
            .unwrap_or_else(|| "auto".to_string())
    } else {
        cli.theme.clone()
    };

    let args = Args {
        inputs: cli.input.clone(),
        theme,
        width,
        slides: cli.slides,
        plain: cli.plain,
        watch: cli.watch,
        toc: cli.toc,
        no_images: cli.no_images,
        frontmatter: cli.frontmatter,
        spacing,
    };

    // Check if input is a directory or no input with a TTY → launch file browser
    let browse_dir = if args.inputs.is_empty() {
        if std::io::stdin().is_terminal() {
            Some(std::env::current_dir()?)
        } else {
            None
        }
    } else {
        let path = PathBuf::from(&args.inputs[0]);
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    };

    if let Some(dir) = browse_dir {
        // File browser loop — user returns here after closing a file
        while let Some(selected) = browser::browse(&dir, &args.theme)? {
            let source = std::fs::read_to_string(&selected)?;
            let mut file_args = args.clone();
            file_args.inputs = vec![selected.to_string_lossy().to_string()];
            if file_args.plain {
                let rendered = render::plain::render_plain(&source, &file_args)?;
                print!("{rendered}");
                break;
            } else {
                app::run(source, file_args)?;
                // After viewer exits, return to browser
            }
        }
        return Ok(());
    }

    emit_watch_warnings(&args);
    let source = read_input(&args)?;

    if args.plain {
        let rendered = render::plain::render_plain(&source, &args)?;
        print!("{rendered}");
        return Ok(());
    }

    app::run(source, args)?;

    Ok(())
}

fn resolve_width(width_str: &Option<String>, config: &Option<config::Config>) -> Option<u16> {
    if let Some(w) = width_str {
        match w.as_str() {
            "narrow" => return Some(60),
            "wide" => return Some(100),
            "full" => return None,
            _ => {
                if let Ok(n) = w.parse::<u16>() {
                    return Some(n);
                }
            }
        }
    }
    config.as_ref().and_then(|c| c.width)
}

fn watch_warnings(args: &Args, stdin_is_tty: bool) -> Vec<&'static str> {
    if !args.watch {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    if args.plain {
        warnings.push("ink: --watch has no effect in --plain mode");
    }

    if args.inputs.is_empty() && !stdin_is_tty {
        warnings.push("ink: --watch has no effect for stdin input");
    }

    if let Some(input) = args.inputs.first() {
        if input.starts_with("http://") || input.starts_with("https://") {
            warnings.push("ink: --watch is not yet supported for URL input");
        }
    }

    warnings
}

fn emit_watch_warnings(args: &Args) {
    for warning in watch_warnings(args, std::io::stdin().is_terminal()) {
        eprintln!("{warning}");
    }
}

fn print_shell_setup(shell: &str) {
    match shell.to_lowercase().as_str() {
        "bash" | "zsh" => {
            println!(
                r#"# ink — terminal markdown reader
# Add these lines to your ~/.{shell}rc:

# Quick alias to view markdown
alias md="ink"

# Browse markdown files in current directory
alias mdb="ink ."

# Use ink for fzf markdown preview
export FZF_DEFAULT_OPTS='--preview "ink --plain {{}} 2>/dev/null"'

# Use ink as a git pager for markdown diffs
# git config --global diff.markdown.textconv "ink --plain""#
            );
        }
        "fish" => {
            println!(
                r#"# ink — terminal markdown reader
# Add these lines to your ~/.config/fish/config.fish:

# Quick alias to view markdown
alias md "ink"

# Browse markdown files in current directory
alias mdb "ink .""#
            );
        }
        _ => {
            eprintln!(
                "ink: unsupported shell '{}'. Supported: bash, zsh, fish",
                shell
            );
        }
    }
}

fn read_input(args: &Args) -> Result<String> {
    if args.inputs.is_empty() {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }

    let input = &args.inputs[0];
    if input.starts_with("http://") || input.starts_with("https://") {
        let resp = reqwest::blocking::get(input)?;
        Ok(resp.text()?)
    } else {
        let path = PathBuf::from(input);
        Ok(std::fs::read_to_string(&path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> Args {
        Args {
            inputs: vec![],
            theme: "dark".to_string(),
            width: None,
            slides: false,
            plain: false,
            watch: true,
            toc: false,
            no_images: true,
            frontmatter: false,
            spacing: Spacing::Normal,
        }
    }

    #[test]
    fn watch_plain_emits_warning() {
        let mut args = args();
        args.plain = true;

        assert!(watch_warnings(&args, true).contains(&"ink: --watch has no effect in --plain mode"));
    }

    #[test]
    fn watch_stdin_emits_warning() {
        let args = args();

        assert!(
            watch_warnings(&args, false).contains(&"ink: --watch has no effect for stdin input")
        );
    }
}
