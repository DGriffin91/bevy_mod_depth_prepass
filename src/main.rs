//! A simple 3D scene with light shining over a cube sitting on a plane.

mod node;

mod prepass;

use bevy::{
    core_pipeline::fxaa::Fxaa,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use prepass::{update_materials, PrepassPlugin, ViewPrepassTextures};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PrepassPlugin)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        //Update materials that use prepass as texture when texture size changes
        .add_system(update_materials::<CustomMaterial>)
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
    let prepass = ViewPrepassTextures::new(window, &mut images, &msaa);

    // plane
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(CustomMaterial {
            depth_normal_prepass_texture: Some(prepass.normals_depth.clone()),
        }),
        ..default()
    });
    // cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(CustomMaterial {
            depth_normal_prepass_texture: Some(prepass.normals_depth.clone()),
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
        .insert(prepass)
        .insert(Fxaa::default());
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
