use bevy::{prelude::*, window::CursorGrabMode};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use wasm_bindgen::prelude::wasm_bindgen;

mod vlox;

#[wasm_bindgen(start)]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .add_systems(Update, grab_mouse)
        .run();
}

fn update() {
    //info!("hello world");
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn grab_mouse(
    mut window: Single<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    // if key.just_pressed(KeyCode::Escape) {
    //     window.cursor_options.visible = true;
    //     window.cursor_options.grab_mode = CursorGrabMode::None;
    // }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
    ));
    //sphere
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(1.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
    //     Transform::from_xyz(8.0, 0.0, 0.0),
    // ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(0, 255, 0))),
    //     Transform::from_xyz(0.0, 8.0, 0.0),
    // ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(0, 0, 255))),
    //     Transform::from_xyz(0.0, 0.0, 8.0),
    // ));

    // Custom mesh
    let mut data = vlox::VloxData::new(8.0);
    data.set(0, 0, 0, 2, 1);

    data.set(1, 0, 0, 2, 1);
    data.set(2, 0, 0, 2, 1);
    data.set(3, 0, 0, 2, 1);

    data.set(0, 1, 0, 2, 1);
    data.set(0, 2, 0, 2, 1);
    data.set(0, 3, 0, 2, 1);

    data.set(0, 0, 1, 2, 1);
    data.set(0, 0, 2, 2, 1);
    data.set(0, 0, 3, 2, 1);

    commands.spawn((
        Mesh3d(meshes.add(data.compute_mesh_at_depth(2))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
