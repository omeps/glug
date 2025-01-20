mod macurses;
pub mod termpin;
use log::{set_logger, warn, Level, Log, Record};
pub use macurses::Ansi8;
use macurses::Ansi8::*;
use macurses::*;
use options::GStoreOptions;
use std::cmp::max;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::mem::swap;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::OnceLock;
use std::thread::{JoinHandle, ThreadId};
use std::{thread, usize};
use termpin::Box2D;
type LogMessage = Result<(String, Level, GLoggerOptionalInfo), GLoggerSignal>;
///The logger. Use `setup` or `setup_with_options` to initiate and `end` to stop.
pub struct GLogger {
    channel: OnceLock<mpsc::Sender<LogMessage>>,
    enabled: OnceLock<GLoggerOptionalQuestions>,
}
#[derive(Debug)]
struct GLoggerOptionalQuestions {
    thread_fingerprint: Option<()>,
    timestamp: Option<()>,
}
impl From<GLoggerOptions> for GLoggerOptionalQuestions {
    fn from(value: GLoggerOptions) -> Self {
        Self {
            thread_fingerprint: value.record_threads.map(|_| ()),
            timestamp: value.timestamps,
        }
    }
}
///Ways to configure the logger.
///
///
///# Examples
///```
/////build with default.
///
///use glug::Ansi8;
///let options = glug::GLoggerOptions {colors: [Ansi8::Red,Ansi8::Blue,Ansi8::Green,Ansi8::Yellow,Ansi8::Yellow], ..Default::default()};
///```
///
///```
/////try your best.
///use glug::Ansi8;
///let options = glug::GLoggerOptions {
///     colors: [Ansi8::Red,
///         Ansi8::Blue,
///         Ansi8::Green,
///         Ansi8::Yellow,
///         Ansi8::Yellow],
///     save_to_file: None,
///     record_threads: None,
///     max_messages_per_loop: Some(100),
///     timestamps: Some(()),
///};
///```
#[derive(Clone, Debug)]
pub struct GLoggerOptions {
    ///which colors to use for logging. 0: Error, 4: Trace
    pub colors: [Ansi8; 5],
    ///what file to save logs to, if one is supplied.
    pub save_to_file: Option<String>,
    ///How to record which threads log what messages, if at all.
    pub record_threads: Option<options::RecordThreadsOptions>,
    ///how many messages to read before printing them.
    pub max_messages_per_loop: Option<usize>,
    ///whether or not to record timestamps.
    pub timestamps: Option<()>,
}
pub mod options {
    //!options to supply to `GLoggerOptions`.
    pub use super::macurses::Ansi8;
    use super::GLoggerOptionalInfo;
    use log::Level;
    use std::io::Write;
    ///Options for how to record threads, including `separate_histograms` and `summary`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RecordThreadsOptions {
        ///use only with big terminal.
        pub separate_histograms: bool,
        ///summary of logs printed at end of logging. Make sure the logger is quit properly.
        pub summary: bool,
    }
    pub struct GStoreOptions<'a, K: PartialEq> {
        pub log_colors: [usize; 5],
        pub separate_log_counts: Option<Box<dyn Fn(GLoggerOptionalInfo) -> Option<K>>>,
        pub writers: &'a mut [Result<Box<dyn Write>, std::io::Error>],
        pub format: &'a dyn Fn((String, Level, GLoggerOptionalInfo)) -> String,
    }
}

impl std::default::Default for GLoggerOptions {
    fn default() -> Self {
        Self {
            timestamps: Some(()),
            colors: [Red, Yellow, Green, Blue, Default],
            save_to_file: None,
            record_threads: Some(options::RecordThreadsOptions {
                separate_histograms: false,
                summary: false,
            }),
            max_messages_per_loop: Some(100),
        }
    }
}
#[derive(Debug, Copy, Clone)]
enum GLoggerSignal {
    Flush,
    Stop,
}
#[derive(Clone)]
pub struct GLoggerOptionalInfo {
    thread_fingerprint: Option<(ThreadId, Option<String>)>,
    timestamp: Option<chrono::DateTime<chrono::Local>>,
}

