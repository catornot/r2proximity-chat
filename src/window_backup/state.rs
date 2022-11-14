use egui::Context;
use egui_wgpu::{renderer::RenderPass, wgpu};
use egui_winit::winit::{self, event::WindowEvent, event_loop::EventLoop, window::Window};

use super::app::Ui;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub s: egui_winit::State,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window, event_loop: &EventLoop<()>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let _adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        #[allow(clippy::filter_next)]
        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .filter(|adapter| {
                // Check if this adapter supports our surface
                !surface.get_supported_formats(adapter).is_empty()
            })
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
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
            surface,
            device,
            queue,
            config,
            size,
            s: egui_winit::State::new(event_loop),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(
        &mut self,
        ui: &mut Ui,
        window: &Window,
        ctx: &Context,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("Render Pass"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //         view: &view,
        //         resolve_target: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Clear(wgpu::Color {
        //                 r: 0.1,
        //                 g: 0.2,
        //                 b: 0.3,
        //                 a: 1.0,
        //             }),
        //             store: true,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        // });

        let mut render_pass = RenderPass::new(&self.device, wgpu::TextureFormat::Bgra8UnormSrgb, 1);

        let screen_descriptor = {
            let size = window.inner_size();
            egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: window.scale_factor() as f32,
            }
        };

        let raw_input: egui::RawInput = self.s.take_egui_input(window);
        let full_output = ctx.run(raw_input, |ctx| {
            ui.update(ctx);
        });

        self.s
            .handle_platform_output(window, ctx, full_output.platform_output);

        let clipped_primitives = ctx.tessellate(full_output.shapes);

        println!( "{:?}", clipped_primitives );
        
        render_pass.update_buffers(
            &self.device,
            &self.queue,
            &clipped_primitives,
            &screen_descriptor,
        );
        for (tex_id, img_delta) in full_output.textures_delta.set {
            render_pass.update_texture(&self.device, &self.queue, tex_id, &img_delta);
        }
        for tex_id in full_output.textures_delta.free {
            render_pass.free_texture(&tex_id);
        }

        render_pass.execute(
            &mut encoder,
            &view,
            &clipped_primitives,
            &screen_descriptor,
            Some(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            }),
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    // fn render_ui(&mut self) {

    //     let raw_input: egui::RawInput = self.platform.take_egui_input(window);
    //     let full_output = self.ctx.run(raw_input, |ctx| {
    //         build_ui(ctx);
    //     });
    //     self.platform
    //         .handle_platform_output(window, &self.ctx, full_output.platform_output);

    //     let clipped_primitives = self.ctx().tessellate(full_output.shapes);

    //     self.renderer
    //         .update_buffers(device, queue, &clipped_primitives, &screen_descriptor);
    //     for (tex_id, img_delta) in full_output.textures_delta.set {
    //         self.renderer
    //             .update_texture(device, queue, tex_id, &img_delta);
    //     }
    //     for tex_id in full_output.textures_delta.free {
    //         self.renderer.free_texture(&tex_id);
    //     }

    //     let clear_color = match load_operation {
    //         wgpu::LoadOp::Clear(c) => Some(c),
    //         wgpu::LoadOp::Load => None,
    //     };
    // }
}
