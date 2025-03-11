use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use cli_table::{format::Justify, Cell, CellStruct, Style, Table, TableStruct};
use std::time::Duration;

// Define color constants
pub const PRIMARY_COLOR: &str = "blue";
pub const SUCCESS_COLOR: &str = "green";
pub const ERROR_COLOR: &str = "red";
pub const WARNING_COLOR: &str = "yellow";
pub const INFO_COLOR: &str = "cyan";
pub const HIGHLIGHT_COLOR: &str = "magenta";

// Progress bar styles
pub fn spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.blue} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn download_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// Text formatting helpers
pub fn title(text: &str) -> String {
    format!("\n{}\n{}\n", text.blue().bold(), "=".repeat(text.len()).blue())
}

pub fn section(text: &str) -> String {
    format!("\n{}\n{}\n", text.cyan().bold(), "-".repeat(text.len()).cyan())
}

pub fn success(text: &str) -> String {
    format!("✓ {}", text.green())
}

pub fn error(text: &str) -> String {
    format!("✗ {}", text.red().bold())
}

pub fn warning(text: &str) -> String {
    format!("! {}", text.yellow())
}

pub fn info(text: &str) -> String {
    format!("ℹ {}", text.cyan())
}

pub fn command(text: &str) -> String {
    format!("> {}", text.magenta())
}

// Table helpers
pub fn create_table(headers: Vec<&str>) -> TableStruct {
    let header_cells: Vec<CellStruct> = headers
        .into_iter()
        .map(|h| h.cell().bold(true).justify(Justify::Center))
        .collect();

    Table::new()
        .header(header_cells)
        .border(Style::modern())
        .separator(cli_table::format::Separator::Row)
}

// Function to list items in a table format
pub fn display_list<T: AsRef<str>>(title: &str, items: &[T]) {
    if items.is_empty() {
        println!("{}", info("No items to display"));
        return;
    }

    println!("{}", section(title));
    for (i, item) in items.iter().enumerate() {
        let number = format!("{}.", i + 1).cyan();
        println!("  {} {}", number, item.as_ref());
    }
}

// Function to create a table for key-value pairs
pub fn key_value_table(title: &str, data: Vec<(&str, &str)>) -> String {
    println!("{}", section(title));
    let mut result = String::new();
    
    let max_key_len = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    
    for (key, value) in data {
        let formatted_key = format!("{:width$}", key, width = max_key_len).cyan();
        result.push_str(&format!("  {}: {}\n", formatted_key, value));
    }
    
    result
}