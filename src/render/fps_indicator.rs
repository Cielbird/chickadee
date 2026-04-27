use std::time::{self};
use wgpu_text::{
    glyph_brush::{
        ab_glyph::FontArc, BuiltInLineBreaker, Layout, OwnedSection, Section, Text, VerticalAlign,
    },
    BrushBuilder, TextBrush,
};

pub struct FpsIndicator {
    last_time: time::Instant,
    brush: TextBrush,
    section: OwnedSection,
}

impl FpsIndicator {
    pub fn new(device: &wgpu::Device, target: &wgpu::Texture, queue: &wgpu::Queue) -> Self {
        let texture_width = target.width();
        let texture_height = target.height();
        let texture_format = target.format();
        let font_size = 10.0;
        let font = FontArc::try_from_slice(include_bytes!("../../res/Rockwell.ttc")).unwrap();
        let brush = BrushBuilder::using_font(font)
            /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(device, texture_width, texture_height, texture_format);
        brush.resize_view(texture_width as f32, texture_height as f32, queue);

        let section = Section::default()
            .add_text(
                Text::new("")
                    .with_scale(font_size)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            )
            .with_bounds((texture_width as f32 * 0.3, texture_height as f32))
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .with_screen_position((0.0, 0.0))
            .to_owned();

        let last_time = time::Instant::now();
        Self {
            last_time,
            brush,
            section,
        }
    }

    pub fn draw(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let now = time::Instant::now();
        let dt = now - self.last_time;
        let fps = 1.0 / dt.as_secs_f32();
        self.last_time = now;

        self.section.text[0].text = format!(
            "\n
            FPS: {fps}\n"
        );

        // Crashes if inner cache exceeds limits.
        self.brush
            .queue(device, queue, [self.section.to_borrowed()])
            .unwrap();
        self.brush.draw(render_pass);
    }
}
