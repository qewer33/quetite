use std::fmt::{Display, write};

pub enum ReportType {
    Info,
    Warning,
    Error,
}

impl Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ReportType::Info => "Info",
            ReportType::Warning => "Warning",
            ReportType::Error => "Error",
        };
        write!(f, "{str}")
    }
}

pub struct Reporter;

impl Reporter {
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
