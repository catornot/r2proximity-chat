use eframe::egui;

use crate::comms::Comms;
use std::sync::mpsc::Receiver;

pub fn init_window(recv: Receiver<Comms>) {
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "Northstar Gaming",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc,recv))),
    );
}

// #[derive(Default)]
struct MyEguiApp {
    our_score: u32,
    there_score: u32,
    recv: Receiver<Comms>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, recv: Receiver<Comms>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { our_score: 0, there_score: 0, recv }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Info about you gaming");

            ui.label(format!("OUR SCORE {}", self.our_score));
            ui.label(format!("THERE SCORE {}", self.there_score));

            
            if let Ok(comms) = self.recv.try_recv() {
                self.our_score = comms.our_score;
                self.there_score = comms.there_score;
            }
        });
    }
}
