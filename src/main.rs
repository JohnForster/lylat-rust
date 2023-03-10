use bevy::{pbr::CubemapVisibleEntities, prelude::*, render::primitives::CubemapFrusta};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct Arwing;

#[derive(Component)]
struct Drone;

#[derive(Component, Default)]
struct Laser;

#[derive(Component, Default)]
struct Destructable;

#[derive(Bundle, Default)]
struct LaserBundle {
    laser: Laser,
    destructable: Destructable,
    scene: Handle<Scene>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    point_light: PointLight,
    cubemap_visible_entities: CubemapVisibleEntities,
    cubemap_frusta: CubemapFrusta,
}

const SCALE: f32 = 0.4;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(rotate_arwing)
        .add_system(rotation_to_movement)
        .add_system(normalize_rotation)
        .add_system(fire_laser)
        .add_system(move_laser)
        .add_system(rotate_drone)
        .add_system(laser_destroy)
        .run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_camera(&mut commands);
    spawn_light(&mut commands);
    spawn_drones(&mut commands, &asset_server);
    spawn_arwing(&mut commands, &asset_server);
}

fn spawn_camera(commands: &mut Commands) {
    let transform =
        Transform::from_xyz(0.0, 1.0, -5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    commands.spawn(Camera3dBundle {
        transform,
        ..default()
    });
}

fn spawn_light(commands: &mut Commands) {
    const HALF_SIZE: f32 = 1.0;
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        ..default()
    },));
}

fn spawn_drones(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let locations = vec![
        Vec3::from((0.0, 0.0, 10.0)),
        Vec3::from((1.0, 1.0, 10.0)),
        Vec3::from((-1.0, 1.0, 10.0)),
    ];
    locations
        .into_iter()
        .for_each(|location| spawn_drone(commands, asset_server, location));
}

fn spawn_drone(commands: &mut Commands, asset_server: &Res<AssetServer>, location: Vec3) {
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/drone.glb#Scene0"),
                transform: Transform {
                    scale: Vec3::from((0.4, 0.4, 0.4)),
                    translation: location,
                    rotation: Quat::from_rotation_z(0.5),
                    ..default()
                },
                ..default()
            },
            Drone,
            Destructable,
            Collider::cuboid(2.0 * SCALE, 1.25 * SCALE, 2.0 * SCALE),
            Sensor,
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::STATIC_STATIC);
}

fn spawn_arwing(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/arwing.glb#Scene0"),
            transform: Transform {
                scale: Vec3::from((0.4, 0.4, 0.4)),
                ..default()
            },
            ..default()
        },
        Arwing,
    ));
}

fn rotate_arwing(
    time: Res<Time>,
    mut q_arwing: Query<&mut Transform, With<Arwing>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    const ROTATION_SPEED: f32 = 1.5;
    const MAX_ROT_X: f32 = 0.4;
    const MAX_ROT_Z: f32 = 0.7;
    for mut transform in &mut q_arwing {
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            let new_rot_x = transform.rotation.x + time.delta_seconds() * ROTATION_SPEED;
            transform.rotation.x = new_rot_x.clamp(-MAX_ROT_X, MAX_ROT_X)
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            let new_rot_x = transform.rotation.x - time.delta_seconds() * ROTATION_SPEED;
            transform.rotation.x = new_rot_x.clamp(-MAX_ROT_X, MAX_ROT_X)
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            let new_rot_z = transform.rotation.z + time.delta_seconds() * ROTATION_SPEED;
            transform.rotation.z = new_rot_z.clamp(-MAX_ROT_Z, MAX_ROT_Z)
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            let new_rot_z = transform.rotation.z - time.delta_seconds() * ROTATION_SPEED;
            transform.rotation.z = new_rot_z.clamp(-MAX_ROT_Z, MAX_ROT_Z)
        }
    }
}

fn rotation_to_movement(time: Res<Time>, mut q_arwing: Query<&mut Transform, With<Arwing>>) {
    const SPEED: f32 = 5.;
    const MAX_TOP: f32 = 0.7;
    const MAX_BOTTOM: f32 = -1.5;
    const MAX_LEFT: f32 = -1.7;
    const MAX_RIGHT: f32 = 1.7;
    for mut transform in &mut q_arwing {
        let new_x = transform.translation.x - transform.rotation.z * time.delta_seconds() * SPEED;
        let new_y = transform.translation.y - transform.rotation.x * time.delta_seconds() * SPEED;
        transform.translation.x = new_x.clamp(MAX_LEFT, MAX_RIGHT);
        transform.translation.y = new_y.clamp(MAX_BOTTOM, MAX_TOP);
    }
}

fn normalize_rotation(time: Res<Time>, mut q_arwing: Query<&mut Transform, With<Arwing>>) {
    const NORMALIZE_FACTOR: f32 = 0.2;
    let normalize_factor = NORMALIZE_FACTOR.powf(time.delta_seconds());
    for mut transform in &mut q_arwing {
        transform.rotation.x *= normalize_factor;
        transform.rotation.y *= normalize_factor;
        transform.rotation.z *= normalize_factor;
    }
}

fn fire_laser(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_arwing: Query<&Transform, With<Arwing>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for transform in &q_arwing {
            commands
                .spawn((
                    LaserBundle {
                        scene: asset_server.load("models/blaster_green.glb#Scene0"),
                        transform: Transform {
                            translation: transform.translation,
                            rotation: transform.rotation,
                            scale: Vec3::from((0.4, 0.4, 0.4)),
                            ..default()
                        },
                        ..default()
                    },
                    Collider::cuboid(0.22 * SCALE, 0.22 * SCALE, 1.25 * SCALE),
                    Sensor,
                ))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveCollisionTypes::STATIC_STATIC);
        }
    }
}

fn move_laser(
    time: Res<Time>,
    mut q_laser: Query<(&mut Transform, Entity), With<Laser>>,
    mut commands: Commands,
) {
    const LASER_SPEED: f32 = 10.0;
    const MAX_DISTANCE: f32 = 50.0;
    for (mut transform, entity) in &mut q_laser {
        let movement_vector = transform.rotation * Vec3::Z;
        transform.translation += movement_vector * time.delta_seconds() * LASER_SPEED;
        if transform.translation.z > MAX_DISTANCE {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn rotate_drone(time: Res<Time>, mut q_drone: Query<&mut Transform, With<Drone>>) {
    for mut transform in &mut q_drone {
        const ROTATION_SPEED: f32 = 2.0;
        transform.rotate_local_y(time.delta_seconds() * ROTATION_SPEED);
        transform.rotate_y(time.delta_seconds() * ROTATION_SPEED);
    }
}

fn laser_destroy(
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    q_destructable: Query<&Destructable>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            if q_destructable.get(*e1).is_ok() {
                commands.entity(*e1).despawn_recursive();
            }
            if q_destructable.get(*e2).is_ok() {
                commands.entity(*e2).despawn_recursive();
            }
        }
    }
}