impl Display for GLoggerOptionalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_fingerprint = match &self.thread_fingerprint {
            Some((_, Some(name))) => format!("[{}]", name.to_string()),
            Some((id, None)) => format!("[id: {:?}]", id),
            None => "".to_string(),
        };
        let timestamp = match &self.timestamp {
            Some(time) => format!("@{:?}", time),
            None => "".to_string(),
        };
        write!(f, "{}{}", formatted_fingerprint, timestamp)
    }
}
impl Log for GLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        //_ is better than allow()
        true
    }

    fn log(&self, record: &Record) {
        let log_message = record.args().to_string(); //to_string here so we own the referenced
        let log_level = record.level();
        let info = GLoggerOptionalInfo {
            thread_fingerprint: if let Some(_) = &self
                .enabled
                .get()
                .expect("tried to log on a not set-up logger")
                .thread_fingerprint
            {
                Some((
                    std::thread::current().id(),
                    match std::thread::current().name() {
                        None => None,
                        Some(name) => Some(name.to_owned()),
                    },
                ))
            } else {
                None
            },
            timestamp: if let Some(_) = &self
                .enabled
                .get()
                .expect("tried to log on a not set-up logger")
                .timestamp
            {
                Some(chrono::Local::now())
            } else {
                None
            },
        };
        if let Err(error) = self
            .channel
            .get()
            .expect("tried to log a message to a log channel but the log channel was not set up")
            .send(Ok((log_message, log_level, info)))
        {
            let (log_message, log_level, _) = error.0.clone().unwrap();
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
pub struct GLoggerRef {
    pub handle: Option<JoinHandle<()>>,
    pub logger: &'static GLogger,
}
impl Drop for GLoggerRef {
    fn drop(&mut self) {
        self.logger.end();
        let mut handle = None;
        swap(&mut handle, &mut self.handle);
        handle.map(|h| h.join());
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
    ///    let gref = glug::GLogger::setup();
    ///    log::info!("logged a message");
    ///}
    ///```
    pub fn setup() -> GLoggerRef {
        Self::setup_with_options(GLoggerOptions::default())
    }
    ///sets up the logger with options. Interchangable with `GLogger::setup`
    ///# Examples
    ///```
    ///use glug::Ansi8;
    ///fn main() {
    ///    let gref = glug::GLogger::setup_with_options(glug::GLoggerOptions { colors:
    ///    [Ansi8::Red,Ansi8::Yellow,Ansi8::Cyan,Ansi8::Magenta,Ansi8::Blue], ..Default::default()});
    ///    log::info!("logged a message");
    ///}
    ///```
    pub fn setup_with_options(options: GLoggerOptions) -> GLoggerRef {
        static LOGGER: GLogger = GLogger {
            channel: OnceLock::new(),
            enabled: OnceLock::new(),
        };
        set_logger(&LOGGER).expect("[glug] tried to set up logger twice");
        log::set_max_level(log::LevelFilter::Trace);
        let (sender, receiver) = channel();
        LOGGER
            .channel
            .set(sender)
            .expect("[glug] tried to set up logger twice");
        LOGGER.enabled.set(options.clone().into()).unwrap();

        let writer_func = move || {
            let file_writer: Option<Result<Box<dyn std::io::Write>, _>> = match options.save_to_file
            {
                Some(path) => Some(match std::fs::File::create(path) {
                    Ok(f) => Ok(Box::new(f) as Box<dyn std::io::Write>),
                    Err(e) => {
                        log::warn!("[glug] failed to open file due to {}", e);
                        Err(e)
                    }
                }),
                None => None,
            };
            let mut writers: Vec<Result<Box<dyn std::io::Write>, std::io::Error>> = vec![];
            match file_writer {
                Some(Ok(f)) => writers.push(Ok(f)),
                Some(Err(e)) => writers.push(Err(e)),
                None => (),
            }
            let separate_log_counts = options.record_threads.map(|_| {
                Box::new(|g: GLoggerOptionalInfo| g.thread_fingerprint.map(|f| f.0))
                    as Box<dyn Fn(GLoggerOptionalInfo) -> Option<_>>
            });
            let format = Box::new(|p: (String, Level, GLoggerOptionalInfo)| {
                format!("{:<6}{} {}", p.1, p.2, p.0)
            });
            let mut terminal = termpin::DivNode::Element(Box::new(termpin::elements::draw_logs));
            terminal.place(
                termpin::DivNode::Element(Box::new(termpin::elements::draw_histogram)),
                (termpin::Direction::Right, Box::new(|x| max(x, 6) - 6)),
            );
            terminal.place(
                termpin::DivNode::Element(Box::new(termpin::elements::summary)),
                (termpin::Direction::Down, Box::new(|x| max(x, 6) - 6)),
            );
            GWriter {
                terminal,
                channel: receiver,
                signals: vec![],
                bound: Box2D {
                    x: 0,
                    y: 0,
                    length: 0,
                    height: 0,
                },
                max_messages_per_loop: options.max_messages_per_loop,
                store: GStoreOptions {
                    log_colors: options.colors.map(|c| c as usize),
                    separate_log_counts,
                    writers: &mut writers,
                    format: &format,
                }
                .into(),
            }
            .log_loop();
        };
        let t = thread::Builder::new()
            .name("glug writer".to_string())
            .spawn(writer_func);
        GLoggerRef {
            handle: Some(t.expect("unable to name writer thread `glug writer`")),
            logger: &LOGGER,
        }
    }
    ///tells the writer to end writing.
    ///# Examples
    ///```
    ///fn main() {
    ///    let gref = glug::GLogger::setup();
    ///    log::info!("logged a message");
    ///}
    ///```
    pub fn end(&self) {
        if let Err(error) = self
            .channel
            .get()
            .expect("[glug] tried to log a message to a logger but the log channel was not set up")
            .send(Err(GLoggerSignal::Stop))
        {
            panic!("[glug] failed to send flush instruction due to {}", error)
        }
    }
}
pub mod gstore {
    use super::{options::GStoreOptions, GLoggerOptionalInfo};
    use log::Level;
    use std::{
        collections::{HashMap, VecDeque},
        hash::Hash,
        io::Write,
    };
    pub struct GStore<'a, K: Eq + Hash> {
        logs: VecDeque<(String, Level, GLoggerOptionalInfo)>,
        pub counts_total: [usize; 5],
        pub counts_keyed: Option<(
            Box<dyn Fn(GLoggerOptionalInfo) -> Option<K>>,
            HashMap<K, [usize; 5]>,
        )>,
        pub log_colors: [usize; 5],
        writers: &'a mut [Result<Box<dyn Write>, std::io::Error>],
        format: &'a dyn Fn((String, Level, GLoggerOptionalInfo)) -> String,
    }
    impl<'a, K: Eq + Hash> From<GStoreOptions<'a, K>> for GStore<'a, K> {
        fn from(value: GStoreOptions<'a, K>) -> Self {
            Self {
                logs: VecDeque::with_capacity(512),
                counts_total: [0; 5],
                counts_keyed: match value.separate_log_counts {
                    Some(f) => Some((f, HashMap::new())),
                    None => None,
                },
                writers: value.writers,
                format: value.format,
                log_colors: value.log_colors,
            }
        }
    }
    impl<'a, K: Eq + Hash> GStore<'a, K> {
        pub fn insert(&mut self, log: (String, Level, GLoggerOptionalInfo)) {
            let (level, info) = (log.1, log.2.clone());
            let message = (self.format)(log);
            for writer in &mut *self.writers {
                if let Ok(w) = writer {
                    match write!(w, "{}{}", message, '\n') {
                        Ok(_) => (),
                        Err(e) => *writer = Err(e),
                    }
                }
            }
            self.counts_total[level as usize - 1] += 1;
            if let Some((get_key, store)) = &mut self.counts_keyed {
                let key = get_key(info.clone());
                match key.map(|key| (*store).get_mut(&key)) {
                    Some(Some(value)) => {
                        value[level as usize - 1] += 1;
                    }
                    Some(None) => {
                        store.insert(
                            get_key(info.clone()).unwrap(),
                            [0, 1, 2, 3, 4].map(|i| match i == level as usize - 1 {
                                true => 1,
                                false => 0,
                            }),
                        );
                    }
                    _ => {}
                }
            }
            self.logs.truncate(511);
            self.logs.push_front((message, level, info));
        }
        pub fn logs(&self) -> &VecDeque<(String, Level, GLoggerOptionalInfo)> {
            &self.logs
        }
    }
}
struct GWriter<'a, K: Eq + Hash> {
    //necessary fields
    terminal: termpin::DivNode<K>,
    channel: mpsc::Receiver<LogMessage>,
    signals: Vec<GLoggerSignal>,
    bound: Box2D<usize>,
    //fields for config
    max_messages_per_loop: Option<usize>,
    store: gstore::GStore<'a, K>,
}

