use crate::error::Result;
use crate::{atlas, components as comps, controls, game_state, sprite, utils};

use allegro::*;
use allegro_font::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;
use rand::prelude::*;

fn decode_tile(vals: [i32; 4], x: i32, y: i32, z: i32) -> i32
{
	let offt = if z == 0 { 21 } else { 0 };
	offt + match vals
	{
		//~ [0, 0, 0, 0] => rng.gen_range(0..3),
		[1, 0, 0, 0] => 3,
		[0, 1, 0, 0] => 4,
		[0, 0, 1, 0] => 5,
		[0, 0, 0, 1] => 6,

		[1, 0, 0, 1] => 7,
		[0, 1, 1, 0] => 8,

		[1, 0, 1, 1] => 9,
		[1, 1, 0, 1] => 10,
		[0, 1, 1, 1] => 11,
		[1, 1, 1, 0] => 12,

		[2, 1, 1, 0] => 13,
		[1, 0, 2, 1] => 14,
		[0, 1, 1, 2] => 15,
		[1, 2, 0, 1] => 16,

		[1, 1, 0, 0] => 17,
		[1, 0, 1, 0] => 18,
		[0, 0, 1, 1] => 19,
		[0, 1, 0, 1] => 20,
		_ =>
		{
			let mut rng = StdRng::seed_from_u64((x + 10000 * y) as u64);
			rng.gen_range(0..3)
		}
	}
}

fn print_heightmap(heightmap: &[i32])
{
	let real_size = (heightmap.len() as f32).sqrt() as i32;
	for y in 0..=real_size
	{
		for x in 0..=real_size
		{
			if x == 0
			{
				if y == 0
				{
					print!("   ");
				}
				else
				{
					print!("{:>2} ", y - 1);
				}
			}
			else if y == 0
			{
				print!("{:>2} ", x - 1);
			}
			else
			{
				print!(
					"{:>2} ",
					heightmap[((x - 1) + (y - 1) * real_size) as usize]
				);
			}
		}
		println!();
	}
}

fn diamond_square(size: i32) -> Vec<i32>
{
	assert!(size >= 0);
	let real_size = 2i32.pow(size as u32) + 1;
	dbg!(real_size);

	let global_max_height = 8;

	let mut heightmap = vec![-1i32; (real_size * real_size) as usize];
	let mut rng = thread_rng();
	let seed: u64 = rng.gen();
	//~ let seed = 13207773306860755903;
	dbg!(seed);
	let mut rng = StdRng::seed_from_u64(seed);

	//~ for stage in 0..=2
	for stage in 0..=size
	{
		let num_cells = 2i32.pow(stage as u32);
		let spacing = (real_size - 1) / num_cells;
		//~ dbg!(stage);
		//~ dbg!(spacing);

		// Square
		for y_idx in 0..=num_cells
		{
			for x_idx in 0..=num_cells
			{
				let y = y_idx * spacing;
				let x = x_idx * spacing;
				if heightmap[(x + y * real_size) as usize] == -1
				{
					let mut min_height = 0;
					let mut max_height = global_max_height;
					let mut mean_height = 0.;
					let mut count = 0;

					//~ println!();

					// Check the diag corners
					for sy in [-1, 1]
					{
						for sx in [-1, 1]
						{
							let cx = x + sx * spacing;
							let cy = y + sy * spacing;
							if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
							{
								let val = heightmap[(cx + cy * real_size) as usize];
								if val >= 0
								{
									min_height = utils::max(min_height, val - spacing);
									max_height = utils::min(max_height, val + spacing);
								}
							}
						}
					}

					// Check the rect corners
					for [sx, sy] in [[-1, 0], [0, -1], [1, 0], [0, 1]]
					{
						let cx = x + sx * spacing;
						let cy = y + sy * spacing;
						if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
						{
							let val = heightmap[(cx + cy * real_size) as usize];
							if val >= 0
							{
								min_height = utils::max(min_height, val - spacing);
								max_height = utils::min(max_height, val + spacing);

								mean_height =
									(mean_height * count as f32 + val as f32) / (count + 1) as f32;
								count += 1;
							}
						}
					}

					if count > 0
					{
						// TODO: Check this jitter values.
						min_height = utils::max(min_height, mean_height as i32 - 2);
						max_height = utils::min(max_height, mean_height as i32 + 2);
					}

					//~ dbg!(stage, x, y, min_height, max_height);
					let new_val = rng.gen_range(min_height..=max_height);
					//~ dbg!(new_val);
					heightmap[(x + y * real_size) as usize] = new_val;
				}
			}
		}

		// Diamond
		for y_idx in 0..num_cells
		{
			for x_idx in 0..num_cells
			{
				let y = y_idx * spacing + spacing / 2;
				let x = x_idx * spacing + spacing / 2;
				if heightmap[(x + y * real_size) as usize] == -1
				{
					let mut min_height = 0;
					let mut max_height = global_max_height;
					let mut mean_height = 0.;
					let mut count = 0;
					//~ println!();
					// Check the diag corners
					for sy in [-1, 1]
					{
						for sx in [-1, 1]
						{
							let cx = x + sx * spacing / 2;
							let cy = y + sy * spacing / 2;
							if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
							{
								let val = heightmap[(cx + cy * real_size) as usize];
								if val >= 0
								{
									min_height = utils::max(min_height, val - spacing / 2);
									max_height = utils::min(max_height, val + spacing / 2);

									mean_height = (mean_height * count as f32 + val as f32)
										/ (count + 1) as f32;
									count += 1;
								}
							}
						}
					}

					// Check the rect corners
					for [sx, sy] in [[-1, 0], [0, -1], [1, 0], [0, 1]]
					{
						let cx = x + sx * spacing;
						let cy = y + sy * spacing;
						if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
						{
							let val = heightmap[(cx + cy * real_size) as usize];
							if val >= 0
							{
								min_height = utils::max(min_height, val - spacing);
								max_height = utils::min(max_height, val + spacing);
							}
						}
					} // 3, 3

					if count > 0
					{
						// TODO: Check this jitter values.
						min_height = utils::max(min_height, mean_height as i32 - 2);
						max_height = utils::min(max_height, mean_height as i32 + 2);
					}
					//~ dbg!(x, y, stage, min_height, max_height);
					let new_val = rng.gen_range(min_height..=max_height);
					//~ dbg!(new_val);
					heightmap[(x + y * real_size) as usize] = new_val;
				}
			}
		}
	}
	heightmap
}

