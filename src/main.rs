//! A simple 3D scene with light shining over a cube sitting on a plane.

mod node;

mod prepass;

use bevy::{
    core_pipeline::fxaa::Fxaa,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
    },
    window::WindowResized,
};

use prepass::{PrepassPlugin, ViewPrepassTextures};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PrepassPlugin)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PreUpdate, window_resized)
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
        width: window.physical_width() as u32,
        height: window.physical_height() as u32,
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
            sample_count: msaa.samples,
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
        })
        .insert(Fxaa::default());
}

fn window_resized(
    mut window_resized_events: EventReader<WindowResized>,
    mut images: ResMut<Assets<Image>>,
    mut image_events: EventWriter<AssetEvent<Image>>,
    mut prepass_textures: Query<&mut ViewPrepassTextures>,
    mut mats: ResMut<Assets<CustomMaterial>>,
    mut windows: ResMut<Windows>,
) {
    if let Some(_event) = window_resized_events.iter().last() {
        //event.width does not match window.physical_width()
        let window = windows.get_primary_mut().unwrap();

        let size = Extent3d {
            width: window.physical_width() as u32,
            height: window.physical_height() as u32,
            ..default()
        };

        if let Some(mut prepass_texture) = prepass_textures.iter_mut().next() {
            let image = images.get_mut(&prepass_texture.normals_depth).unwrap();
            image.resize(size);
            prepass_texture.size = size;
            image_events.send(AssetEvent::Modified {
                handle: prepass_texture.normals_depth.clone(),
            });
            for _ in mats.iter_mut() {
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
    //#[texture(0, multisampled = true)]
    #[texture(0)]
    #[sampler(1)]
    pub depth_normal_prepass_texture: Option<Handle<Image>>,
}
