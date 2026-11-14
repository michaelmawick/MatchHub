use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use eframe::egui;

pub struct Task<T> {
    receiver: Receiver<T>,
}

pub enum Poll<T> {
    Pending,
    Ready(T),
    Lost,
}

impl<T: Send + 'static> Task<T> {
    pub fn spawn(ctx: &egui::Context, job: impl FnOnce() -> T + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let _ = sender.send(job());
            ctx.request_repaint();
        });
        Self { receiver }
    }

    pub fn poll(&self) -> Poll<T> {
        match self.receiver.try_recv() {
            Ok(value) => Poll::Ready(value),
            Err(TryRecvError::Empty) => Poll::Pending,
            Err(TryRecvError::Disconnected) => Poll::Lost,
        }
    }
}
