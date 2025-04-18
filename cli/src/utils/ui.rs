use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
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

// Simple table formatting
pub fn create_table_header(headers: Vec<&str>) -> String {
    let header = headers.join(" | ");
    let separator = "-".repeat(header.len());
    format!("{}\n{}", header.bold(), separator)
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
pub fn key_value_table(title: &str, data: &[(&str, String)]) {
    println!("{}", section(title));
    
    let max_key_len = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    
    for (key, value) in data {
        let formatted_key = format!("{:width$}", key, width = max_key_len).cyan();
        println!("  {}: {}", formatted_key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_formatting_functions() {
        // Test title formatting
        let title_text = "Test Title";
        let title_result = title(title_text);
        assert!(title_result.contains(title_text));
        
        // Test section formatting
        let section_text = "Test Section";
        let section_result = section(section_text);
        assert!(section_result.contains(section_text));
        
        // Test success message
        let success_text = "Operation completed";
        let success_result = success(success_text);
        assert!(success_result.contains(success_text));
        
        // Test error message
        let error_text = "Something went wrong";
        let error_result = error(error_text);
        assert!(error_result.contains(error_text));
        
        // Test warning message
        let warning_text = "Proceed with caution";
        let warning_result = warning(warning_text);
        assert!(warning_result.contains(warning_text));
    }
}