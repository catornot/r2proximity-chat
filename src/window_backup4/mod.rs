use crate::comms::Comms;
use std::sync::mpsc::Receiver;

use eframe::{egui, epaint::Vec2, EventLoopBuilderHook, RequestRepaintEvent};
use egui_winit::winit::{
    event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
};

type EventLoopBuild = Option<EventLoopBuilderHook>;

pub fn init_window(recv: Receiver<Comms>) {
    env_logger::init();

    let func = |event_loop_builder: &mut EventLoopBuilder<RequestRepaintEvent>| {
        event_loop_builder.with_any_thread(true);
    };

    let event_loop_builder: EventLoopBuild = Some(Box::new(func));

    let options = eframe::NativeOptions {
        always_on_top: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: Some(Vec2::new(300.0, 200.0)),
        resizable: false,
        follow_system_theme: false,
        run_and_return: false,
        event_loop_builder,
        ..Default::default()
    };

    eframe::run_native(
        "NorthstarGaming",
        options,
        Box::new(|_cc| Box::new(Window::new(recv))),
    );
}

struct Window {
    x: i32,
    y: i32,
    z: i32,
    recv: Receiver<Comms>,
}

impl Window {
    fn new(recv: Receiver<Comms>) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            recv,
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Info about you gaming");
            
            ui.add_space(1.0);
            ui.label(format!("ORIGIN {}, {}, {}", self.x, self.y, self.z));

            if let Ok(comms) = self.recv.try_recv() {
                (self.x, self.y, self.z) = comms.into();
            }
        });
    }
}
