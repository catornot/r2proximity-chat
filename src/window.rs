use std::sync::mpsc::Sender;

use crate::comms::{SendComms, SHARED};

use eframe::{egui, epaint::Vec2, EventLoopBuilderHook, RequestRepaintEvent};
use egui_winit::winit::{
    event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
};

type EventLoopBuild = Option<EventLoopBuilderHook>;

pub fn init_window(send: Sender<SendComms>) {
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
        "Murzik's Proximity chat",
        options,
        Box::new(move |_cc| Box::new(Window::new(send))),
    );
}

struct Window {
    muted: bool,
    send: Sender<SendComms>,
}

impl Window {
    fn new(send: Sender<SendComms>) -> Self {
        Self { muted: false, send }
    }
}

impl eframe::App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered(|ui| {
                ui.heading("Murzik's Proximity chat");
            });
            ui.centered(|ui| {
                ui.small("Pet all the cats you see!");
            });
            ui.end_row();

            ui.add_space(10.0);

            let connect_text = if SHARED.connected.read().is_ok_and(|x| *x) {
                "Connected"
            } else {
                "Disconnected"
            };

            ui.label(connect_text);
            ui.add_space(1.0);

            // // todo: add connected status and other stuff :)

            let text_mute = if self.muted { "Unmute" } else { " Mute " };

            if ui.button(text_mute).clicked() {
                self.muted = !self.muted;

                self.send.send(SendComms { mute: self.muted }).unwrap();
            }

            ui.add_space(10.0);
            ui.label("consider running this command: ");
            ui.text_edit_singleline(&mut String::from(
                "script_client CodeCallback_GetPlayerName()",
            ));

            ui.add_space(10.0);
            ui.small("REAL discord invite");
            ui.hyperlink("https://discord.gg/S7xsKuuhYb");
        });
    }
}
