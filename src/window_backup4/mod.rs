use crate::comms::Comms;
use client::Client;
use std::sync::mpsc::Receiver;

use eframe::{egui, epaint::Vec2, EventLoopBuilderHook, RequestRepaintEvent};
use egui_winit::winit::{
    event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
};

mod client;

type EventLoopBuild = Option<EventLoopBuilderHook>;

pub fn init_window(recv: Receiver<Comms>) {
    let func = |event_loop_builder: &mut EventLoopBuilder<RequestRepaintEvent>| {
        event_loop_builder.with_any_thread(true);
    };

    let event_loop_builder: EventLoopBuild = Some(Box::new(func));

    let options = eframe::NativeOptions {
        always_on_top: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: Some(Vec2::new(400.0, 300.0)),
        resizable: false,
        follow_system_theme: false,
        run_and_return: false,
        event_loop_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Monarch ProxiChat",
        options,
        Box::new(|_cc| Box::new(Window::new(recv))),
    );
}

struct Window {
    x: i32,
    y: i32,
    z: i32,
    addr: String,
    recv: Receiver<Comms>,
    client: Client,
}

impl Window {
    fn new(recv: Receiver<Comms>) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            addr: String::from("localhost:7888"),
            recv,
            client: Client::new(),
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered(|ui| {
                ui.heading("Monarch ProxiChat");
            });
            ui.centered(|ui| {
                ui.small("Be the reason someone's country gets socialism.");
            });
            ui.end_row();
    
            if self.client.has_stream() {
                self.client.run();
            } else {
                ui.small("Enter your name");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.client.name);
                });
                
                ui.small("Enter the server's ip:port");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.addr);
                    if ui.button("connect").clicked() {
                        match self.client.connect(&self.addr) {
                            Ok(_) => {
                                log::info!("CONNECTION ESTABLISHED")
                            }
                            Err(err) => {
                                log::warn!("connection failed : {err:?}");
                                self.client.cancel();
                            }
                        }
                    }
                });
            }

            if let Ok(comms) = self.recv.try_recv() {
                (self.x, self.y, self.z) = comms.into();
            }

            ui.add_space(1.0);
            ui.label(format!("ORIGIN {}, {}, {}", self.x, self.y, self.z));
        });
    }
}
