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
    name_overwrite: String,
}

impl Window {
    fn new(send: Sender<SendComms>) -> Self {
        Self {
            muted: false,
            deafen: false,
            send,
            name_overwrite: String::new(),
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

            let server_text = match SHARED.server_name.try_read() {
                Ok(s) => (*s).clone(),
                Err(_) => "none".to_string(),
            };

            let connect_text = if SHARED.connected.try_read().is_ok_and(|x| *x) {
                format!("Connected to {server_text}")
            } else {
                "Disconnected".to_string()
            };

            ui.add_visible_ui(&server_text[..] == "Unnamed Northstar Server", |ui| {
                ui.small("this server isn't updated - proximity chat may break")
            });
            ui.horizontal(|ui| {
                ui.label(connect_text);
                
                ui.menu_button("copy", |ui| {
                    ui.label("server name copy thing :)");
                    ui.text_edit_singleline(&mut server_text.clone());
                });
            });
            ui.add_space(1.0);

            let text_mute = if self.muted { "Unmute" } else { " Mute " };
            let text_deafen = if self.deafen { "Undeafen" } else { " Deafen " };

            let mut should_send = false;

            ui.horizontal(|ui| {
                if ui.button(text_mute).clicked() {
                    self.muted = !self.muted;
                    should_send = true
                }

                if ui.button(text_deafen).clicked() {
                    self.deafen = !self.deafen;
                    should_send = true
                }
            });

            let mut name_overwrite = None;
            let mut reset_server_name = false;

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.name_overwrite)
                    .context_menu(|ui| {
                        if ui.button("Push Overwrite").clicked() && !self.name_overwrite.is_empty()
                        {
                            name_overwrite = Some(self.name_overwrite.clone());
                            should_send = true
                        }
                        if ui.button("reset").clicked() {
                            self.name_overwrite = server_text;

                            reset_server_name = true;
                            should_send = true
                        }
                    });
            });

            if should_send {
                self.send
                    .send(SendComms {
                        mute: self.muted,
                        deaf: self.deafen,
                        name_overwrite,
                        reset_server_name,
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
