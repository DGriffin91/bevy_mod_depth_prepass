use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_graph::{NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
            RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ExtractedView, ViewDepthTexture},
    },
};

use bevy::render::render_graph::Node;

use crate::prepass::{Opaque3dPrepass, ViewPrepassTextures};

pub struct PrepassNode {
    main_view_query: QueryState<
        (
            &'static ExtractedCamera,
            &'static RenderPhase<Opaque3dPrepass>,
            &'static ViewDepthTexture,
            &'static ViewPrepassTextures,
        ),
        With<ExtractedView>,
    >,
}

impl PrepassNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            main_view_query: QueryState::new(world),
        }
    }
}

impl Node for PrepassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(PrepassNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();
        if let Ok((camera, opaque_prepass_phase, view_depth_texture, view_prepass_textures)) =
            self.main_view_query.get_manual(world, view_entity)
        {
            if opaque_prepass_phase.items.is_empty() {
                return Ok(());
            }

            if let Some(view_normals_texture) =
                &gpu_images.get(&view_prepass_textures.normals_depth)
            {
                // Set up the pass descriptor with the depth attachment and optional color attachments
                let pass_descriptor = RenderPassDescriptor {
                    label: Some("prepass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view_normals_texture.texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK.into()),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &view_depth_texture.view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                };

                let render_pass = render_context
                    .command_encoder
                    .begin_render_pass(&pass_descriptor);
                let mut tracked_pass = TrackedRenderPass::new(render_pass);
                if let Some(viewport) = camera.viewport.as_ref() {
                    tracked_pass.set_camera_viewport(viewport);
                }

                {
                    let draw_functions = world.resource::<DrawFunctions<Opaque3dPrepass>>();

                    let mut draw_functions = draw_functions.write();
                    for item in &opaque_prepass_phase.items {
                        let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
                        draw_function.draw(world, &mut tracked_pass, view_entity, item);
                    }
                }
            }
        }

        Ok(())
    }
}
