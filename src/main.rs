use anyhow::{bail, Result};
use bevy::{
    log::LogPlugin,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    winit::WinitWindows,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_pancam::*;
use clap::Parser;
use itertools::Itertools;
use rand::prelude::*;
use winit::window::Icon;

const ANT_SPEED: f32 = 20.;
const TILE_SIZE: f32 = 20.;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct AntApp {
    /// Set custom render rate
    #[arg(short, long, default_value_t = 60)]
    rate: u8,
    /// Pattern to use
    #[arg(short, long, default_value = "RL")]
    pattern: String,
}

fn main() -> Result<()> {
    let ant_app = AntApp::parse();
    let pattern = Pattern::parse(ant_app.pattern)?;

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Langton's ant".to_owned(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            PanCamPlugin,
            EmbeddedAssetPlugin::default(),
        ))
        .init_state::<AppState>()
        .insert_resource(Time::<Fixed>::from_hz(ant_app.rate.into()))
        .insert_resource(pattern)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (set_window_icon, setup))
        .add_systems(Update, pause)
        .add_systems(
            FixedUpdate,
            run_rotation.run_if(in_state(AppState::Running)),
        )
        .run();

    Ok(())
}

#[derive(Clone, Copy)]
enum Turn {
    Right,
    Left,
}

#[derive(Resource)]
struct Pattern {
    colors: Vec<Color>,
    turns: Vec<Turn>,
}

impl Pattern {
    fn parse_pattern(&mut self, pattern: String) {
        let mut rng = rand::thread_rng();

        for p in pattern.to_lowercase().chars() {
            let color = Color::srgb(rng.gen_range(0.1..0.8), rng.gen_range(0.1..0.8), 0.);
            match p {
                'r' => {
                    self.colors.push(color);
                    self.turns.push(Turn::Right);
                }
                'l' => {
                    self.colors.push(color);
                    self.turns.push(Turn::Left);
                }
                _ => (),
            }
        }
    }

    fn parse(pattern: String) -> Result<Self> {
        let mut s = Pattern {
            colors: Vec::new(),
            turns: Vec::new(),
        };

        s.parse_pattern(pattern);
        if s.colors.len() < 2 {
            bail!("incorrect pattern: should be at least 2 correct values (L, R)");
        }

        Ok(s)
    }

    fn first(&self) -> (Color, Turn) {
        return (*self.colors.get(1).unwrap(), *self.turns.first().unwrap());
    }

    fn next(&self, current: Color) -> (Color, Turn) {
        for ((color, next_color), turn) in self
            .colors
            .iter()
            .circular_tuple_windows::<(&Color, &Color)>()
            .zip(self.turns.iter())
        {
            if *color == current {
                return (*next_color, *turn);
            }
        }
        panic!("can't find macthing color")
    }
}

enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Component)]
struct Ant(Direction);

#[derive(Component)]
struct Tile;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Paused,
    #[default]
    Running,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("ant.png"),
            ..default()
        },
        Ant(Direction::North),
    ));
}

fn run_rotation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pattern: Res<Pattern>,
    mut ant_query: Query<(&mut Ant, &mut Transform)>,
    mut tile_query: Query<(&Transform, &mut Handle<ColorMaterial>), Without<Ant>>,
) {
    let (mut ant, mut ant_transform) = ant_query.single_mut();

    let mut flip_color = false;
    let mut next_turn = Turn::Left;

    for (tile_transform, tile_color) in tile_query.iter_mut() {
        if (tile_transform.translation.x, tile_transform.translation.y)
            == (ant_transform.translation.x, ant_transform.translation.y)
        {
            flip_color = true;
            let current_color = materials.get_mut(tile_color.id()).unwrap();
            (current_color.color, next_turn) = pattern.next(current_color.color);
        }
    }

    if !flip_color {
        let (color, turn) = pattern.first();
        next_turn = turn;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE))),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    ant_transform.translation.x,
                    ant_transform.translation.y,
                    -1.,
                ),
                ..default()
            },
            Tile,
        ));
    }

    match next_turn {
        Turn::Left => {
            ant_transform.rotate_z(f32::to_radians(90.));
            match ant.0 {
                Direction::North => ant.0 = Direction::West,
                Direction::South => ant.0 = Direction::East,
                Direction::West => ant.0 = Direction::South,
                Direction::East => ant.0 = Direction::North,
            }
        }
        Turn::Right => {
            ant_transform.rotate_z(f32::to_radians(-90.));
            match ant.0 {
                Direction::North => ant.0 = Direction::East,
                Direction::South => ant.0 = Direction::West,
                Direction::West => ant.0 = Direction::North,
                Direction::East => ant.0 = Direction::South,
            }
        }
    }

    match ant.0 {
        Direction::North => ant_transform.translation.y += ANT_SPEED,
        Direction::South => ant_transform.translation.y -= ANT_SPEED,
        Direction::West => ant_transform.translation.x -= ANT_SPEED,
        Direction::East => ant_transform.translation.x += ANT_SPEED,
    }
}

fn pause(
    game_state: Res<State<AppState>>,
    mut next_game_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        match *game_state.get() {
            AppState::Paused => next_game_state.set(AppState::Running),
            AppState::Running => next_game_state.set(AppState::Paused),
        }
    }
}

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/ant.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
