use std::borrow::Borrow;

use reerror::{throw, Context, Result};
use wgpu::{Color, CommandEncoder, RenderBundle, SurfaceTexture, TextureView};

use crate::{
    context::{device, gbuffer, queue, surface},
    filters::display::DisplayFilter,
};

#[derive(Debug)]
pub struct Frame<'a> {
    pub(crate) frame: SurfaceTexture,
    pub(crate) frame_view: TextureView,
    pub(crate) encoder: CommandEncoder,
    geom: Vec<&'a RenderBundle>,
    lights: Vec<&'a RenderBundle>,
    filters: Vec<&'a RenderBundle>,
}

impl<'a> Frame<'a> {
    pub fn present(mut self) {
        // TODO: Handle camera transform setup
        {
            let mut rpass = gbuffer().rpass(&mut self.encoder, Some(Color::BLACK));
            rpass.execute_bundles(self.geom);
        }

        {
            let mut rpass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lighting"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gbuffer().hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.execute_bundles(self.lights);
        }

        // really ugly way of generating a default list of filters but only creating them if there
        // is an empty filter list.
        //
        // This is ugly because we are regenerating the list every frame.
        // TODO: this should probably be recorded once and statically cached
        let display = if self.filters.is_empty() {
            Some(DisplayFilter::default())
        } else {
            None
        };

        let filters = if !self.filters.is_empty() {
            self.filters
        } else {
            vec![display.as_ref().unwrap().bundle()]
        };

        {
            let mut rpass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Filters"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            rpass.execute_bundles(filters);
        }

        let _ = queue().submit(Some(self.encoder.finish()));
        self.frame.present();
    }

    /// Try to fetch a new frame
    /// The frame will be renderered and presented when this object is dropped
    pub fn new() -> Result<Frame<'a>> {
        // get next frame
        let frame = surface()
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder =
            device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Ok(Frame {
            frame,
            frame_view,
            encoder,
            geom: vec![],
            lights: vec![],
            filters: vec![],
        })
    }

    pub fn draw_geom(&mut self, geom: &'a dyn Borrow<RenderBundle>) {
        self.geom.push(geom.borrow());
    }

    pub fn draw_light(&mut self, light: &'a dyn Borrow<RenderBundle>) {
        self.lights.push(light.borrow());
    }

    pub fn draw_filter(&mut self, filter: &'a dyn Borrow<RenderBundle>) {
        self.filters.push(filter.borrow());
    }
}
