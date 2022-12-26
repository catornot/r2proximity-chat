use crate::{comms::Comms, DISCORD};
use std::sync::mpsc::Receiver;

use eframe::{egui, epaint::Vec2, EventLoopBuilderHook, RequestRepaintEvent};
use egui_winit::winit::{
    event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
};

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
    recv: Receiver<Comms>,
    muted: bool,
}

impl Window {
    fn new(recv: Receiver<Comms>) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            recv,
            muted: false,
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

            ui.add_space(10.0);

            // todo: add connected status and other stuff :)

            let text_mute = if self.muted {
                "Mute"
            } else {
                "Unmute"
            };

            if ui.button(text_mute).clicked() {
                let mute = unsafe { DISCORD.client.self_muted() };
                if let Ok(mute) = mute {
                    self.muted = mute;

                    _ = unsafe { DISCORD.client.set_self_mute(!mute) };
                }
            }

            ui.add_space(10.0);
            ui.label("consider running this command: ");
            ui.text_edit_singleline(&mut String::from(
                "script_client CodeCallback_GetPlayerName()",
            ));

            ui.add_space(10.0);
            ui.small("REAL discord invite");
            ui.hyperlink("https://discord.gg/S7xsKuuhYb");

            if let Ok(comms) = self.recv.try_recv() {
                (self.x, self.y, self.z) = comms.into();
            }

            ui.add_space(5.0);
            ui.small("whar");
            ui.label(format!("ORIGIN {}, {}, {}", self.x, self.y, self.z));
        });
    }
}