fn smooth_heightmap(heightmap: &[i32]) -> Vec<i32>
{
	let real_size = (heightmap.len() as f32).sqrt() as i32;
	let mut res = vec![0; heightmap.len()];
	for y in 0..real_size
	{
		for x in 0..real_size
		{
			let mut mean_height = 0.;
			let mut count = 0;
			for sy in [-1, 1]
			{
				for sx in [-1, 1]
				{
					let cx = x + sx;
					let cy = y + sy;
					if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
					{
						let val = heightmap[(cx + cy * real_size) as usize];
						mean_height =
							(mean_height * count as f32 + val as f32) / (count + 1) as f32;
						count += 1;
					}
				}
			}
			res[(x + y * real_size) as usize] = mean_height as i32;
		}
	}
	res
}

fn lower_heightmap(heightmap: &[i32]) -> Vec<i32>
{
	let mut min_height = 1000;
	for v in heightmap
	{
		min_height = utils::min(*v, min_height);
	}
	let mut res = heightmap.to_vec();
	for v in &mut res
	{
		*v -= min_height;
	}
	res
}

fn world_to_screen(pos: Point3<f32>) -> Point2<f32>
{
	Point2::new(
		pos.x as f32 * 64. - pos.y as f32 * 64.,
		pos.x * 32. + pos.y * 32. - pos.z * 24.,
	)
}

fn spawn_player(pos: Point3<f32>, dir: f32, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: dir },
		comps::Velocity {
			vel: Vector3::zeros(),
			dir_vel: 0.,
		},
		comps::FixedEngine { power: 2. },
		comps::Drawable {
			kind: comps::DrawableKind::Oriented {
				sprite: "data/plane.cfg".to_string(),
			},
		},
		comps::ParticleSpawners {
			spawners: vec![
				comps::ParticleSpawner {
					offset: Vector3::new(-0.3, 0.2, -0.4),
					kind: comps::ParticleKind::Stationary,
					spawn_delay: 0.15,
					time_to_spawn: 0.,
					duration: 1.,
					sprite: "data/engine_particles.cfg".to_string(),
				},
				comps::ParticleSpawner {
					offset: Vector3::new(-0.3, -0.2, -0.4),
					kind: comps::ParticleKind::Stationary,
					spawn_delay: 0.15,
					time_to_spawn: 0.,
					duration: 1.,
					sprite: "data/engine_particles.cfg".to_string(),
				},
			],
		},
		comps::CastsShadow,
		comps::ExplodeOnCollision,
		comps::OnDeathEffects {
			effects: vec![
				comps::OnDeathEffect::SplashWater,
				comps::OnDeathEffect::Explosion {
					kind: comps::ExplosionKind::Explosion,
				},
			],
		},
		comps::WaterCollector {
			time_to_splash: 0.,
			time_to_drop: 0.,
			water_amount: 99,
		},
	))
}

