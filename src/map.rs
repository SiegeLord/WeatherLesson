use crate::error::Result;
use crate::{game_state, sprite, utils};

use allegro::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;
use rand::prelude::*;

fn decode_tile(vals: [i32; 4], x: i32, y: i32) -> i32
{
	match vals
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
		_ => {
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
	//~ let seed = 11961432304471787294;
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
	print_heightmap(&heightmap);
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
			//~ println!();
			// Check the diag corners
			for sy in [-1, 1]
			{
				for sx in [-1, 1]
				{
					let cx = x + sx;
					let cy = y + sy;
					if cx >= 0 && cy >= 0 && cx < real_size && cy < real_size
					{
						let val = heightmap[(cx + cy * real_size) as usize];
						mean_height = (mean_height * count as f32 + val as f32)
							/ (count + 1) as f32;
						count += 1;
					}
				}
			}
			res[(x + y * real_size) as usize] = mean_height as i32;
		}
	}
	res
}

fn world_to_screen(pos: Vector3<f32>) -> Vector2<f32>
{
	Vector2::new(
		pos.x as f32 * 64. - pos.y as f32 * 64.,
		pos.x * 32. + pos.y * 32. - pos.z * 24.,
	)
}

pub struct Map
{
	heightmap: Vec<i32>,
	size: i32,
	display_width: f32,
	display_height: f32,
	tiles: sprite::Sprite,
	camera_pos: utils::Vec2D,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		let size = 5;
		Ok(Self {
			heightmap: smooth_heightmap(&diamond_square(size)),
			size: 2i32.pow(size as u32) + 1,
			display_width: display_width,
			display_height: display_height,
			tiles: sprite::Sprite::load("data/terrain.cfg", state)?,
			camera_pos: utils::Vec2D::zeros(),
		})
	}

	pub fn logic(
		&mut self, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		Ok(None)
	}

	pub fn input(
		&mut self, event: &Event, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		match event
		{
			Event::KeyDown { keycode, .. } => match keycode
			{
				KeyCode::Left =>
				{
					self.camera_pos.x -= 16.;
				}
				KeyCode::Right =>
				{
					self.camera_pos.x += 16.;
				}
				KeyCode::Up =>
				{
					self.camera_pos.y -= 16.;
				}
				KeyCode::Down =>
				{
					self.camera_pos.y += 16.;
				}
				_ => (),
			},
			_ => (),
		}
		Ok(None)
	}

	pub fn draw(&self, state: &game_state::GameState) -> Result<()>
	{
		state.core.clear_to_color(Color::from_rgb_f(0., 0., 0.));

		let dx = self.display_width / 2. - self.camera_pos.x;
		let dy = self.display_height / 2. - self.camera_pos.y;
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

				let variant = decode_tile(vals, x, y);
				let xy = world_to_screen(Vector3::new(x as f32, y as f32, min_val as f32));
				self.tiles.draw(
					xy - utils::Vec2D::new(64. - dx, 96. - dy),
					variant,
					Color::from_rgb_f(1., 1., 1.),
					state,
				);

				//~ let xy = world_to_screen(Vector3::new(
					//~ x as f32,
					//~ y as f32,
					//~ min_val as f32 + vals.iter().sum::<i32>() as f32 / 4.,
				//~ ));

				//~ state.prim.draw_filled_circle(
					//~ dx + xy.x,
					//~ dy + xy.y,
					//~ 16.,
					//~ Color::from_rgb_f(1., 0.6, 0.6),
				//~ );
			}
		}
		Ok(())
	}
}
