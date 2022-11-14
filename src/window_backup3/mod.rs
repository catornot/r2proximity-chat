use crate::comms::Comms;
use egui_wgpu::wgpu::{self, Adapter, Device, Instance, Surface};
use egui_winit::{
    egui::{self, Context},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
        platform::windows::EventLoopBuilderExtWindows,
        window::{Window, WindowBuilder},
    },
};
use futures::executor::block_on;
use std::sync::mpsc::Receiver;

pub struct RenderDescriptor<'a> {
    pub textures_to_update: &'a [&'a egui::TextureId],
    pub window: &'a window::Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub render_target: &'a wgpu::TextureView,
    pub load_operation: wgpu::LoadOp<wgpu::Color>,
}

pub fn init_window(recv: Receiver<Comms>) {
    env_logger::init();

    let event_loop: EventLoop<()> = EventLoopBuilder::with_user_event()
        .with_any_thread(true)
        .build();
    let window = WindowBuilder::new()
        .with_title("NorthStar Gaming")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(init_adapter(&surface, &instance));

    let size = window.inner_size();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(window_id) if window_id == window.id() => render(),
            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        }
    });
}

async fn init_adapter(surface: &Surface, instance: &Instance) -> Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}

fn render(
    window: &Window,
    ctx: &Context,
    surface: &Surface,
    device: Device,
) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    render_pass.draw(vertices, instances);

    // let mut render_pass = RenderPass::new(device, wgpu::TextureFormat::Bgra8UnormSrgb, 1);

    let screen_descriptor = {
        let size = window.inner_size();
        egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: window.scale_factor() as f32,
        }
    };

    // let raw_input: egui::RawInput = window.take_egui_input(window);
    // let full_output = ctx.run(raw_input, |ctx| {
    //     ui.update(ctx);
    // });

    // self.s
    //     .handle_platform_output(window, ctx, full_output.platform_output);

    // let clipped_primitives = ctx.tessellate(full_output.shapes);

    // println!("{:?}", clipped_primitives);

    // render_pass.update_buffers(
    //     &self.device,
    //     &self.queue,
    //     &clipped_primitives,
    //     &screen_descriptor,
    // );
    // for (tex_id, img_delta) in full_output.textures_delta.set {
    //     render_pass.update_texture(&self.device, &self.queue, tex_id, &img_delta);
    // }
    // for tex_id in full_output.textures_delta.free {
    //     render_pass.free_texture(&tex_id);
    // }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