fn spawn_particle(
	pos: Point3<f32>, vel: Vector3<f32>, sprite: String, creation_time: f64, duration: f64,
	world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Velocity {
			vel: vel,
			dir_vel: 0.,
		},
		comps::TimeToDie {
			time_to_die: creation_time + duration,
		},
		comps::Drawable {
			kind: comps::DrawableKind::Animated {
				sprite: sprite,
				start_time: creation_time,
				total_duration: duration,
				once: true,
			},
		},
	))
}

fn spawn_cloud(pos: Point3<f32>, dir: f32, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: dir },
		comps::Drawable {
			kind: comps::DrawableKind::Oriented {
				sprite: "data/plane.cfg".to_string(),
			},
		},
		comps::CastsShadow,
	))
}

fn spawn_mushroom(pos: Point3<f32>, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Drawable {
			kind: comps::DrawableKind::Mushroom {
				sprite: "data/mushroom.cfg".to_string(),
				variant: 0,
			},
		},
		comps::CastsShadow,
		comps::Mushroom {
			on_fire: false,
			health: 1.,
		},
	))
}

fn change_on_fire(mushroom: hecs::Entity, on_fire: bool, world: &mut hecs::World) -> Result<()>
{
	let mut change_component = false;
	if let Ok(mut mushroom) = world.get::<&mut comps::Mushroom>(mushroom)
	{
		let old_on_fire = mushroom.on_fire;
		mushroom.on_fire = on_fire;
		change_component = old_on_fire != mushroom.on_fire;
	}
	if change_component
	{
		if on_fire
		{
			world.insert_one(
				mushroom,
				comps::ParticleSpawners {
					spawners: vec![comps::ParticleSpawner {
						offset: Vector3::new(0., 0., 1.),
						kind: comps::ParticleKind::Fire,
						spawn_delay: 0.15,
						time_to_spawn: 0.,
						duration: 1.,
						sprite: "data/fire.cfg".to_string(),
					}],
				},
			)?;
		}
		else
		{
			world.remove_one::<comps::ParticleSpawners>(mushroom)?;
		}
	}
	Ok(())
}

fn spawn_splash(pos: Point3<f32>, creation_time: f64, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Drawable {
			kind: comps::DrawableKind::Animated {
				sprite: "data/splash.cfg".to_string(),
				start_time: creation_time,
				total_duration: 0.5,
				once: true,
			},
		},
		comps::TimeToDie {
			time_to_die: creation_time + 0.5,
		},
	))
}

fn spawn_water_blob(
	pos: Point3<f32>, vel: Vector3<f32>, creation_time: f64, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Velocity {
			vel: vel,
			dir_vel: 0.,
		},
		comps::AffectedByGravity,
		comps::AffectedByFriction,
		comps::Drawable {
			kind: comps::DrawableKind::Animated {
				sprite: "data/water_blob.cfg".to_string(),
				start_time: creation_time,
				total_duration: 0.5,
				once: false,
			},
		},
		comps::CastsShadow,
		comps::ExplodeOnCollision,
		comps::OnDeathEffects {
			effects: vec![
				comps::OnDeathEffect::SplashWater,
				comps::OnDeathEffect::Explosion {
					kind: comps::ExplosionKind::Splash,
				},
			],
		},
	))
}

