use bevy::{prelude::*, render::camera::ScalingMode, tasks::IoTaskPool};
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::*;
use bevy_web_asset::WebAssetPlugin;
use bytemuck::{Pod, Zeroable};
use matchbox_socket::WebRtcSocket;

mod input;
use input::*;

mod components;
use components::*;

mod config;
use config::SIZE;

mod cursor;
use cursor::*;

#[derive(Resource)]
struct Session {
    socket: Option<WebRtcSocket>,
}

#[derive(Resource)]
struct LocalPlayerHandle(usize);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct MoveSpeed(f32);

#[derive(Default, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Packet {
    angle: Vec2,
    input: u32,
}

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = Packet;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

fn main() {
    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .with_rollback_schedule(
            Schedule::default().with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded()
                    .with_system(move_players)
                    .with_system(fire_bullets),
            ),
        )
        .register_rollback_component::<Transform>()
        .build(&mut app);

    app.add_plugin(WebAssetPlugin::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        // fill the entire browser window
                        fit_canvas_to_parent: true,
                        #[cfg(use_canvas)]
                        canvas: Some("#game_canvas".into()),
                        ..default()
                    },
                    ..default()
                })
                .disable::<AssetPlugin>(),
        )
        .add_plugin(PanCamPlugin::default())
        .add_plugin(CursorPlugin)
        .insert_resource(ClearColor(Color::rgb(0.23, 0.23, 0.23)))
        .insert_resource(MoveSpeed(0.15))
        .register_type::<MoveSpeed>()
        .add_startup_system(setup)
        .add_startup_system(start_matchbox_socket)
        .add_startup_system(spawn_player)
        .add_system(wait_for_players)
        .add_system(move_bullets);

    if !cfg!(debug_assertions) {
        app.add_plugin(DebugLinesPlugin::default())
            .add_plugin(WorldInspectorPlugin);
    }

    app.run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn(camera_bundle).insert(PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
        enabled: true,
        zoom_to_cursor: true,
        max_scale: Some(5.),
        min_scale: 0.1,
        max_x: Some(SIZE.0),
        min_x: Some(-SIZE.0),
        max_y: Some(SIZE.1),
        min_y: Some(-SIZE.1),
    });
}

fn spawn_player(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Player { handle: 0, ..default() },
        Rollback::new(rip.next_id()),
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 0., 0.)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            texture: asset_server.load("https://pb.teodorkallman.com/api/files/do5wjchjokp39xu/993uolh5bif5cxe/christmas_xiao_CsVpPbYYud.png"),
            ..default()
        },
    ));

    commands.spawn((
        Player { handle: 1, ..default() },
        Rollback::new(rip.next_id()),
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 3., 0.)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            texture: asset_server.load("https://pb.teodorkallman.com/api/files/do5wjchjokp39xu/diip395x5vth5yp/diluc_annoyed_face_OGBBgYbgHM.png"),
            ..default()
        },
    ));
}

fn fire_bullets(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    time: Res<Time>,
    mut commands: Commands,
    #[cfg(debug_assertions)] mut lines: ResMut<DebugLines>,
) {
    for (transform, mut player) in player_query.iter_mut() {
        let (input, _) = inputs[player.handle];
        player.reloading.tick(time.delta());

        if has_fired(input.input) && player.reloading.finished() {
            player.reloading.reset();
            let xy = transform.translation.truncate();
            #[cfg(debug_assertions)]
            lines.line(xy.extend(100.), input.angle.extend(100.), 1.);
            let direction = (input.angle - xy).normalize();
            commands.spawn((
                Bullet { direction },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2 { x: 0.2, y: 0.2 }),
                        ..default()
                    },
                    transform: Transform::from_translation(xy.extend(100.)),
                    ..default()
                },
            ));
        }
    }
}

fn move_bullets(
    mut bullet_query: Query<(Entity, &mut Transform, &Bullet)>,
    mut commands: Commands,
) {
    for (entity, mut transform, bullet) in bullet_query.iter_mut() {
        let move_delta = (bullet.direction * 0.5).extend(0.);

        transform.translation += move_delta;
        if transform.translation.distance(Vec2::ZERO.extend(100.)) > 20. {
            commands.entity(entity).despawn();
        }
    }
}

fn move_players(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    move_speed: Res<MoveSpeed>,
) {
    for (mut transform, mut player) in player_query.iter_mut() {
        let (input, _) = inputs[player.handle];

        let direction = direction(input.input);

        if direction == Vec2::ZERO {
            continue;
        }
        player.last_direction = direction;
        let move_delta = (direction * move_speed.0).extend(0.);

        transform.translation += move_delta;
        transform.translation = transform.translation.clamp(
            Vec3::new(-SIZE.0, -SIZE.1, -1000.),
            Vec3::new(SIZE.0, SIZE.1, 1000.),
        );
    }
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "wss://matchbox.teodorkallman.com/extreme_bevy?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    IoTaskPool::get().spawn(message_loop).detach();

    commands.insert_resource(Session {
        socket: Some(socket),
    });
}

fn wait_for_players(mut commands: Commands, mut session: ResMut<Session>) {
    let Some(socket) = &mut session.socket else {
        // If there is no socket we've already started the game
        return;
    };

    // Check for new connections
    socket.accept_new_connections();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");
    // TODO
    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerHandle(i));
        }
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = session.socket.take().unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2PSession(ggrs_session));
}
