use crate::error::Result;
use crate::{atlas, components as comps, controls, game_state, sprite, ui, utils};

use allegro::*;
use allegro_audio::*;
use allegro_font::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;
use rand::prelude::*;
use utils::ColorExt;

#[derive(Copy, Clone, PartialEq, Eq)]
enum UIState
{
	Regular,
	Victory,
	InMenu,
}

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

fn diamond_square<R: Rng>(size: i32, rng: &mut R) -> Vec<i32>
{
	assert!(size >= 0);
	let real_size = 2i32.pow(size as u32) + 1;
	dbg!(real_size);

	let global_max_height = 8;

	let mut heightmap = vec![-1i32; (real_size * real_size) as usize];

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
		comps::CastsShadow { size: 1 },
		comps::ExplodeOnCollision {
			out_of_bounds_ok: true,
		},
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
			water_amount: 20,
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

fn spawn_cloud(pos: Point3<f32>, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Velocity {
			vel: Vector3::new(0.1, 0.2, 0.),
			dir_vel: 0.,
		},
		comps::Drawable {
			kind: comps::DrawableKind::Fixed {
				sprite: "data/cloud.cfg".to_string(),
				variant: 0,
			},
		},
		comps::CastsShadow { size: 0 },
		comps::Cloud,
	))
}

fn spawn_mushroom(pos: Point3<f32>, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Drawable {
			kind: comps::DrawableKind::Fixed {
				sprite: "data/mushroom.cfg".to_string(),
				variant: 0,
			},
		},
		comps::CastsShadow { size: 1 },
		comps::Mushroom { on_fire: false },
	))
}

fn spawn_obelisk(pos: Point3<f32>, dest: Point3<f32>, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		comps::Position { pos: pos, dir: 0. },
		comps::Drawable {
			kind: comps::DrawableKind::Fixed {
				sprite: "data/obelisk.cfg".to_string(),
				variant: 0,
			},
		},
		comps::Obelisk { dest: dest },
	))
}

