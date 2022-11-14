use crate::comms::Comms;

use egui;
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};

use super::render::build_gui::BuildUI;

#[derive(Clone)]
pub struct Window {
    our_score: u32,
    there_score: u32,
    pub recv: Arc<Mutex<Receiver<Comms>>>,
}

impl Window {
    pub fn new(recv: Receiver<Comms>) -> Self {
        Self {
            our_score: 0,
            there_score: 0,
            recv: Arc::new(Mutex::new(recv)),
        }
    }
}

impl BuildUI for Window {
    fn build_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Northstar gaming").show(ctx, |ui| {
            ui.heading("Info about you gaming");

            ui.label(format!("OUR SCORE {}", self.our_score));
            ui.label(format!("THERE SCORE {}", self.there_score));

            if let Ok(lock) = self.recv.lock() {
                if let Ok(comms) = lock.try_recv() {
                    self.our_score = comms.our_score;
                    self.there_score = comms.there_score;
                }
            }
            println!("window created")
        });
    }
}

impl Default for Window {
    fn default() -> Self {
        let (_, recv) = channel::<Comms>();

        Self {
            our_score: 0,
            there_score: 0,
            recv: Arc::new(Mutex::new(recv)),
        }
    }
}
