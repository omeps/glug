mod macurses;
use log::{set_logger, Level, Log, Record};
pub use macurses::Ansi8;
use macurses::Ansi8::*;
use macurses::*;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::OnceLock;
use std::{thread, usize};
type LogMessage = Result<(String, Level), GLoggerSignal>;
///The logger. Use `setup` or `setup_with_options` to initiate and `end` to stop.
#[derive(Clone)]
pub struct GLogger {
    channel: OnceLock<mpsc::Sender<LogMessage>>,
}
///Ways to configure the logger. These include: colors. 0.2.0 will expand these, if it ever gets
///made.
///
///# Examples
///```
/////build with default
///use glug::Ansi8;
///let options = glug::GLoggerOptions {colors: [Ansi8::Red,Ansi8::Blue,Ansi8::Green,Ansi8::Yellow,Ansi8::Yellow], ..Default::default()};
///```
///
///```
/////do it the easy way
///use glug::Ansi8;
///let options = glug::GLoggerOptions {colors: [Ansi8::Red,Ansi8::Blue,Ansi8::Green,Ansi8::Yellow,Ansi8::Yellow]};
///```
#[derive(Clone)]
pub struct GLoggerOptions {
    pub colors: [Ansi8; 5],
    pub save_to_file: Option<String>,
}
impl std::default::Default for GLoggerOptions {
    fn default() -> Self {
        Self {
            colors: [Red, Yellow, Green, Blue, Default],
            save_to_file: None,
        }
    }
}
#[derive(Debug, Copy, Clone)]
enum GLoggerSignal {
    Flush,
    Stop,
}
impl Log for GLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        //_ is better than allow()
        true
    }

    fn log(&self, record: &Record) {
        let log_message = record.args().to_string(); //to_string here so we own the referenced
        let log_level = record.level();
        if let Err(error) = self
            .channel
            .get()
            .expect("tried to log a message to a set-up logger but the log channel was not set up")
            .send(Ok((log_message, log_level)))
        {
            let (log_message, log_level) = error.0.clone().unwrap();
            panic!(
                "failed to send log {:?} at level {:?} due to {}",
                log_message, log_level, error
            )
        }
    }
    fn flush(&self) {
        if let Err(error) = self
            .channel
            .get()
            .expect("tried to log a message to a logger but the log channel was not set up")
            .send(Err(GLoggerSignal::Flush))
        {
            panic!("failed to send flush instruction due to {}", error)
        }
    }
}
impl GLogger {
    ///sets up the logger. The JoinHandle can
    ///be used to wait for writing to end, and the logger can tell the writing thread to
    ///end. Interchangable with `setup_with_options`.
    ///**The writing thread will only end if told to.**
    ///# Examples
    ///```
    ///fn main() {
    ///    let (writer, logger) = glug::GLogger::setup();
    ///    log::info!("logged a message");
    ///    logger.end();
    ///    writer.join().unwrap();
    ///}
    ///```
    pub fn setup() -> (thread::JoinHandle<()>, &'static GLogger) {
        Self::setup_with_options(GLoggerOptions::default())
    }
    ///sets up the logger with options. Interchangable with `GLogger::setup`
    ///# Examples
    ///```
    ///use glug::Ansi8;
    ///fn main() {
    ///    let (writer, logger) = glug::GLogger::setup_with_options(glug::GLoggerOptions { colors:
    ///    [Ansi8::Red,Ansi8::Yellow,Ansi8::Cyan,Ansi8::Magenta,Ansi8::Blue]});
    ///    log::info!("logged a message");
    ///    logger.end();
    ///    writer.join().unwrap();
    ///}
    ///```
    pub fn setup_with_options(
        options: GLoggerOptions,
    ) -> (thread::JoinHandle<()>, &'static GLogger) {
        static LOGGER: GLogger = GLogger {
            channel: OnceLock::new(),
        };
        set_logger(&LOGGER).expect("tried to set up logger twice");
        log::set_max_level(log::LevelFilter::Trace);
        let (sender, receiver) = channel();
        LOGGER
            .channel
            .set(sender)
            .expect("tried to set up logger twice");
        let t = thread::spawn(move || {
            GWriter {
                channel: receiver,
                logs: vec![],
                signals: vec![],
                log_colors: options.colors.map(|c| c as usize),
                log_counts: [0; 5],
                termwidth: 0,
                termlength: 0,
                file: match options.save_to_file {
                    Some(path) => match std::fs::File::create(path) {
                        Ok(f) => Some(f),
                        Err(e) => {
                            log::warn!("GLogger: failed to open file due to {}", e);
                            None
                        }
                    },
                    None => None,
                },
            }
            .log_loop();
        });
        (t, &LOGGER)
    }
    ///tells the writer to end writing.
    ///# Examples
    ///```
    ///fn main() {
    ///    let (writer, logger) = glug::GLogger::setup();
    ///    log::info!("logged a message");
    ///    logger.end();
    ///    writer.join().unwrap();
    ///}
    ///```
    pub fn end(&self) {
        if let Err(error) = self
            .channel
            .get()
            .expect("tried to log a message to a logger but the log channel was not set up")
            .send(Err(GLoggerSignal::Stop))
        {
            panic!("failed to send flush instruction due to {}", error)
        }
    }
}
struct GWriter {
    channel: mpsc::Receiver<LogMessage>,
    logs: Vec<(String, Level)>,
    signals: Vec<GLoggerSignal>,
    log_colors: [usize; 5],
    log_counts: [usize; 5],
    termwidth: usize,
    termlength: usize,
    file: Option<std::fs::File>,
}
fn nice_lines(string: &str, max_len: usize, level: Level) -> Vec<(String, Level)> {
    let mut lines = vec![];
    string.split('\n').for_each(|line| {
        line.chars()
            .collect::<Box<[char]>>()
            .chunks(max_len)
            .for_each(|l| lines.push((String::from_iter(l), level)))
    });
    lines
}
impl GWriter {
    fn log_loop(&mut self) {
        loop {
            self.read();
            self.draw();
            for signal in &self.signals {
                match signal {
                    GLoggerSignal::Flush => self.flush(),
                    GLoggerSignal::Stop => {
                        eprint!("{}\n", color!(0)); //reset color to gracefully exit
                        return;
                    }
                }
            }
        }
    }
    fn flush(&self) {
        todo!()
    }
    fn draw(&mut self) {
        let length = self.termlength - 6;
        for log in &self.logs {
            eprintln!(
                "{}{:<length$} ",
                color!(self.log_colors[log.1 as usize - 1]),
                log.0,
            );
        }
        for i in 0..self.termwidth {
            eprint!("{}", set_cursor!(i, length + 2));
            for j in 0..5 {
                eprint!(
                    "{}{} ",
                    match self.log_counts[j] >= self.termwidth - i {
                        true => color!(7),
                        false => color!(27),
                    },
                    color!(self.log_colors[j])
                );
            }
            eprint!("{}", color!(Reset as usize))
        }
    }

