use crate::PtyMessage;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::{
    io::{BufRead, BufReader, Write},
    thread::{self},
    time::Duration,
};
use std::thread::{sleep, spawn};

pub struct PtyManager {
    pty_pair: PtyPair,
    writer: Box<dyn Write + Send>,
}

impl PtyManager {
    pub fn new() -> Self {
        let pty_system = native_pty_system();

        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        #[cfg(target_os = "windows")]
        let cmd = CommandBuilder::new("powershell.exe");
        #[cfg(not(target_os = "windows"))]
        let cmd = CommandBuilder::new("zsh");

        let mut child = pty_pair.slave.spawn_command(cmd).unwrap();

        thread::spawn(move || {
            child.wait().unwrap();
        });

        let writer = pty_pair.master.take_writer().unwrap();

        Self {
            pty_pair,
            writer,
        }
    }

    pub fn start(&self) -> std::sync::mpsc::Receiver<PtyMessage> {
        let (tx, rx) = std::sync::mpsc::channel::<PtyMessage>();

        let reader = self.pty_pair.master.try_clone_reader().unwrap();

        spawn(move  || {
            let mut buf = BufReader::new(reader);
            loop {
                sleep(Duration::from_millis(1));
                let data = buf.fill_buf().unwrap().to_vec();
                buf.consume(data.len());
                if data.len() > 0 {
                    if let Err(e) = tx.send(PtyMessage::Buffer(data)) {
                        println!("send pty message error: {}", e);
                    }
                }
            }
        });
        rx
    }

    pub fn write_to_pty(&mut self, data: &str) -> Result<(), ()> {
        write!(self.writer, "{}", data).map_err(|_| ())
    }

    pub fn resize_pty(&self, rows: u16, cols: u16) -> Result<(), ()> {
        self.pty_pair
            .master
            .resize(PtySize {
                rows,
                cols,
                ..Default::default()
            })
            .map_err(|_| ())
    }
}