impl<'a, K: Eq + Hash + Debug> GWriter<'a, K> {
    fn log_loop(&mut self) {
        loop {
            self.read();
            if let Err(e) = self.draw() {
                warn!("{}", e)
            }
            for signal in &self.signals {
                match signal {
                    GLoggerSignal::Flush => self.flush(),
                    GLoggerSignal::Stop => {
                        self.max_messages_per_loop = None;
                        eprint!(
                            "{}{}{}",
                            color!(0),
                            macurses::set_cursor!(self.bound.height, 0),
                            macurses::show_cursor!()
                        ); //reset color to gracefully exit
                        return;
                    }
                }
            }
        }
    }
    fn flush(&self) {
        todo!()
    }
    fn draw(&mut self) -> Result<(), String> {
        self.terminal.descend(self.bound, &self.store)
    }

    fn read(&mut self) {
        self.signals.clear();
        self.bound = match termsize::get() {
            Some(size) => size.into(),
            None => {
                panic!("[glug] could not determine terminal size. Use another terminal or logger")
            }
        };
        let mut messages_received = 0;
        while let Ok(message) = self.channel.try_recv() {
            messages_received += 1;
            match message {
                Ok(log) => {
                    self.store.insert(log);
                }
                Err(signal) => self.signals.push(signal),
            }
            if let Some(max) = self.max_messages_per_loop {
                if messages_received >= max {
                    return;
                }
            }
        }
    }
}
