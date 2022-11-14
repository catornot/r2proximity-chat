use egui_winit::winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
use platform::winit::{event_loop::EventLoop, platform::windows::EventLoopBuilderExtWindows};
use std::sync::mpsc::Receiver;

use futures::executor::block_on;

use super::app::Window as Win;
use super::render::lib::*;
use crate::comms::Comms;

pub fn spawn_window(recv: Receiver<Comms>) {
    let ui = Win::new(recv);

    let event_loop: EventLoop<()> = EventLoopBuilder::with_user_event()
        .with_any_thread(true)
        .build();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_title("northstar gaming")
        .build(&event_loop)
        .unwrap();

    let mut wgpu = block_on(WgpuCtx::init(&window, ui));

    let mut backend = Backend::new(BackendDescriptor {
        device: &wgpu.device,
        rt_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        event_loop: &event_loop,
    });

    event_loop.run(move |event, _, control_flow| {
        backend.handle_event(&event);

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => render(&mut wgpu, &window, &mut backend),

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn render(wgpu: &mut WgpuCtx, window: &Window, backend: &mut Backend) {
    let mut encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    // let frame = match wgpu.swap_chain.get_current_texture() {
    //     Ok(frame) => frame,
    //     Err(e) => {
    //         eprintln!("wgpu error: {}", e);
    //         return;
    //     }
    // };
    // let rt = &frame.output.view;

    let output = match wgpu.surface.get_current_texture() {
        Ok(output) => output,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let rt = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    backend.render(
        RenderDescriptor {
            textures_to_update: &[],
            window,
            device: &wgpu.device,
            queue: &wgpu.queue,
            encoder: &mut encoder,
            render_target: &rt,
            load_operation: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        },
        &mut wgpu.ui,
    );

    wgpu.queue.submit(Some(encoder.finish()));
}

struct WgpuCtx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    ui: Win,
}

impl WgpuCtx {
    async fn init(window: &Window, ui: Win) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let size = window.inner_size();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            ui,
        }
    }
}
