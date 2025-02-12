use bevy::{
    asset::RenderAssetUsages,
    picking::pointer::{Location, PointerId, PointerInteraction, PointerLocation},
    prelude::*,
    render::{
        camera::NormalizedRenderTarget,
        mesh::{
            Indices, MeshAabb,
            PrimitiveTopology::TriangleList,
            VertexAttributeValues::{Float32x3, Float32x4},
        },
    },
    window::{CursorGrabMode, WindowMode, WindowRef},
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use uuid::Uuid;
use vlox::VloxData;

mod vlox;

const DEPTH_TO_UNIT: u8 = 2;

const MIN_VLOX_DEPTH: u8 = 1;
const MAX_VLOX_DEPTH: u8 = 5;
const INITIAL_VLOX_DEPTH: u8 = 5;
const COMPUTE_MESH_DEPTH: u8 = 5;

const CONTROLS_VLOX_SIZE_UP: KeyCode = KeyCode::Equal;
const CONTROLS_VLOX_SIZE_DOWN: KeyCode = KeyCode::Minus;

pub fn start() {
    let mut app = App::new();

    app.add_plugins(MeshPickingPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NoCameraPlayerPlugin)
        .init_resource::<VloxSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, pause_resume)
        .add_systems(Update, focus_camera)
        .add_systems(Update, edit_mesh)
        .add_systems(Update, update_pointer_location);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_systems(Startup, || {
            let canvas: web_sys::HtmlCanvasElement = wasm_bindgen::JsCast::unchecked_into(
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector("canvas")
                    .unwrap()
                    .unwrap(),
            );
            let style = canvas.style();
            style.set_property("width", "100%").unwrap();
            style.set_property("height", "100%").unwrap();
        });
    }

    app.run();
}

fn update_pointer_location(
    mut pointer: Single<&mut PointerLocation, With<Camera>>,
    win: Single<(Entity, &Window)>,
) {
    if let Some(location) = &mut pointer.location {
        let window = win.1;
        let width = window.width();
        let height = window.height();

        location.position = Vec2::new(width / 2.0, height / 2.0)
    }
}
fn focus_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        camera.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn pause_resume(
    mut window: Single<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.any_just_pressed(vec![MouseButton::Left, MouseButton::Right]) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;

        #[cfg(target_arch = "wasm32")]
        {
            let canvas: web_sys::HtmlCanvasElement = wasm_bindgen::JsCast::unchecked_into(
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector("canvas")
                    .unwrap()
                    .unwrap(),
            );
            canvas.request_fullscreen().unwrap();
        }
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
    mut vlox_settings: ResMut<VloxSettings>,
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
    vlox_settings.data = vlox::VloxData::new(DEPTH_TO_UNIT);
    vlox_settings.data.set(0, 0, 0, 2, 1);

    vlox_settings.data.set(1, 0, 0, 2, 1);
    vlox_settings.data.set(2, 0, 0, 2, 1);
    vlox_settings.data.set(3, 0, 0, 2, 1);

    vlox_settings.data.set(0, 1, 0, 2, 1);
    vlox_settings.data.set(0, 2, 0, 2, 1);
    vlox_settings.data.set(0, 3, 0, 2, 1);

    vlox_settings.data.set(0, 0, 1, 2, 1);
    vlox_settings.data.set(0, 0, 2, 2, 1);
    vlox_settings.data.set(0, 0, 3, 2, 1);

    vlox_settings.selected_depth = INITIAL_VLOX_DEPTH;
    vlox_settings.selected_value = 1;
    vlox_settings.materials.set(0, vlox::Material::Void);
    vlox_settings.materials.set(
        1,
        vlox::Material::Solid(vlox::SolidMaterial {
            name: "White".to_string(),
            data: VloxData::new(0),
            colors: vec![vlox::Color::new(1.0, 1.0, 1.0, 1.0)],
        }),
    );
    vlox_settings.materials.set(
        2,
        vlox::Material::Solid(vlox::SolidMaterial {
            name: "Red".to_string(),
            data: VloxData::new(0),
            colors: vec![vlox::Color::new(1.0, 0.0, 0.0, 1.0)],
        }),
    );
    vlox_settings.materials.set(
        3,
        vlox::Material::Solid(vlox::SolidMaterial {
            name: "Green".to_string(),
            data: VloxData::new(0),
            colors: vec![vlox::Color::new(0.0, 1.0, 0.0, 1.0)],
        }),
    );
    vlox_settings.materials.set(
        4,
        vlox::Material::Solid(vlox::SolidMaterial {
            name: "Blue".to_string(),
            data: VloxData::new(0),
            colors: vec![vlox::Color::new(0.0, 0.0, 1.0, 1.0)],
        }),
    );

    let (vertices, normals, colors, indices) = vlox_settings
        .data
        .compute_mesh_at_depth(COMPUTE_MESH_DEPTH, &vlox_settings.materials);
    let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
    set_vlox_mesh(&mut mesh, vertices, normals, colors, indices);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MainMesh,
    ));
}

