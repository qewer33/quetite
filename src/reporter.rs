use std::fmt::{Display, write};

use crate::{lexer::cursor::Cursor, src::Src};

pub enum ReportType {
    Info,
    Warning,
    Error,
}

impl Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ReportType::Info => "info",
            ReportType::Warning => "warning",
            ReportType::Error => "error",
        };
        write!(f, "{str}")
    }
}

pub struct Reporter;

impl Reporter {
    pub fn report_at(rtype: ReportType, msg: &str, src: &Src, cursor: Cursor) {
        println!(
            "{}:{}:{}: {}: {}",
            src.file.file_name().unwrap().to_str().unwrap(),
            cursor.line,
            cursor.col,
            rtype,
            msg
        );

        let line = cursor.line;
        if line > 0 {
            println!("{} | {}", line - 1, src.lines[line - 1]);
        }
        println!("{} | {}", line, src.lines[line]);
        if line < src.lines.len() - 1 {
            println!("{} | {}", line + 1, src.lines[line + 1]);
        }
    }

    pub fn info_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Info, msg, src, cursor);
    }

    pub fn warning_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Warning, msg, src, cursor);
    }

    pub fn error_at(msg: &str, src: &Src, cursor: Cursor) {
        Reporter::report_at(ReportType::Error, msg, src, cursor);
    }

    pub fn report(rtype: ReportType, msg: &str) {
        println!("{}: {}", rtype, msg);
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
