use chrono::{Date, DateTime, Local, Timelike};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(PartialEq, Clone, Copy)]
pub enum LogMode {
    File,
    Console,
    Both,
}

pub struct Logger {
    base_dir: PathBuf,
    cur_date: Date<Local>,
    last_log: DateTime<Local>,
    day_passed: bool,
}

impl Logger {
    fn gen_path(&self, source: &str, date: &Date<Local>) -> io::Result<PathBuf> {
        let base_dir = self.base_dir.as_path();
        let year_str = format!("{}", date.format("%Y"));
        let month_str = format!("{}", date.format("%m"));
        let day_str = format!("{}", date.format("%d"));

        let path = base_dir.join(source).join(year_str).join(month_str);
        fs::create_dir_all(&path)?;
        Ok(path.join(format!("{}.txt", day_str)))
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Logger {
        Logger {
            base_dir: path.as_ref().to_path_buf(),
            cur_date: Local::today(),
            last_log: Local::now(),
            day_passed: false,
        }
    }

    pub fn log_with_mode<P: AsRef<str>>(
        &mut self,
        source: &str,
        what: P,
        mode: LogMode,
    ) -> io::Result<()> {
        let now = Local::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S");
        let time_diff = now.signed_duration_since(self.last_log);

        if mode == LogMode::Console || mode == LogMode::Both {
            println!("[{}] {}: {}", now_str, source, what.as_ref());
        }

        if mode == LogMode::File || mode == LogMode::Both {
            if now.date() > self.last_log.date() {
                self.day_passed = true;
            }

            if self.day_passed && (time_diff.num_seconds() > 14400 || now.hour() >= 6) {
                self.cur_date = now.date();
                self.day_passed = false;
            }

            let path = self.gen_path(source, &self.cur_date)?;
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            let what = format!("[{}] {}\n", now_str, what.as_ref());
            file.write_all(what.as_bytes())?;
            self.last_log = now;
        }

        Ok(())
    }

    pub fn log<P: AsRef<str>>(&mut self, source: &str, what: P) -> io::Result<()> {
        self.log_with_mode(source, what, LogMode::Both)
    }
}