fn spawn_explosion(pos: Point3<f32>, creation_time: f64, world: &mut hecs::World) -> hecs::Entity
{
	let explosion = world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Drawable {
			kind: comps::DrawableKind::Animated {
				sprite: "data/explosion.cfg".to_string(),
				start_time: creation_time,
				total_duration: 0.5,
				once: true,
			},
		},
		comps::TimeToDie {
			time_to_die: creation_time + 0.5,
		},
	));

	let mut rng = thread_rng();
	for _ in 0..5
	{
		world.spawn((
			comps::Position { pos: pos, dir: 0. },
			comps::Velocity {
				vel: Vector3::new(
					rng.gen_range(-2.0..2.0),
					rng.gen_range(-2.0..2.0),
					rng.gen_range(3.0..5.0),
				),
				dir_vel: 0.,
			},
			comps::AffectedByGravity,
			comps::ParticleSpawners {
				spawners: vec![comps::ParticleSpawner {
					offset: Vector3::new(0., 0., 0.),
					kind: comps::ParticleKind::Stationary,
					spawn_delay: 0.1,
					time_to_spawn: 0.,
					duration: 1.,
					sprite: "data/engine_particles.cfg".to_string(),
				}],
			},
			comps::TimeToDie {
				time_to_die: creation_time + 1.5,
			},
		));
	}

	explosion
}

fn get_height(heightmap: &[i32], pos: Point2<f32>) -> Option<f32>
{
	let size = (heightmap.len() as f32).sqrt() as i32;
	let x = (pos.x + 0.5) as i32;
	let y = (pos.y + 0.5) as i32;
	let fx = 0.5 + pos.x - x as f32;
	let fy = 0.5 + pos.y - y as f32;

	if x >= 0 && y >= 0 && x + 1 < size && y + 1 < size
	{
		let h00 = heightmap[((x + 0) + (y + 0) * size) as usize] as f32;
		let h01 = heightmap[((x + 0) + (y + 1) * size) as usize] as f32;
		let h10 = heightmap[((x + 1) + (y + 0) * size) as usize] as f32;
		let h11 = heightmap[((x + 1) + (y + 1) * size) as usize] as f32;

		let h0 = (1. - fy) * h00 + fy * h01;
		let h1 = (1. - fy) * h10 + fy * h11;

		Some((1. - fx) * h0 + fx * h1)
	}
	else
	{
		None
	}
}

fn get_mushroom(mushrooms: &[Option<hecs::Entity>], pos: Point2<f32>) -> Option<hecs::Entity>
{
	let size = (mushrooms.len() as f32).sqrt() as i32;
	let x = (pos.x + 0.5) as i32;
	let y = (pos.y + 0.5) as i32;
	if x >= 0 && y >= 0 && x < size && y < size
	{
		mushrooms[(x + y * size) as usize]
	}
	else
	{
		None
	}
}

pub struct Map
{
	heightmap: Vec<i32>,
	mushrooms: Vec<Option<hecs::Entity>>,
	size: i32,
	display_width: f32,
	display_height: f32,
	camera_pos: Point3<f32>,
	world: hecs::World,
	player: hecs::Entity,

	up_state: bool,
	down_state: bool,
	left_state: bool,
	right_state: bool,
	drop_state: bool,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		let size = 5;
		let mut world = hecs::World::default();

		let player_pos = Point3::new(0., 2., 10.);
		let player = spawn_player(player_pos, 0., &mut world);

		for y in 5..10
		{
			for x in 5..10
			{
				spawn_cloud(
					Point3::new(x as f32, y as f32, x as f32 + 5.),
					x as f32 + y as f32 * 0.1,
					&mut world,
				);
			}
		}

		state.cache_sprite("data/terrain.cfg")?;
		state.cache_sprite("data/plane.cfg")?;
		state.cache_sprite("data/engine_particles.cfg")?;
		state.cache_sprite("data/explosion.cfg")?;
		state.cache_sprite("data/splash.cfg")?;
		state.cache_sprite("data/water_blob.cfg")?;
		state.cache_sprite("data/mushroom.cfg")?;
		state.cache_sprite("data/fire.cfg")?;
		//~ state.atlas.dump_pages();

		let heightmap = lower_heightmap(&smooth_heightmap(&diamond_square(size)));
		let mut mushrooms = Vec::with_capacity(heightmap.len());

		let real_size = 2i32.pow(size as u32) + 1;
		let mut rng = thread_rng();
		for y in 0..real_size - 1
		{
			for x in 0..real_size - 1
			{
				let h = get_height(&heightmap, Point2::new(x as f32, y as f32)).unwrap();
				mushrooms.push(
					if h > 1. && rng.gen_bool(0.3)
					{
						let mushroom =
							spawn_mushroom(Point3::new(x as f32, y as f32, h as f32), &mut world);
						if rng.gen_bool(0.5)
						{
							change_on_fire(mushroom, true, &mut world)?;
						}
						Some(mushroom)
					}
					else
					{
						None
					},
				);
			}
		}
		print_heightmap(&heightmap);

