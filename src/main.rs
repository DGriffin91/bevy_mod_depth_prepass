//! A simple 3D scene with light shining over a cube sitting on a plane.

mod node;
mod prepass;
mod prepass_render;

use bevy::{
    core_pipeline::core_3d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_graph::RenderGraph,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        RenderApp,
    },
    window::WindowResized,
};
use node::PrepassNode;
use prepass::ViewPrepassTextures;
use prepass_render::PrepassPlugin;

fn main() {
    App::new()
        // not working, haven't gotten everywhere configured with the same multisample
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PrepassNodePlugin)
        // PrepassPlugin probably doesn't need to be generic
        .add_plugin(PrepassPlugin::<CustomMaterial>::default())
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        //.add_system(window_resized)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut windows: ResMut<Windows>,
    mut images: ResMut<Assets<Image>>,
    msaa: Res<Msaa>,
) {
    let window = windows.get_primary_mut().unwrap();

    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        ..default()
    };
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST,
        },
        ..default()
    };
    // fill image.data with zeroes
    image.resize(size);
    let image_handle = images.add(image);

    // plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(CustomMaterial {
            depth_normal_prepass_texture: Some(image_handle.clone()),
        }),
        ..default()
    });
    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(CustomMaterial {
            depth_normal_prepass_texture: Some(image_handle.clone()),
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(ViewPrepassTextures {
            normals_depth: image_handle,
            size,
        });
}

// Not working
/*
Caused by:
    In a RenderPass
      note: encoder = `<CommandBuffer-(0, 1, Vulkan)>`
    In a pass parameter
      note: command buffer = `<CommandBuffer-(0, 1, Vulkan)>`
    attachments have differing sizes: ("depth", Extent3d { width: 1600, height: 900, depth_or_array_layers: 1 })
    is followed by ("color", Extent3d { width: 1280, height: 720, depth_or_array_layers: 1 })
*/
fn window_resized(
    mut window_resized_events: EventReader<WindowResized>,
    mut images: ResMut<Assets<Image>>,
    mut image_events: EventWriter<AssetEvent<Image>>,
    mut prepass_textures: Query<&mut ViewPrepassTextures>,
    mut mats: ResMut<Assets<CustomMaterial>>,
) {
    if let Some(event) = window_resized_events.iter().last() {
        dbg!(&event);
        let size = Extent3d {
            width: event.width as u32,
            height: event.height as u32,
            ..default()
        };

        if let Some(mut prepass_texture) = prepass_textures.iter_mut().next() {
            let image = images.get_mut(&prepass_texture.normals_depth).unwrap();
            image.resize(size);
            prepass_texture.size = size;
            image_events.send(AssetEvent::Modified {
                handle: prepass_texture.normals_depth.clone(),
            });
            for mat in mats.iter_mut() {
                //Touch material
            }
        }
    }
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "717f64fe-6844-4822-8926-e0ed374244c8"]
pub struct CustomMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub depth_normal_prepass_texture: Option<Handle<Image>>,
}

pub struct PrepassNodePlugin;
impl Plugin for PrepassNodePlugin {
    fn build(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        let prepass_node = PrepassNode::new(&mut render_app.world);
        let mut binding = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = binding.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

        draw_3d_graph.add_node("PREPASS", prepass_node);
        draw_3d_graph
            .add_slot_edge(
                draw_3d_graph.input_node().unwrap().id,
                core_3d::graph::input::VIEW_ENTITY,
                "PREPASS",
                PrepassNode::IN_VIEW,
            )
            .unwrap();
        draw_3d_graph
            .add_node_edge("PREPASS", core_3d::graph::node::MAIN_PASS)
            .unwrap();
    }
}
