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
    deafen: bool,
    send: Sender<SendComms>,
}

impl Window {
    fn new(send: Sender<SendComms>) -> Self {
        Self {
            muted: false,
            deafen: false,
            send,
        }
    }
}

impl eframe::App for Window {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered(|ui| {
                ui.heading("Murzik's Proximity chat");
                ui.end_row();
                ui.small("Pet all the cats you see!");
            });

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
            let text_deafen = if self.deafen { "Undeafen" } else { " Deafen " };

            // no mutable stuff :D
            let saved_mute = self.muted;
            let saved_deafen = self.deafen;

            ui.horizontal(|ui| {
                if ui.button(text_mute).clicked() {
                    self.muted = !self.muted;
                }

                if ui.button(text_deafen).clicked() {
                    self.deafen = !self.deafen;
                }
            });

            if saved_deafen != self.deafen || saved_mute != self.muted {
                self.send
                    .send(SendComms {
                        mute: self.muted,
                        deaf: self.deafen,
                    })
                    .unwrap();
            }

            ui.add_space(5.0);

            ui.collapsing("Members", |ui| {
                let members = match SHARED.members.try_write() {
                    Ok(lock) => lock,
                    Err(_) => return,
                };

                for member in members.iter() {
                    ui.label(member);
                }
            })
        });
    }
}
