use std::time;
use wgpu_text::{
    BrushBuilder, TextBrush, glyph_brush::{
        BuiltInLineBreaker, Layout, OwnedSection, Section, SectionText, Text, ToSectionText, VerticalAlign, ab_glyph::FontArc
    }
};

pub struct FpsIndicator {
    last_time: time::Instant,
    brush: TextBrush,
    section: OwnedSection,
}

impl FpsIndicator {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {
        let font_size = 20.0;
        let font =
            FontArc::try_from_slice(include_bytes!("../../res/BorneCeline-Regular.otf")).unwrap();
        let brush = BrushBuilder::using_font(font)
            /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(device, config.width, config.height, config.format);
        brush.resize_view(config.width as f32, config.height as f32, &queue);

        let section = Section::default()
            .add_text(
                Text::new(
                    "Try typing some text,\n \
                del - delete all, backspace - remove last character",
                )
                .with_scale(font_size)
                .with_color([0.9, 0.5, 0.5, 1.0]),
            )
            .with_bounds((config.width as f32 * 0.4, config.height as f32))
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Center)
                    .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .with_screen_position((50.0, config.height as f32 * 0.5))
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
        let section = &self.section;

        // Crashes if inner cache exceeds limits.
        self.brush
            .queue(&device, &queue, [section])
            .unwrap();
        self.brush.draw(render_pass);
    }
}