    fn read(&mut self) {
        self.logs.clear();
        self.signals.clear();
        (self.termwidth, self.termlength) = match termsize::get() {
            Some(size) => (size.rows as usize, size.cols as usize),
            None => {
                panic!("[glug] could not determine terminal size. Use another terminal or logger")
            }
        };
        let mut messages_received = 0;
        while let Ok(message) = self.channel.try_recv() {
            messages_received += 1;
            match message {
                Ok(log) => {
                    self.log_counts[log.1 as usize - 1] += 1;
                    let formatted = format!("{:<6}{}", log.1, log.0);
                    if let Some(file) = &mut self.file {
                        match file.write(format!("{}\n", formatted).as_bytes()) {
                            Ok(size) => {
                                if size <= formatted.as_bytes().len() {
                                    if let Err(e) = file.sync_all() {
                                        log::error!(
                                            "[glug] file sync failed. Check out what happened. Error: {}",e
                                        );
                                    }
                                    self.file = None;
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "[glug] file write failed. Check out what happened. Error: {}",
                                    e
                                );
                                self.file = None;
                            }
                        }
                    }
                    self.logs
                        .append(&mut nice_lines(&formatted, self.termlength - 6, log.1));
                }
                Err(signal) => self.signals.push(signal),
            }
            if messages_received >= 100 {
                return;
            }
        }
    }
}
