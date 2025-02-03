use bevy::{prelude::*, window::CursorGrabMode};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};

mod vlox;

pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, focus_camera)
        .add_systems(Update, grab_mouse)
        .run();
}

fn focus_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        camera.look_at(Vec3::ZERO, Vec3::Y);
    }
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

    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
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
        Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
    ));

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
        Mesh3d(meshes.add(data.compute_mesh_at_depth(6))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
