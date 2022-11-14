use egui::Context;

pub trait BuildUI {
    fn build_ui( &mut self, ctx: &Context );
}