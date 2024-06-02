#[macro_use]
mod macurses;
use log::{set_logger, Level, Log, Record};
pub use macurses::Ansi8;
use macurses::Ansi8::*;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::OnceLock;
use std::{thread, usize};
type LogMessage = Result<(String, Level), GLoggerSignal>;
#[derive(Clone)]
pub struct GLogger {
    channel: OnceLock<mpsc::Sender<LogMessage>>,
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
    pub fn setup() -> (thread::JoinHandle<()>, &'static GLogger) {
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
                log_colors: [
                    Red as usize,
                    Yellow as usize,
                    Green as usize,
                    Blue as usize,
                    Default as usize,
                ],
            }
            .log_loop();
        });
        (t, &LOGGER)
    }
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
                        println!("{}", color!(0));
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
        for log in &self.logs {
            println!(
                "{}{}, {}",
                color!(self.log_colors[log.1 as usize - 1]),
                log.1,
                log.0
            );
        }
    }
    fn read(&mut self) {
        self.logs.clear();
        self.signals.clear();
        while let Ok(message) = self.channel.try_recv() {
            match message {
                Ok(log) => self.logs.push(log),
                Err(signal) => self.signals.push(signal),
            }
        }
    }
}
