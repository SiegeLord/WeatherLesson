use crate::error::Result;
use crate::{atlas, controls, sfx, sprite, utils};
use allegro::*;
use allegro_font::*;
use allegro_image::*;
use allegro_primitives::*;
use allegro_ttf::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Options
{
	pub fullscreen: bool,
	pub width: i32,
	pub height: i32,
	pub play_music: bool,
	pub vsync_method: i32,
	pub sfx_volume: f32,
	pub music_volume: f32,
	pub controls: controls::Controls,
}

pub enum NextScreen
{
	Game
	{
		seed: u64,
		restart_music: bool,
	},
	Menu,
	Quit,
}

pub struct GameState
{
	pub core: Core,
	pub prim: PrimitivesAddon,
	pub image: ImageAddon,
	pub font: FontAddon,
	pub ttf: TtfAddon,
	pub tick: i64,
	pub paused: bool,
	pub hide_mouse: bool,

	pub swirl_amount: f32,

	pub sfx: sfx::Sfx,
	pub atlas: atlas::Atlas,
	pub ui_font: Font,
	pub number_font: Font,
	pub options: Options,
	pub draw_scale: f32,
	pub display_width: f32,
	pub display_height: f32,
	bitmaps: HashMap<String, Bitmap>,
	sprites: HashMap<String, sprite::Sprite>,
}

impl GameState
{
	pub fn new() -> Result<GameState>
	{
		let options: Options = utils::load_config("options.cfg")?;
		let core = Core::init()?;
		let prim = PrimitivesAddon::init(&core)?;
		let image = ImageAddon::init(&core)?;
		let font = FontAddon::init(&core)?;
		let ttf = TtfAddon::init(&font)?;
		core.install_keyboard()
			.map_err(|_| "Couldn't install keyboard".to_string())?;
		core.install_mouse()
			.map_err(|_| "Couldn't install mouse".to_string())?;

		let sfx = sfx::Sfx::new(options.sfx_volume, options.music_volume, &core)?;

		let ui_font = ttf
			.load_ttf_font("data/MHTIROGLA.ttf", -32, TtfFlags::zero())
			.map_err(|_| "Couldn't load 'data/MHTIROGLA.ttf'".to_string())?;
		let number_font = ttf
			.load_ttf_font("data/MHTIROGLA.ttf", -32, TtfFlags::zero())
			.map_err(|_| "Couldn't load 'data/advanced_pixel_lcd-7.ttf'".to_string())?;

		Ok(GameState {
			options: options,
			core: core,
			prim: prim,
			image: image,
			tick: 0,
			bitmaps: HashMap::new(),
			sprites: HashMap::new(),
			font: font,
			ttf: ttf,
			sfx: sfx,
			paused: false,
			atlas: atlas::Atlas::new(2048),
			ui_font: ui_font,
			number_font: number_font,
			draw_scale: 1.,
			display_width: 0.,
			display_height: 0.,
			swirl_amount: 0.,
			hide_mouse: false,
		})
	}

	pub fn transform_mouse(&self, x: f32, y: f32) -> (f32, f32)
	{
		let bw = 800.;
		let bh = 600.;

		let x = (x - self.display_width / 2.) / self.draw_scale + bw / 2.;
		let y = (y - self.display_height / 2.) / self.draw_scale + bh / 2.;
		(x, y)
	}

	pub fn cache_bitmap<'l>(&'l mut self, name: &str) -> Result<&'l Bitmap>
	{
		Ok(match self.bitmaps.entry(name.to_string())
		{
			Entry::Occupied(o) => o.into_mut(),
			Entry::Vacant(v) => v.insert(utils::load_bitmap(&self.core, name)?),
		})
	}

	pub fn cache_sprite<'l>(&'l mut self, name: &str) -> Result<&'l sprite::Sprite>
	{
		Ok(match self.sprites.entry(name.to_string())
		{
			Entry::Occupied(o) => o.into_mut(),
			Entry::Vacant(v) => v.insert(sprite::Sprite::load(name, &self.core, &mut self.atlas)?),
		})
	}

	pub fn get_bitmap<'l>(&'l self, name: &str) -> Option<&'l Bitmap>
	{
		self.bitmaps.get(name)
	}

	pub fn get_sprite<'l>(&'l self, name: &str) -> Option<&'l sprite::Sprite>
	{
		self.sprites.get(name)
	}

	pub fn time(&self) -> f64
	{
		self.tick as f64 * utils::DT as f64
	}
}
