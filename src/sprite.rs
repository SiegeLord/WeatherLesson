use crate::error::Result;
use crate::game_state::GameState;
use crate::utils;
use allegro::*;
use na::Point2;
use nalgebra as na;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct SpriteDesc
{
	bitmap: String,
	width: i32,
	height: i32,
	#[serde(default)]
	center_x: i32,
	#[serde(default)]
	center_y: i32,
}

#[derive(Clone, Debug)]
pub struct Sprite
{
	desc: SpriteDesc,
}

impl Sprite
{
	pub fn load(sprite: &str, state: &mut GameState) -> Result<Sprite>
	{
		let desc: SpriteDesc = utils::load_config(sprite)?;
		state.cache_bitmap(&desc.bitmap)?;
		Ok(Sprite { desc: desc })
	}

	pub fn draw(&self, pos: Point2<f32>, variant: i32, tint: Color, state: &GameState)
	{
		let bitmap = state.get_bitmap(&self.desc.bitmap).unwrap();

		let w = self.desc.width as f32;
		let h = self.desc.height as f32;

		state.core.draw_tinted_bitmap_region(
			bitmap,
			tint,
			0.,
			variant as f32 * h,
			w,
			h,
			pos.x - self.desc.center_x as f32,
			pos.y - self.desc.center_y as f32,
			Flag::zero(),
		);
	}

	pub fn draw_beam(&self, pos: Point2<f32>, variant: i32, len: f32, theta: f32, state: &GameState)
	{
		let bitmap = state.get_bitmap(&self.desc.bitmap).unwrap();

		//~ let w = self.desc.width as f32;
		let h = self.desc.height as f32;

		state.core.draw_tinted_scaled_rotated_bitmap_region(
			bitmap,
			0.,
			variant as f32 * h,
			len,
			h,
			Color::from_rgb_f(1., 1., 1.),
			len / 2.,
			h / 2.,
			pos.x,
			pos.y,
			1.,
			1.,
			theta,
			Flag::zero(),
		);
	}
}