fn change_on_fire(mushroom: hecs::Entity, on_fire: bool, world: &mut hecs::World) -> Result<bool>
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
	Ok(change_component)
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
		comps::CastsShadow { size: 2 },
		comps::ExplodeOnCollision {
			out_of_bounds_ok: false,
		},
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
	time_to_spread_fire: f64,
	subscreens: Vec<ui::SubScreen>,
	ui_state: UIState,
	seed: u64,
	collision_alert: bool,
	time_to_play_alert: f64,
	obelisk_sound: SampleInstance,
	num_fires: i32,
	num_blobs: i32,
	num_extinguished: i32,

	up_state: bool,
	down_state: bool,
	left_state: bool,
	right_state: bool,
	drop_state: bool,
	minimap_state: bool,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32, seed: u64,
		restart_music: bool,
	) -> Result<Self>
	{
		state.hide_mouse = true;
		if state.options.play_music && restart_music
		{
			state.sfx.set_music_file("data/andreas_theme.xm");
			state.sfx.play_music()?;
		}

		let size = state.options.map_size;
		let real_size = 2i32.pow(size as u32) + 1;
		let mut rng = StdRng::seed_from_u64(seed);

		let mut world = hecs::World::default();

		let dir = rng.gen_range(0.0..2. * f32::pi());
		let radius = real_size as f32 / 2.;
		let player_pos = Point3::new(radius, radius, 0.)
			+ Vector3::new(radius * dir.cos(), radius * dir.sin(), 12.);
		let player = spawn_player(player_pos, f32::pi() + dir, &mut world);

		for _ in 0..size * size
		{
			spawn_cloud(
				Point3::new(
					rng.gen_range(0..real_size) as f32,
					rng.gen_range(0..real_size) as f32,
					rng.gen_range(10..15) as f32,
				),
				&mut world,
			);
		}

		state.cache_sprite("data/terrain.cfg")?;
		state.cache_sprite("data/plane.cfg")?;
		state.cache_sprite("data/engine_particles.cfg")?;
		state.cache_sprite("data/explosion.cfg")?;
		state.cache_sprite("data/splash.cfg")?;
		state.cache_sprite("data/water_blob.cfg")?;
		state.cache_sprite("data/mushroom.cfg")?;
		state.cache_sprite("data/fire.cfg")?;
		state.cache_sprite("data/cloud.cfg")?;
		state.cache_sprite("data/obelisk.cfg")?;
		state.cache_sprite("data/shadow.cfg")?;
		//~ state.atlas.dump_pages();

		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;
		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;

		state.sfx.cache_sample("data/explosion.ogg")?;
		state.sfx.cache_sample("data/fly_down.ogg")?;
		state.sfx.cache_sample("data/fly_up.ogg")?;
		state.sfx.cache_sample("data/near_teleport_cont.ogg")?;
		state.sfx.cache_sample("data/teleport.ogg")?;
		state.sfx.cache_sample("data/water_drop.ogg")?;
		state.sfx.cache_sample("data/water_splash.ogg")?;
		state.sfx.cache_sample("data/extinguish.ogg")?;

		let mut heightmap = lower_heightmap(&smooth_heightmap(&diamond_square(size, &mut rng)));

		loop
		{
			let num_water: i32 = heightmap
				.iter()
				.map(|&h| {
					if h == 0
					{
						1
					}
					else
					{
						0
					}
				})
				.sum();
			if num_water < ((real_size * real_size) as f32 * state.options.water_factor) as i32
			{
				for h in &mut heightmap
				{
					*h = utils::max(0, *h - 1);
				}
			}
			else
			{
				break;
			}
		}

		let mut mushroom_map = vec![(false, 0.); heightmap.len()];
		let mushroom_heightmap = diamond_square(size, &mut rng);
		let max_mushroom_height = mushroom_heightmap.iter().max().unwrap();

		let mut num_mushrooms = 0;
		for y in 1..real_size - 1
		{
			for x in 1..real_size - 1
			{
				let idx = (x + real_size * y) as usize;
				let h = get_height(&heightmap, Point2::new(x as f32, y as f32)).unwrap();
				let mh = mushroom_heightmap[idx];
				mushroom_map[idx] = (h > 0.5 && max_mushroom_height - mh < 2, h);
				if mushroom_map[idx].0
				{
					num_mushrooms += 1;
				}
			}
		}

		let target_num_mushrooms = ((real_size * real_size) as f32 * 0.2) as i32;
		dbg!(num_mushrooms, target_num_mushrooms);
		'done: for _ in 0..target_num_mushrooms - num_mushrooms
		{
			for _ in 0..50
			{
				let x = rng.gen_range(0..real_size);
				let y = rng.gen_range(0..real_size);
				if let Some(h) = get_height(&heightmap, Point2::new(x as f32, y as f32))
				{
					let idx = (x + real_size * y) as usize;
					if h > 0.5 && !mushroom_map[idx].0
					{
						mushroom_map[idx].0 = true;
						num_mushrooms += 1;
						if num_mushrooms >= target_num_mushrooms
						{
							break 'done;
						}
					}
				}
			}
		}

		let mut num_fires = 0;
		let target_num_fires = (state.options.fire_start_probability * num_mushrooms as f32) as i32;
		let mut visited_mushrooms = 0;

		let mut mushrooms = vec![None; mushroom_map.len()];
		for y in 0..real_size - 1
		{
			for x in 0..real_size - 1
			{
				let (has_mushroom, h) = mushroom_map[(x + real_size * y) as usize];
				mushrooms[(x + real_size * y) as usize] = if has_mushroom
				{
					let mushroom =
						spawn_mushroom(Point3::new(x as f32, y as f32, h as f32), &mut world);
					if rng.gen_bool((target_num_fires - num_fires) as f64 / (num_mushrooms - visited_mushrooms) as f64)
					{
						change_on_fire(mushroom, true, &mut world)?;
						num_fires += 1;
					}
					visited_mushrooms += 1;
					Some(mushroom)
				}
				else
				{
					None
				};
			}
		}

		let mut obelisk_locs = vec![];
		for _ in 0..((size - 3) as f32 * state.options.obelisk_factor) as i32
		{
			'placed: for _ in 0..50
			{
				let x = rng.gen_range(0..real_size - 1);
				let y = rng.gen_range(0..real_size - 1);
				let h = get_height(&heightmap, Point2::new(x as f32, y as f32)).unwrap();

				if h > 0.5
					&& !obelisk_locs.iter().any(|&e| e == (x, y))
					&& mushrooms[(x + y * real_size) as usize].is_none()
				{
					obelisk_locs.push((x, y));
					for _ in 0..50
					{
						let dx = rng.gen_range(2..real_size - 2);
						let dy = rng.gen_range(2..real_size - 2);
						let h2 = get_height(&heightmap, Point2::new(dx as f32, dy as f32)).unwrap();

						if !obelisk_locs.iter().any(|&e| e == (dx, dy))
						{
							spawn_obelisk(
								Point3::new(x as f32, y as f32, h),
								Point3::new(dx as f32, dy as f32, h2 + 6.),
								&mut world,
							);
							break 'placed;
						}
					}
				}
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
			minimap_state: false,
			time_to_spread_fire: state.time() + 5.,
			subscreens: vec![],
			ui_state: UIState::Regular,
			seed: seed,
			collision_alert: false,
			time_to_play_alert: state.time(),
			obelisk_sound: state
				.sfx
				.play_continuous_sound("data/near_teleport_cont.ogg", 0.)?,
			num_fires: 0,
			num_blobs: 0,
			num_extinguished: 0,
		})
	}

	pub fn logic(
		&mut self, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		if self.ui_state != UIState::Regular
		{
			return Ok(None);
		}
		let mut to_die = vec![];

		// Player input.
		let mut spawn_water = None;
		let mut rng = thread_rng();
		let mut player_pos = None;
		let mut player_vel = None;
		if let Ok((pos, mut vel, mut water_col)) = self.world.query_one_mut::<(
			&comps::Position,
			&mut comps::Velocity,
			&mut comps::WaterCollector,
		)>(self.player)
		{
			player_pos = Some(pos.pos);
			player_vel = Some(vel.vel);
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
					state.sfx.play_sound("data/water_drop.ogg")?;
					spawn_water = Some((
						pos.pos + Vector3::new(0., 0., -1.),
						vel.vel
							+ Vector3::new(rng.gen_range(-0.1..0.1), rng.gen_range(-0.1..0.1), 0.),
					));
				}
			}
		}
		if let Some((pos, vel)) = spawn_water
		{
			spawn_water_blob(pos, vel, state.time(), &mut self.world);
			self.num_blobs += 1;
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
		let mushroom_height = 2.;
		for (id, (pos, explode)) in self
			.world
			.query_mut::<(&comps::Position, &comps::ExplodeOnCollision)>()
		{
			let mut do_explode = false;
			let mushroom_height = get_mushroom(&self.mushrooms, pos.pos.xy())
				.and_then(|_| Some(mushroom_height))
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
				do_explode = !explode.out_of_bounds_ok;
			}
			if do_explode
			{
				to_die.push(id);
			}
		}

		// Collision alert.
		self.collision_alert = false;
		if let (Some(player_pos), Some(player_vel)) = (player_pos, player_vel)
		{
			let mut alert = false;
			for dt in [0.2, 0.4, 0.6, 0.8, 1.]
			{
				let test_pos = player_pos + dt * player_vel;

				let mushroom_height = get_mushroom(&self.mushrooms, test_pos.xy())
					.and_then(|_| Some(mushroom_height))
					.unwrap_or(0.);
				if let Some(h) = get_height(&self.heightmap, test_pos.xy())
				{
					let h = h + mushroom_height;
					if test_pos.z - h < 0.5
					{
						alert = true;
					}
				}
			}
			self.collision_alert = alert;
			if self.collision_alert && state.time() > self.time_to_play_alert
			{
				state.sfx.play_sound("data/alert.ogg")?;
				self.time_to_play_alert = state.time() + 3.;
			}
		}

		// Cloud.
		for (_, (pos, _)) in self
			.world
			.query_mut::<(&mut comps::Position, &comps::Cloud)>()
		{
			pos.pos.x = pos.pos.x.rem_euclid(self.size as f32);
			pos.pos.y = pos.pos.y.rem_euclid(self.size as f32);
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
			let norm = vel.vel.xy().norm();
			if norm > 0.
			{
				let friction = vel.vel.xy().normalize();
				let friction = 0.5 * friction * vel.vel.xy().norm_squared();
				vel.vel.x -= utils::DT * friction.x;
				vel.vel.y -= utils::DT * friction.y;
			}
		}

		// Obelisk.
		let mut teleport = None;
		let mut near_obelisk = false;
		if let Some(player_pos) = player_pos
		{
			for (_, (pos, obelisk)) in self
				.world
				.query_mut::<(&comps::Position, &comps::Obelisk)>()
			{
				let norm = (player_pos.xy() - pos.pos.xy()).norm();

				let effect_dist = 3.;
				if norm < effect_dist
				{
					let f = norm / effect_dist;
					state.swirl_amount = 0. * f + 5. * (1. - f);
					near_obelisk = true;
				}
				if norm < 1.
				{
					teleport = Some(obelisk.dest);
				}
			}
		}
		if !near_obelisk
		{
			state.swirl_amount = utils::max(state.swirl_amount - 12. * utils::DT, 0.);
		}
		self.obelisk_sound
			.set_gain(state.swirl_amount / 5.)
			.unwrap();
		if let Some(dest) = teleport
		{
			if let Ok(mut pos) = self.world.get::<&mut comps::Position>(self.player)
			{
				state.sfx.play_sound("data/teleport.ogg")?;
				pos.pos = dest;
			}
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

					if water_col.water_amount < 100
					{
						add_splash.push(Point3::new(pos.pos.x, pos.pos.y, 0.01));
						if let Some(player_pos) = player_pos
						{
							state.sfx.play_positional_sound(
								"data/water_splash.ogg",
								world_to_screen(pos.pos),
								world_to_screen(player_pos),
								1.,
							)?;
						}
					}
					water_col.water_amount = utils::min(water_col.water_amount, 99);
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

		// Fire counting.
		self.num_fires = 0;
		for (_, mushroom) in self.world.query_mut::<&comps::Mushroom>()
		{
			if mushroom.on_fire
			{
				self.num_fires += 1;
			}
		}

		if self.num_fires == 0
		{
			state.paused = true;
			state.swirl_amount = 0.;
			self.ui_state = UIState::Victory;
		}

		// Fire spread
		let mut ignite = vec![];
		if state.time() > self.time_to_spread_fire
		{
			for (_, (pos, mushroom)) in self
				.world
				.query_mut::<(&comps::Position, &comps::Mushroom)>()
			{
				if mushroom.on_fire && rng.gen_bool(state.options.fire_spread_probability as f64)
				{
					let idx = rng.gen_range(0..4);
					let [dx, dy] = [[-1., 0.], [1., 0.], [0., 1.], [0., -1.]][idx];
					if let Some(mushroom) =
						get_mushroom(&self.mushrooms, pos.pos.xy() + Vector2::new(dx, dy))
					{
						ignite.push(mushroom);
					}
				}
			}
			self.time_to_spread_fire = state.time() + 15.;
		}
		for mushroom in ignite
		{
			change_on_fire(mushroom, true, &mut self.world)?;
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
								extinguish.push((pos.pos, mushroom));
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
					if let Some(player_pos) = player_pos
					{
						state.sfx.play_positional_sound(
							"data/explosion.ogg",
							world_to_screen(pos),
							world_to_screen(player_pos),
							1.,
						)?;
					}
					spawn_explosion(pos, state.time(), &mut self.world);
				}
				comps::ExplosionKind::Splash =>
				{
					if let Some(player_pos) = player_pos
					{
						state.sfx.play_positional_sound(
							"data/water_splash.ogg",
							world_to_screen(pos),
							world_to_screen(player_pos),
							1.,
						)?;
					}
					spawn_splash(pos, state.time(), &mut self.world);
				}
			}
		}

		// Extinguish
		for (pos, mushroom) in extinguish
		{
			if change_on_fire(mushroom, false, &mut self.world)?
			{
				self.num_extinguished += 1;
				if let Some(player_pos) = player_pos
				{
					state.sfx.play_positional_sound(
						"data/extinguish.ogg",
						world_to_screen(pos),
						world_to_screen(player_pos),
						1.,
					)?;
				}
			}
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
		if self.ui_state == UIState::InMenu
		{
			if let Event::KeyDown {
				keycode: KeyCode::Escape,
				..
			} = event
			{
				state.sfx.play_sound("data/ui2.ogg").unwrap();
				self.subscreens.pop().unwrap();
			}
			if let Some(action) = self
				.subscreens
				.last_mut()
				.and_then(|s| s.input(state, event))
			{
				match action
				{
					ui::Action::ControlsMenu =>
					{
						self.subscreens
							.push(ui::SubScreen::ControlsMenu(ui::ControlsMenu::new(
								self.display_width,
								self.display_height,
								state,
							)));
					}
					ui::Action::OptionsMenu =>
					{
						self.subscreens
							.push(ui::SubScreen::OptionsMenu(ui::OptionsMenu::new(
								self.display_width,
								self.display_height,
								state,
							)));
					}
					ui::Action::MainMenu => return Ok(Some(game_state::NextScreen::Menu)),
					ui::Action::Back =>
					{
						self.subscreens.pop().unwrap();
					}
					_ => (),
				}
			}
			if self.subscreens.is_empty()
			{
				self.ui_state = UIState::Regular;
				state.paused = false;
				state.hide_mouse = true;
			}
		}
		else
		{
			if let Some((down, action)) = state.options.controls.decode_event(event)
			{
				match action
				{
					controls::Action::Ascend =>
					{
						if down
						{
							state.sfx.play_sound("data/fly_up.ogg")?;
						}
						self.up_state = down;
					}
					controls::Action::Descend =>
					{
						if down
						{
							state.sfx.play_sound("data/fly_down.ogg")?;
						}
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
					controls::Action::Restart =>
					{
						if down
						{
							return Ok(Some(game_state::NextScreen::Game {
								seed: self.seed,
								restart_music: false,
							}));
						}
					}
					controls::Action::Minimap =>
					{
						self.minimap_state = down;
					}
				}
			}

			match event
			{
				Event::KeyDown { keycode, .. } => match keycode
				{
					KeyCode::Escape =>
					{
						state.sfx.play_sound("data/ui2.ogg").unwrap();
						self.subscreens
							.push(ui::SubScreen::InGameMenu(ui::InGameMenu::new(
								self.display_width,
								self.display_height,
							)));
						self.ui_state = UIState::InMenu;
						state.paused = true;
						state.hide_mouse = false;
						state.swirl_amount = 0.;
					}
					_ => (),
				},
				_ => (),
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

		// Shadows
		for (_, (pos, shadow)) in self
			.world
			.query::<(&comps::Position, &comps::CastsShadow)>()
			.iter()
		{
			if let Some(h) = get_height(&self.heightmap, pos.pos.xy())
			{
				let xy = world_to_screen(Point3::new(pos.pos.x, pos.pos.y, h));
				let xy = utils::round_point(xy + Vector2::new(dx, dy));

				let sprite = "data/shadow.cfg";
				let sprite = state
					.get_sprite(&sprite)
					.expect(&format!("Could not find sprite: {}", sprite));
				sprite.draw(xy, shadow.size, Color::from_rgb_f(1., 1., 1.), state);
			}
		}

		// Sprites
		let mut pos_and_sprite = vec![];
		for (id, (pos, drawable)) in self
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

					let offt = if let Ok(vel) = self.world.get::<&comps::Velocity>(id)
					{
						if vel.vel.z > 0.1
						{
							num_orientations
						}
						else if vel.vel.z < -0.1
						{
							2 * num_orientations
						}
						else
						{
							0
						}
					}
					else
					{
						0
					};

					let variant = (num_orientations
						- (((pos.dir.rem_euclid(2. * f32::pi()) + f32::pi() + window_size / 2.)
							/ window_size) as i32 + num_orientations / 4)
							% num_orientations) % num_orientations;
					(sprite.clone(), offt + variant)
				}
				comps::DrawableKind::Fixed { sprite, variant } => (sprite.clone(), *variant),
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
		for (_, xy, sprite, variant) in pos_and_sprite
		{
			let sprite = state
				.get_sprite(&sprite)
				.expect(&format!("Could not find sprite: {}", sprite));
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
				FontAlign::Left,
				"WATER",
			);

			state.core.draw_text(
				&state.number_font,
				Color::from_rgb_f(0.4, 0.8, 0.4),
				256.,
				24.,
				FontAlign::Centre,
				&format!("{:0>2}", water_col.water_amount),
			);
		}

		state.core.draw_text(
			&state.ui_font,
			Color::from_rgb_f(0.8, 0.6, 0.4),
			self.display_width - 256.,
			24.,
			FontAlign::Left,
			"FIRES",
		);

		state.core.draw_text(
			&state.number_font,
			Color::from_rgb_f(0.4, 0.8, 0.8),
			self.display_width - 64.,
			24.,
			FontAlign::Centre,
			&format!("{:0>2}", self.num_fires),
		);

		if self.ui_state == UIState::Regular
		{
			if self.world.contains(self.player)
			{
				if self.collision_alert
				{
					state.core.draw_text(
						&state.ui_font,
						Color::from_rgb_f(0.9, 0.3, 0.3),
						self.display_width / 2.,
						self.display_height - 48.,
						FontAlign::Centre,
						"!! COLLISION IMMINENT !!",
					);
				}
			}
			else
			{
				state.prim.draw_filled_rectangle(
					0.,
					0.,
					self.display_width,
					self.display_height,
					Color::from_rgba_f(0., 0., 0., 0.5),
				);

				state.core.draw_text(
					&state.ui_font,
					Color::from_rgb_f(0.7, 0.7, 0.9),
					self.display_width / 2.,
					self.display_height / 2. - 24.,
					FontAlign::Centre,
					"CRASHED!",
				);
				state.core.draw_text(
					&state.ui_font,
					Color::from_rgb_f(0.7, 0.7, 0.9),
					self.display_width / 2.,
					self.display_height / 2. + 24.,
					FontAlign::Centre,
					&format!(
						"PRESS {} TO RESTART",
						state
							.options
							.controls
							.controls
							.get_by_left(&controls::Action::Restart)
							.unwrap()
							.to_str()
							.to_uppercase()
					),
				);
			}

			if self.minimap_state
			{
				let cx = self.display_width / 2.;
				let cy = self.display_height / 2.;
				let w = 512.;
				let ox = cx - w / 2.;
				let oy = cy - w / 2.;

				state.prim.draw_filled_rectangle(
					ox,
					oy,
					ox + w,
					oy + w,
					Color::from_rgba_f(0., 0., 0.3, 0.5),
				);

				let f = (0.5 + 0.5 * (state.time() * 10.).sin()) as f32;

				let w = w - 32.;

				if let Ok(pos) = self.world.get::<&comps::Position>(self.player)
				{
					let color = Color::from_rgba_f(0.1, 0.9, 0.1, 0.5);
					state.prim.draw_filled_circle(
						ox + pos.pos.x / self.size as f32 * w,
						oy + pos.pos.y / self.size as f32 * w,
						6. * f + 8. * (1. - f),
						color,
					);
				}

				for (_, (pos, mushroom)) in self
					.world
					.query_mut::<(&comps::Position, &comps::Mushroom)>()
				{
					let color = Color::from_rgba_f(0.9, 0.6, 0.4, 0.5);
					if mushroom.on_fire
					{
						state.prim.draw_filled_circle(
							ox + pos.pos.x / self.size as f32 * w,
							oy + pos.pos.y / self.size as f32 * w,
							4. * f + 5. * (1. - f),
							color,
						);
					}
				}

				for (_, (pos, _)) in self
					.world
					.query_mut::<(&comps::Position, &comps::Obelisk)>()
				{
					let color = Color::from_rgba_f(0.9, 0.1, 0.8, 0.5);
					state.prim.draw_filled_circle(
						ox + pos.pos.x / self.size as f32 * w,
						oy + pos.pos.y / self.size as f32 * w,
						4. * f + 5. * (1. - f),
						color,
					);
				}
			}
		}
		else if self.ui_state == UIState::Victory
		{
			state.prim.draw_filled_rectangle(
				0.,
				0.,
				self.display_width,
				self.display_height,
				Color::from_rgba_f(0., 0., 0., 0.5),
			);

			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.7, 0.7, 0.9),
				self.display_width / 2.,
				self.display_height / 2. - 48. * 2.,
				FontAlign::Centre,
				&format!("YOU DID IT!",),
			);

			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.7, 0.7, 0.9),
				self.display_width / 2.,
				self.display_height / 2. - 48. * 1.,
				FontAlign::Centre,
				&format!("YOU DUMPED {} TONS OF WATER", self.num_blobs,),
			);

			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.7, 0.7, 0.9),
				self.display_width / 2.,
				self.display_height / 2. + 48. * 0.,
				FontAlign::Centre,
				&format!("YOU EXTINGUISHED {} MUSHROOMS", self.num_extinguished,),
			);

			let accuracy = self.num_extinguished as f32 / self.num_blobs as f32;
			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.7, 0.7, 0.9),
				self.display_width / 2.,
				self.display_height / 2. + 48. * 1.,
				FontAlign::Centre,
				&format!(
					"YOUR ACCURACY WAS {:.2}{}",
					accuracy,
					if accuracy > 0.9 { " WOW!" } else { "" }
				),
			);

			state.core.draw_text(
				&state.ui_font,
				Color::from_rgb_f(0.7, 0.7, 0.9),
				self.display_width / 2.,
				self.display_height / 2. + 48. * 2.,
				FontAlign::Centre,
				"PRESS ESCAPE TO TRY AGAIN",
			);
		}
		else
		{
			if let Some(subscreen) = self.subscreens.last()
			{
				state.prim.draw_filled_rectangle(
					0.,
					0.,
					self.display_width,
					self.display_height,
					Color::from_rgba_f(0., 0., 0., 0.5),
				);
				subscreen.draw(state);
			}
		}

		Ok(())
	}
}