		Ok(Self {
			heightmap: heightmap,
			mushrooms: mushrooms,
			size: real_size,
			display_width: display_width,
			display_height: display_height,
			camera_pos: player_pos,
			world: world,
			player: player,
			up_state: false,
			down_state: false,
			left_state: false,
			right_state: false,
			drop_state: false,
		})
	}

	pub fn logic(
		&mut self, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		let mut to_die = vec![];

		// Player input.
		let mut spawn_water = None;
		let mut rng = thread_rng();
		if let Ok((pos, mut vel, mut water_col)) = self.world.query_one_mut::<(
			&comps::Position,
			&mut comps::Velocity,
			&mut comps::WaterCollector,
		)>(self.player)
		{
			let left_right = self.left_state as i32 - (self.right_state as i32);
			let up_down = self.up_state as i32 - (self.down_state as i32);

			vel.dir_vel = -left_right as f32 * 1.;
			let max_vert_speed = 3.;
			let desired_vel = up_down as f32 * max_vert_speed;
			let f = utils::clamp(water_col.water_amount as f32 / 50., 0., 1.);
			let accel = f * 1. + (1. - f) * 5.;
			if vel.vel.z > desired_vel
			{
				vel.vel.z -= accel * utils::DT;
			}
			else if vel.vel.z < desired_vel
			{
				vel.vel.z += accel * utils::DT;
			}
			let z_speed = vel.vel.z.abs();
			if z_speed > max_vert_speed
			{
				vel.vel.z = max_vert_speed.copysign(vel.vel.z);
			}

			if self.drop_state
			{
				if state.time() > water_col.time_to_drop && water_col.water_amount > 0
				{
					water_col.time_to_drop = state.time() + 0.4;
					water_col.water_amount -= 1;
					spawn_water = Some((
						pos.pos + Vector3::new(0., 0., -1.),
						vel.vel
							+ Vector3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.),
					));
				}
			}
		}
		if let Some((pos, vel)) = spawn_water
		{
			spawn_water_blob(pos, vel, state.time(), &mut self.world);
		}

		// Camera.
		if let Ok(pos) = self.world.get::<&comps::Position>(self.player)
		{
			self.camera_pos = pos.pos;
		}

		// Fixed engine.
		for (_, (pos, eng, vel)) in
			self.world
				.query_mut::<(&comps::Position, &comps::FixedEngine, &mut comps::Velocity)>()
		{
			let dir_vel = Rotation2::new(pos.dir) * Vector2::new(1., 0.);
			// Thinner air -> faster speed.
			let f = utils::clamp(pos.pos.z / 20., 0., 1.);
			let height_adj = f * 1.5 + (1. - f) * 1.;

			let horiz_vel = height_adj * eng.power * Vector2::new(dir_vel.x, dir_vel.y);
			vel.vel.x = horiz_vel.x;
			vel.vel.y = horiz_vel.y;
		}

		// Collision.
		for (id, (pos, _)) in self
			.world
			.query_mut::<(&comps::Position, &comps::ExplodeOnCollision)>()
		{
			let mut do_explode = false;
			let mushroom_height = get_mushroom(&self.mushrooms, pos.pos.xy())
				.and_then(|_| Some(2.))
				.unwrap_or(0.);
			if let Some(h) = get_height(&self.heightmap, pos.pos.xy())
			{
				let h = h + mushroom_height;
				if pos.pos.z - h < 0.5
				{
					do_explode = true;
				}
			}
			else
			{
				do_explode = true;
			}
			if do_explode
			{
				to_die.push(id);
			}
		}

		// Gravity.
		for (_, (vel, _)) in self
			.world
			.query_mut::<(&mut comps::Velocity, &comps::AffectedByGravity)>()
		{
			vel.vel.z -= utils::DT * 5.;
		}

		// Friction.
		for (_, (vel, _)) in self
			.world
			.query_mut::<(&mut comps::Velocity, &comps::AffectedByGravity)>()
		{
			let friction = vel.vel.xy().normalize();
			let friction = 0.5 * friction * vel.vel.xy().norm_squared();
			vel.vel.x -= utils::DT * friction.x;
			vel.vel.y -= utils::DT * friction.y;
		}

		// Velocity.
		for (_, (pos, vel)) in self
			.world
			.query_mut::<(&mut comps::Position, &comps::Velocity)>()
		{
			pos.pos += utils::DT * vel.vel;
			pos.pos.z = utils::clamp(pos.pos.z, 0., 15.);
			pos.dir += utils::DT * vel.dir_vel;
		}

		// Water collection.
		let mut add_splash = vec![];
		for (_, (pos, water_col)) in self
			.world
			.query_mut::<(&comps::Position, &mut comps::WaterCollector)>()
		{
			if let Some(h) = get_height(&self.heightmap, pos.pos.xy())
			{
				if h < 0.1 && pos.pos.z - h < 2. && state.time() > water_col.time_to_splash
				{
					water_col.time_to_splash = state.time() + 0.25;
					water_col.water_amount += 5;
					add_splash.push(Point3::new(pos.pos.x, pos.pos.y, 0.01));
				}
			}
		}
		for pos in add_splash
		{
			spawn_splash(pos, state.time(), &mut self.world);
		}

		// Particle spawners.
		let mut to_spawn = vec![];
		for (_, (pos, spawners)) in self
			.world
			.query_mut::<(&comps::Position, &mut comps::ParticleSpawners)>()
		{
			for mut spawner in &mut spawners.spawners
			{
				if state.time() > spawner.time_to_spawn
				{
					let offset_xy = Rotation2::new(pos.dir) * spawner.offset.xy();
					let offset = Vector3::new(offset_xy.x, offset_xy.y, spawner.offset.z);

					let vel = match spawner.kind
					{
						comps::ParticleKind::Stationary => Vector3::zeros(),
						comps::ParticleKind::Fire =>
						{
							Vector3::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5), 5.)
						}
					};
					to_spawn.push((
						pos.pos + offset,
						vel,
						spawner.sprite.clone(),
						spawner.duration,
					));
					spawner.time_to_spawn = state.time() + spawner.spawn_delay;
				}
			}
		}
		for (pos, vel, sprite, duration) in to_spawn
		{
			spawn_particle(pos, vel, sprite, state.time(), duration, &mut self.world);
		}

		// Time to die
		for (id, time_to_die) in self.world.query_mut::<&comps::TimeToDie>()
		{
			if state.time() > time_to_die.time_to_die
			{
				to_die.push(id);
			}
		}

		// On death effects
		let mut explosions = vec![];
		let mut extinguish = vec![];
		for id in &to_die
		{
			if let Ok((pos, on_death_effects)) = self
				.world
				.query_one_mut::<(&comps::Position, &comps::OnDeathEffects)>(*id)
			{
				for effect in &on_death_effects.effects
				{
					match effect
					{
						comps::OnDeathEffect::Explosion { kind } =>
						{
							explosions.push((pos.pos, *kind))
						}
						comps::OnDeathEffect::SplashWater =>
						{
							if let Some(mushroom) = get_mushroom(&self.mushrooms, pos.pos.xy())
							{
								extinguish.push(mushroom);
							}
						}
					}
				}
			}
		}

		// Explosions
		for (pos, kind) in explosions
		{
			match kind
			{
				comps::ExplosionKind::Explosion =>
				{
					spawn_explosion(pos, state.time(), &mut self.world);
				}
				comps::ExplosionKind::Splash =>
				{
					spawn_splash(pos, state.time(), &mut self.world);
				}
			}
		}

		// Extinguish
		for mushroom in extinguish
		{
			change_on_fire(mushroom, false, &mut self.world)?;
		}

		// Remove dead entities
		to_die.sort();
		to_die.dedup();
		for id in to_die
		{
			//~ dbg!("died", id);
			self.world.despawn(id)?;
		}

		Ok(None)
	}

	pub fn input(
		&mut self, event: &Event, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		if let Some((down, action)) = state.options.controls.decode_event(event)
		{
			match action
			{
				controls::Action::MoveForward =>
				{
					self.up_state = down;
				}
				controls::Action::MoveBackward =>
				{
					self.down_state = down;
				}
				controls::Action::TurnLeft =>
				{
					self.left_state = down;
				}
				controls::Action::TurnRight =>
				{
					self.right_state = down;
				}
				controls::Action::DropWater =>
				{
					self.drop_state = down;
				}
			}
		}
		Ok(None)
	}

	pub fn draw(&mut self, state: &game_state::GameState) -> Result<()>
	{
		state.core.clear_to_color(Color::from_rgb_f(0., 0., 0.));

		let camera_xy = world_to_screen(self.camera_pos);

		// Map drawing
		state.core.hold_bitmap_drawing(true);
		let dx = self.display_width / 2. - camera_xy.x;
		let dy = self.display_height / 2. - camera_xy.y;
		let tiles = state.get_sprite("data/terrain.cfg").unwrap();
		for y in 0..self.size - 1
		{
			for x in 0..self.size - 1
			{
				let mut min_val = 1000;
				let mut vals = [0; 4];
				let mut idx = 0;
				for sy in [0, 1]
				{
					for sx in [0, 1]
					{
						let z = self.heightmap[((x + sx) + (y + sy) * self.size) as usize];
						min_val = utils::min(min_val, z);
						vals[idx] = z;
						idx += 1;
					}
				}
				for v in &mut vals
				{
					*v -= min_val;
				}

				let variant = decode_tile(vals, x, y, min_val);
				let xy = world_to_screen(Point3::new(x as f32, y as f32, min_val as f32));
				tiles.draw(
					utils::round_point(xy - utils::Vec2D::new(64. - dx, 96. - dy)),
					variant,
					Color::from_rgb_f(1., 1., 1.),
					state,
				);
			}
		}
		state.core.hold_bitmap_drawing(false);

		// Shadows
		for (_, (pos, _)) in self
			.world
			.query::<(&comps::Position, &comps::CastsShadow)>()
			.iter()
		{
			if let Some(h) = get_height(&self.heightmap, pos.pos.xy())
			{
				let xy = world_to_screen(Point3::new(pos.pos.x, pos.pos.y, h));
				let xy = utils::round_point(xy + Vector2::new(dx, dy));

				state.prim.draw_filled_ellipse(
					xy.x,
					xy.y,
					16.,
					8.,
					Color::from_rgba_f(0., 0., 0., 0.4),
				);
			}
		}

		// Sprites
		let mut pos_and_sprite = vec![];
		for (_, (pos, drawable)) in self
			.world
			.query::<(&comps::Position, &comps::Drawable)>()
			.iter()
		{
			let xy = world_to_screen(pos.pos);

			let (sprite, variant) = match &drawable.kind
			{
				comps::DrawableKind::Oriented { sprite } =>
				{
					let num_orientations = 8;
					let window_size = 2. * f32::pi() / num_orientations as f32;

					let variant = (num_orientations
						- (((pos.dir.rem_euclid(2. * f32::pi()) + f32::pi() + window_size / 2.)
							/ window_size) as i32 + num_orientations / 4)
							% num_orientations) % num_orientations;
					(sprite.clone(), variant)
				}
				comps::DrawableKind::Mushroom { sprite, variant } => (sprite.clone(), *variant),
				comps::DrawableKind::Animated {
					sprite,
					start_time,
					total_duration,
					once,
				} =>
				{
					let num_variants = state.get_sprite(&sprite).unwrap().num_variants();
					let variant =
						(num_variants as f64 * (state.time() - start_time) / total_duration) as i32;
					let variant = if *once
					{
						utils::clamp(variant, 0, num_variants - 1)
					}
					else
					{
						variant % num_variants
					};
					(sprite.clone(), variant)
				}
			};

			pos_and_sprite.push((pos.pos, xy, sprite, variant));
		}
		pos_and_sprite.sort_by(|(pos1, _, _, _), (pos2, _, _, _)| {
			let yz1 = [pos1.z, pos1.y];
			let yz2 = [pos2.z, pos2.y];

			yz1.partial_cmp(&yz2).unwrap()
		});
		state.core.hold_bitmap_drawing(true);
		for (_, xy, sprite, variant) in pos_and_sprite
		{
			let sprite = state.get_sprite(&sprite).unwrap();
			sprite.draw(
				utils::round_point(xy + Vector2::new(dx, dy)),
				variant,
				Color::from_rgb_f(1., 1., 1.),
				state,
			);
		}
		state.core.hold_bitmap_drawing(false);

		// UI
		if let Ok(water_col) = self.world.get::<&comps::WaterCollector>(self.player)
		{
			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.4, 0.4, 0.8),
				48.,
				24.,
				FontAlign::Centre,
				"WATER",
			);

			state.core.draw_text(
				&state.number_font,
				Color::from_rgb_f(0.4, 0.8, 0.4),
				96.,
				24.,
				FontAlign::Centre,
				&format!("{:0>2}", water_col.water_amount),
			);
		}

		Ok(())
	}
}