/// A system that draws hit indicators for every pointer.
fn edit_mesh(
    pointers: Query<(&PointerInteraction, &PointerId)>,
    mut gizmos: Gizmos,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut vlox_settings: ResMut<VloxSettings>,
    main_mesh: Single<(&Mesh3d, &MainMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
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
        let depth = vlox_settings.selected_depth;
        let vlox_size = vlox_settings
            .data
            .vlox_size(vlox_settings.data.num_vlox(depth));
        let half_vlox = vlox_size / 2.0;

        let preview_xyz = point + normal * half_vlox;
        let (vx, vy, vz) = vlox_settings.data.xyz_f32_to_vlox_xyz(
            preview_xyz.x,
            preview_xyz.y,
            preview_xyz.z,
            depth,
        );
        let (preview_x, preview_y, preview_z) =
            vlox_settings.data.vlox_xyz_to_xyz_f32(vx, vy, vz, depth);
        let preview_xyz = Vec3::new(preview_x, preview_y, preview_z);
        gizmos.cuboid(
            Transform::from_translation(preview_xyz).with_scale(Vec3::ONE * vlox_size),
            Color::WHITE,
        );

        let (vx, vy, vz) = vlox_settings
            .data
            .xyz_f32_to_vlox_xyz(point.x, point.y, point.z, depth);
        println!("{},{},{} depth: {}", vx, vy, vz, depth);

        if mouse_button_input.just_pressed(MouseButton::Left) {
            let point = point + normal * half_vlox;

            let bounds = vlox_settings.data.size() / 2.0;
            if !(point.x >= bounds
                || point.x <= -bounds
                || point.y >= bounds
                || point.y <= -bounds
                || point.z >= bounds
                || point.z <= -bounds)
            {
                if let Some(mesh) = meshes.get_mut(&main_mesh.0 .0) {
                    let (vx, vy, vz) = vlox_settings
                        .data
                        .xyz_f32_to_vlox_xyz(point.x, point.y, point.z, depth);
                    let selected_value = vlox_settings.selected_value;
                    vlox_settings.data.set(vx, vy, vz, depth, selected_value);

                    let (vertices, normals, colors, indices) = vlox_settings
                        .data
                        .compute_mesh_at_depth(COMPUTE_MESH_DEPTH, &vlox_settings.materials);
                    set_vlox_mesh(mesh, vertices, normals, colors, indices);
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Right) {
            let depth = vlox_settings.selected_depth;
            let half_vlox = vlox_settings
                .data
                .vlox_size(vlox_settings.data.num_vlox(depth))
                / 2.0;
            let point = point - normal * half_vlox;

            let bounds = vlox_settings.data.size() / 2.0;
            if !(point.x >= bounds
                || point.x <= -bounds
                || point.y >= bounds
                || point.y <= -bounds
                || point.z >= bounds
                || point.z <= -bounds)
            {
                if let Some(mesh) = meshes.get_mut(&main_mesh.0 .0) {
                    let (vx, vy, vz) = vlox_settings
                        .data
                        .xyz_f32_to_vlox_xyz(point.x, point.y, point.z, depth);
                    vlox_settings.data.set(vx, vy, vz, depth, 0);

                    let (vertices, normals, colors, indices) = vlox_settings
                        .data
                        .compute_mesh_at_depth(COMPUTE_MESH_DEPTH, &vlox_settings.materials);
                    set_vlox_mesh(mesh, vertices, normals, colors, indices);
                    info!("updated mesh")
                }
            }
        }
    }
    if keyboard_input.just_pressed(CONTROLS_VLOX_SIZE_UP)
        && vlox_settings.selected_depth > MIN_VLOX_DEPTH
    {
        vlox_settings.selected_depth -= 1;
        println!("new depth: {}", vlox_settings.selected_depth);
    }
    if keyboard_input.just_pressed(CONTROLS_VLOX_SIZE_DOWN)
        && vlox_settings.selected_depth < MAX_VLOX_DEPTH
    {
        vlox_settings.selected_depth += 1;
        println!("new depth: {}", vlox_settings.selected_depth);
    }

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        vlox_settings.selected_value = 1;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        vlox_settings.selected_value = 2;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        vlox_settings.selected_value = 3;
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        vlox_settings.selected_value = 4;
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        //vlox_settings.selected_value = 5;
    }
    if keyboard_input.just_pressed(KeyCode::Digit6) {
        //vlox_settings.selected_value = 6;
    }
    if keyboard_input.just_pressed(KeyCode::Digit7) {
        //vlox_settings.selected_value = 7;
    }
    if keyboard_input.just_pressed(KeyCode::Digit8) {
        //vlox_settings.selected_value = 8;
    }
    if keyboard_input.just_pressed(KeyCode::Digit9) {
        //vlox_settings.selected_value = 9;
    }
    if keyboard_input.just_pressed(KeyCode::Digit0) {
        //vlox_settings.selected_value = 0;
    }
}

#[derive(Resource, Default)]
struct VloxSettings {
    selected_value: vlox::MaterialId,
    selected_depth: u8,
    data: vlox::VloxData,
    materials: vlox::MaterialMap,
}

#[derive(Component)]
struct MainMesh;

fn set_vlox_mesh(
    mesh: &mut Mesh,
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
) {
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Float32x3(normals));
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Float32x4(colors));
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_aabb();
}
