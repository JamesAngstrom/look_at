extern crate amethyst;

use amethyst::{
    assets::{Loader, AssetLoaderSystemData},
    ecs::prelude::*,
    core::transform::{Transform, TransformBundle},
    core::nalgebra::{Vector3},
    prelude::*,
    renderer::{Material, Shape, PosNormTex, DrawShaded, MaterialDefaults, Mesh, Rgba,
               Camera, Projection, AmbientColor, SkyboxColor, DirectionalLight, Light,
               DrawSkybox, Stage, Pipeline, RenderBundle},
    core::timing::{Time},
    utils::{application_root_dir},
};

use std::f32::consts::*;

pub struct Follow {
    pub entity: Entity
}

impl Component for Follow {
    type Storage = DenseVecStorage<Self>;
}

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        world.add_resource(AmbientColor(Rgba(0.15, 0.18, 0.35, 1.0)));

        let mesh = world.exec(|loader: AssetLoaderSystemData<Mesh>| {
            loader.load_from_data(
                Shape::Sphere(32, 32).generate::<Vec<PosNormTex>>(None),
                (),
            )
        });

        let mat = {
            let textures = &world.read_resource();
            let loader = world.read_resource::<Loader>();
            let mat_defaults = world.read_resource::<MaterialDefaults>();
            let albedo = loader.load_from_data([1.0, 0.0, 1.0, 0.0].into(), (), textures);
            Material {
                albedo,
                ..mat_defaults.0.clone()
            }
        };

        let mut trans = Transform::default();
        trans.set_scale(3.0, 3.0, 3.0);
        trans.set_position(Vector3::new(5.0, 30.0, 5.0));
        let sphere = world
            .create_entity()
            .with(mesh)
            .with(mat)
            .with(trans)
            .build();

        initialize_camera(world, sphere);
        initialize_lights(world);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(DrawSkybox::new())
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, None))?
        .with(FollowSystem::new(), "follow_system", &[]);
    let mut game = Application::new("", Example, game_data)?;
    game.run();
    Ok(())
}

fn initialize_camera(world: &mut World, target: Entity) {
    let mut transform = Transform::default();
    transform.set_position(Vector3::new(0.0, 10.0, 300.0));

    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.0, 60.0 * PI / 180.0)))
        .with(transform)
        .with(Follow { entity: target })
        .build();
}

fn initialize_lights(world: &mut World) {
    world.add_resource(AmbientColor(Rgba(0.15, 0.18, 0.35, 1.0)));
    {
        let mut skybox = world.write_resource::<SkyboxColor>();
        skybox.zenith = Rgba::green();
        skybox.nadir = Rgba::red();
    }

    let dir = Vector3::new(0.7, -1.0, 0.8).normalize();

    let light: Light = DirectionalLight {
        color: Rgba(0.4, 0.4, 0.5, 1.0),
        direction: [dir.x, dir.y, dir.z]
    }.into();

    let mut transform = Transform::default();
    transform.set_position(Vector3::new(5.0, 20.0, 15.0));

    world.create_entity().with(light).with(transform).build();

}

pub struct FollowSystem {
    target: Option<Entity>
}

impl FollowSystem {
    pub fn new() -> Self {
        FollowSystem { target: None }
    }
}

impl<'s> System<'s> for FollowSystem {
    type SystemData = (
        ReadStorage<'s, Follow>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (followers, cameras, mut transforms, time): Self::SystemData) {
        let point = match self.target {
            None => Vector3::new(0.0, 0.0, 0.0),
            Some(target) => {
                *transforms.get(target).unwrap().translation()
            }
        };

        for (follow, _camera, mut transform) in (&followers, &cameras, &mut transforms).join() {
            self.target = Some(follow.entity);

            const SPEED: f32 = 20.0;
            let dir = point - transform.translation();
            if dir.magnitude() > 35.0 {
                let translation = dir.normalize() * SPEED * time.delta_seconds();
                transform.translate_xyz(translation.x, translation.y, translation.z);
            }
            transform.move_left(SPEED / 2.0 * time.delta_seconds());
            transform.set_y(point.y + 15.0);

            transform.look_at(point, Vector3::new(0.0, 1.0, 0.0));
        }
    }
}
