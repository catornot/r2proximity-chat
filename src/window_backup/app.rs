use egui::{self, Context};
use egui_wgpu::wgpu;
use egui_winit::winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
    platform::windows::EventLoopBuilderExtWindows
};
// use rrplug::prelude::wait;
use std::sync::mpsc::Receiver;

use crate::comms::Comms;

use super::state::State;

pub struct Ui {
    our_score: u32,
    there_score: u32,
    recv: Receiver<Comms>,
}

impl Ui {
    pub fn new( recv: Receiver<Comms> ) -> Self {
        Self {
            our_score: 0,
            there_score: 0,
            recv
        }
    }
    pub fn update(&mut self, ctx: &Context) {
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

pub struct Win {
    
}

impl Win {
    pub fn new(recv: Receiver<Comms>) -> Self {
        // let mut egui_ctx = egui::CtxRef::default();




        env_logger::init();
        let event_loop = EventLoopBuilder::with_user_event()
                    .with_any_thread(true)
                    .build();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut gaming = Self {};
        
        futures::executor::block_on( gaming.run(event_loop, window, Ui::new(recv)) );

        gaming
    }
}

impl Win {
    

    pub async fn run(&mut self, event_loop: EventLoop<()>, window: Window, mut ui: Ui ) {
        let mut state = State::new(&window, &event_loop).await;

        // event_loop.run(move |event, _, control_flow| {
        //     // backend.handle_event(&event);

        //     match event {
        //         Event::MainEventsCleared => window.request_redraw(),
        //         // Event::RedrawRequested(_) => render(&wgpu, &window, &mut backend),

        //         Event::WindowEvent {
        //             event: WindowEvent::CloseRequested,
        //             ..
        //         } => {
        //             *control_flow = ControlFlow::Exit;
        //         }
        //         _ => {}
        //     }
        // });

        let ctx = Context::default();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => if !state.input(event) { // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    state.update();
                    match state.render(&mut ui, &window, &ctx) {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
