use anyhow::{bail, Result};
use bevy::prelude::*;
use itertools::Itertools;
use rand::prelude::*;

const ANT_SPEED: f32 = 20.;
const BRICK_SIZE: f32 = 20.;

#[derive(Default)]
pub struct AntPlugin<A, G> {
    pub app_state: A,
    pub grid_state: G,
    pub pattern: String,
    pub ant_pattern: AntPattern,
}

impl<A: States, G: States> Plugin for AntPlugin<A, G> {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.ant_pattern.clone())
            .add_systems(Startup, setup.run_if(in_state(self.grid_state.clone())))
            .add_systems(
                FixedUpdate,
                (ant_movement, bricks_rotation)
                    .chain()
                    .run_if(in_state(self.app_state.clone()))
                    .run_if(in_state(self.grid_state.clone())),
            );
    }
}

#[derive(Clone, Copy, Component)]
enum PatternDirection {
    R1,
    R2,
    L1,
    L2,
    Uturn,
    None,
}

#[derive(Resource, Clone, Default)]
pub struct AntPattern {
    colors: Vec<Color>,
    turns: Vec<PatternDirection>,
}

impl AntPattern {
    pub fn parse(pattern: String) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let mut colors = Vec::new();
        let mut turns = Vec::new();

        for p in pattern.to_lowercase().chars() {
            let color = Color::srgb(
                rng.gen_range(0.2..0.8),
                rng.gen_range(0.2..0.8),
                rng.gen_range(0.2..0.8),
            );
            match p {
                'r' => {
                    colors.push(color);
                    turns.push(PatternDirection::R1);
                }
                'l' => {
                    colors.push(color);
                    turns.push(PatternDirection::L1);
                }
                'u' => {
                    colors.push(color);
                    turns.push(PatternDirection::Uturn);
                }
                'n' => {
                    colors.push(color);
                    turns.push(PatternDirection::None);
                }
                _ => (),
            }
        }

        if colors.len() < 2 {
            bail!("incorrect pattern: should be at least 2 correct values (L, R, U, N)");
        }

        Ok(Self { colors, turns })
    }

    fn first(&self) -> (Color, PatternDirection) {
        return (*self.colors.get(1).unwrap(), *self.turns.first().unwrap());
    }

    fn next(&self, current: Color) -> (Color, PatternDirection) {
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

#[derive(Component)]
enum AntDirection {
    Up,
    Down,
    Left,
    Right,
}

impl AntDirection {
    fn rotate_left(&mut self) {
        match *self {
            Self::Up => *self = Self::Left,
            Self::Left => *self = Self::Down,
            Self::Down => *self = Self::Right,
            Self::Right => *self = Self::Up,
        };
    }

    fn rotate_right(&mut self) {
        match *self {
            Self::Up => *self = Self::Right,
            Self::Left => *self = Self::Up,
            Self::Down => *self = Self::Left,
            Self::Right => *self = Self::Down,
        };
    }

    fn rotate_back(&mut self) {
        match *self {
            Self::Up => *self = Self::Down,
            Self::Left => *self = Self::Right,
            Self::Down => *self = Self::Up,
            Self::Right => *self = Self::Left,
        };
    }
}

#[derive(Component)]
struct Brick;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("ant.png"),
            ..default()
        },
        AntDirection::Up,
        PatternDirection::None,
    ));
}

fn ant_movement(mut ant_query: Query<(&mut AntDirection, &mut Transform, &PatternDirection)>) {
    let (mut ant_direction, mut ant_transform, pattern_direction) = ant_query.single_mut();

    match pattern_direction {
        PatternDirection::R1 => {
            ant_transform.rotate_z(f32::to_radians(-90.0));
            ant_direction.rotate_right();
        }
        PatternDirection::L1 => {
            ant_transform.rotate_z(f32::to_radians(90.0));
            ant_direction.rotate_left();
        }
        PatternDirection::Uturn => {
            ant_transform.rotate_z(f32::to_radians(180.0));
            ant_direction.rotate_back();
        }
        PatternDirection::None => (),
        _ => todo!(),
    }

    match *ant_direction {
        AntDirection::Up => ant_transform.translation.y += ANT_SPEED,
        AntDirection::Down => ant_transform.translation.y -= ANT_SPEED,
        AntDirection::Left => ant_transform.translation.x -= ANT_SPEED,
        AntDirection::Right => ant_transform.translation.x += ANT_SPEED,
    }
}

fn bricks_rotation(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    pattern: Res<AntPattern>,
    mut ant_query: Query<(&mut PatternDirection, &Transform)>,
    mut brick_query: Query<(&mut Sprite, &Transform), With<Brick>>,
) {
    let (mut pattern_direction, ant) = ant_query.single_mut();

    let mut flip_color = false;
    for (mut brick_sprite, brick) in brick_query.iter_mut() {
        if (brick.translation.x, brick.translation.y) == (ant.translation.x, ant.translation.y) {
            flip_color = true;
            (brick_sprite.color, *pattern_direction) = pattern.next(brick_sprite.color);
        }
    }

    if !flip_color {
        let (color, direction) = pattern.first();
        *pattern_direction = direction;

        // commands.spawn(MaterialMesh2dBundle {
        //     mesh: Mesh2dHandle(meshes.add(RegularPolygon::new(BRICK_SIZE / 2_f32.sqrt(), 6))),
        //     material: materials.add(color),
        //     transform: Transform::from_xyz(ant.translation.x, ant.translation.y, -1.),
        //     ..default()
        // });

        commands.spawn((
            SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform::from_xyz(ant.translation.x, ant.translation.y, -1.)
                    .with_scale(Vec3::new(BRICK_SIZE, BRICK_SIZE, 0.)),
                ..default()
            },
            Brick,
        ));
    }
}
