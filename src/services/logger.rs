use chrono::Local;
use colored::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct Logger {
    file_path: String,
}

impl Logger {
    pub fn new(file_path: &str, starting_balance: f64) -> Self {
        if !Path::new(file_path).exists() {
            File::create(file_path).expect("Failed to create log file");
        }

        let logger = Logger {
            file_path: file_path.to_string(),
        };
        logger.log_start(starting_balance);

        logger
    }
    fn log_start(&self, balance: f64) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_line = format!(
            "{} | LOG STARTED | Starting balance: {:.6}\n",
            timestamp, balance
        );
        print!("{}", log_line);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .expect("Cannot open log file");
        file.write_all(log_line.as_bytes())
            .expect("Failed to write to log file");
    }

    pub fn log_trade(&self, symbol: &str, action: &str, price: f64, amount: f64, pl_percent: f64) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let action_colored = match action.to_lowercase().as_str() {
            "buy" => action.green(),
            "sell" => action.red(),
            _ => action.normal(),
        };

        let pl_colored = if pl_percent >= 0.0 {
            format!("{:.2}%", pl_percent).green()
        } else {
            format!("{:.2}%", pl_percent).red()
        };

        let log_line = format!(
            "{} | {} | {} | Price: {:.6} | Amount: {:.6} | P/L: {}",
            timestamp, symbol, action_colored, price, amount, pl_colored
        );
        println!("{}", log_line);

        let log_line_plain = format!(
            "{} | {} | {} | Price: {:.6} | Amount: {:.6} | P/L: {:.2}%\n",
            timestamp, symbol, action, price, amount, pl_percent
        );
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .expect("Cannot open log file");
        file.write_all(log_line_plain.as_bytes())
            .expect("Failed to write to log file");
    }
}
