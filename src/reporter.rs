use colored::Colorize;
use std::fmt::Display;

use crate::{lexer::cursor::Cursor, parser::parse_err::ParseErr, src::Src};

pub enum ReportType {
    Info,
    Warning,
    Error,
}

impl Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ReportType::Info => "info".blue(),
            ReportType::Warning => "warning".yellow(),
            ReportType::Error => "error".red(),
        };
        write!(f, "{str}")
    }
}

pub struct Reporter;

impl Reporter {
    pub fn report_at(
        rtype: ReportType,
        msg: &str,
        src: &Src,
        cursor: Cursor,
        expected: Option<String>,
        found: Option<String>,
    ) {
        println!("{}: {}", rtype, msg.bold());
        println!(
            "{}{}:{}:{}:",
            "--> ".blue(),
            src.file.display().to_string().blue(),
            cursor.line.to_string().blue(),
            cursor.col.to_string().blue(),
        );

        let line = cursor.line;
        if line > 0 {
            println!(
                "{} {} {}",
                (line - 1).to_string().blue(),
                "|".blue(),
                src.lines[line - 1]
            );
        }
        println!(
            "{} {} {}",
            line.to_string().blue(),
            "|".blue(),
            src.lines[line]
        );
        print!("   {}{}", " ".repeat(cursor.col), "^ here: ".yellow());
        if let Some(estr) = expected {
            print!("expected '{}'", estr);
            if let Some(fstr) = found {
                print!(", found '{}'", fstr);
            }
            println!();
        } else {
            println!("{}", msg);
        }
        if line < src.lines.len() - 1 {
            println!(
                "{} {} {}",
                (line + 1).to_string().blue(),
                "|".blue(),
                src.lines[line + 1]
            );
        }
        println!();
    }

    pub fn info_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Info, msg, src, cursor, None, None);
    }

    pub fn warning_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Warning, msg, src, cursor, None, None);
    }

    pub fn error_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Error, msg, src, cursor, None, None);
    }

    pub fn parse_err_at(err: &ParseErr, src: &Src) {
        Reporter::report_at(
            ReportType::Error,
            err.msg.as_str(),
            src,
            err.cursor,
            err.expected.clone(),
            err.found.clone(),
        );
    }

    pub fn report(rtype: ReportType, msg: &str) {
        println!("{}: {}", rtype, msg.bold());
    }

    pub fn info(msg: &str) {
        Reporter::report(ReportType::Info, msg);
    }

    pub fn warning(msg: &str) {
        Reporter::report(ReportType::Warning, msg);
    }

    pub fn error(msg: &str) {
        Reporter::report(ReportType::Error, msg);
    }
}
