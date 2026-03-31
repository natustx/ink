use crate::parser;
use crate::parser::frontmatter;

/// Print document outline (heading structure).
pub fn print_outline(source: &str) {
    let (_, content) = frontmatter::strip_frontmatter(source);
    let doc = parser::parse(&content);

    if doc.headings.is_empty() {
        println!("  (no headings found)");
        return;
    }

    for h in &doc.headings {
        let indent = "  ".repeat((h.level as usize).saturating_sub(1));
        let marker = match h.level {
            1 => "█",
            2 => "▌",
            3 => "▎",
            _ => "·",
        };
        println!("{indent}{marker} {}", h.text);
    }
}

/// Print document statistics.
pub fn print_stats(source: &str, filename: &str) {
    let (fm, content) = frontmatter::strip_frontmatter(source);

    let doc = parser::parse(&content);

    let words = content.split_whitespace().count();
    let chars = content.chars().count();
    let lines = content.lines().count();
    let reading_time = (words as f64 / 200.0).ceil() as usize;
    let heading_count = doc.headings.len();
    let link_count = count_pattern(&content, "](");
    let code_block_count = content.matches("```").count() / 2;
    let image_count = count_pattern(&content, "![");
    let table_count = content.lines().filter(|l| l.contains("---|")).count();

    println!("╭─ {} ─╮", filename);
    println!("│");
    println!("│  Words:         {words}");
    println!("│  Characters:    {chars}");
    println!("│  Lines:         {lines}");
    println!("│  Reading time:  ~{reading_time} min");
    println!("│");
    println!("│  Headings:      {heading_count}");
    println!("│  Links:         {link_count}");
    println!("│  Code blocks:   {code_block_count}");
    println!("│  Images:        {image_count}");
    println!("│  Tables:        {table_count}");
    if fm.is_some() {
        println!("│  Frontmatter:   yes");
    }
    println!("│");
    println!("╰───╯");
}

/// Print a simple line-by-line diff between two markdown files.
pub fn print_diff(source_a: &str, source_b: &str, name_a: &str, name_b: &str) {
    let lines_a: Vec<&str> = source_a.lines().collect();
    let lines_b: Vec<&str> = source_b.lines().collect();

    println!("\x1b[1m--- {name_a}\x1b[0m");
    println!("\x1b[1m+++ {name_b}\x1b[0m");
    println!();

    let max_lines = lines_a.len().max(lines_b.len());

    for i in 0..max_lines {
        let a = lines_a.get(i).copied().unwrap_or("");
        let b = lines_b.get(i).copied().unwrap_or("");

        if a == b {
            println!("  {a}");
        } else {
            if !a.is_empty() || i < lines_a.len() {
                println!("\x1b[31m- {a}\x1b[0m");
            }
            if !b.is_empty() || i < lines_b.len() {
                println!("\x1b[32m+ {b}\x1b[0m");
            }
        }
    }
}

fn count_pattern(text: &str, pattern: &str) -> usize {
    text.matches(pattern).count()
}

/// Compute word count and reading time for status bar display.
pub fn document_stats(source: &str) -> (usize, usize) {
    let words = source.split_whitespace().count();
    let reading_time = (words as f64 / 200.0).ceil() as usize;
    (words, reading_time)
}
