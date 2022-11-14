use std::sync::mpsc::Receiver;

use eframe::{egui, epaint::Vec2};

use super::comms::Comms;
use super::utils::{DEFAULT_WAIT,wait};

pub fn init_window(recv: Receiver<Comms>) {
    env_logger::init();

    let options = eframe::NativeOptions {
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: Some(Vec2::new(500.0, 400.0)),
        resizable: false,
        follow_system_theme: false,
        run_and_return: false,
        ..Default::default()
    };

    eframe::run_native(
        "Server Window",
        options,
        Box::new(|_cc| Box::new(Window::new(recv))),
    );
}

struct Window {
    connected: Vec<String>,
    recv: Receiver<Comms>,
}

impl Window {
    fn new(recv: Receiver<Comms>) -> Self {
        Self {
            connected: Vec::new(),
            recv,
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Server Status");
            
            ui.add_space(1.0);
            

            for name in &self.connected {
                ui.horizontal(|ui| {
                    ui.label(name);
                    if ui.checkbox(&mut true, "Muted").clicked() {
                        println!("clicked")

                    }
                });
            }

            if let Ok(comms) = self.recv.try_recv() {
                self.connected = comms.connected;
            }

            wait(DEFAULT_WAIT);
        });
    }
}
