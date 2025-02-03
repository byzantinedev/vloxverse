use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind::{PINK_100, RED_500},
    picking::pointer::{Location, PointerId, PointerInteraction, PointerLocation},
    prelude::*,
    render::{
        camera::NormalizedRenderTarget,
        mesh::{Indices, PrimitiveTopology::TriangleList, VertexAttributeValues::Float32x3},
    },
    window::{CursorGrabMode, WindowRef},
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use uuid::Uuid;

mod vlox;

pub fn start() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, focus_camera)
        .add_systems(Update, grab_mouse)
        .add_systems(Update, draw_mesh_intersections)
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
    win: Single<(Entity, &Window)>,
) {
    let window_entity = win.0;
    let window = win.1;
    let width = window.width();
    let height = window.height();

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
        PointerId::Custom(Uuid::new_v4()),
        PointerLocation::new(Location {
            target: NormalizedRenderTarget::Window(
                WindowRef::Primary.normalize(Some(window_entity)).unwrap(),
            ),
            position: Vec2::new(width / 2.0, height / 2.0),
        }),
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

    let (vertices, normals, indices) = data.compute_mesh_at_depth(6);
    let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Float32x3(normals));
    mesh.insert_indices(Indices::U32(indices));
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<(&PointerInteraction, &PointerId)>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|(interaction, id)| {
            if !id.is_custom() {
                return None;
            } else {
                interaction.get_nearest_hit()
            }
        })
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}
