use crate::error::Result;
use crate::{components, controls, game_state, map, ui, utils};

use allegro::*;
use allegro_sys::*;
use nalgebra::{Matrix4, Point2};
use rand::prelude::*;

pub struct Menu
{
	display_width: f32,
	display_height: f32,
	switch_time: f64,
	do_switch: bool,
	seed: u64,

	subscreens: Vec<ui::SubScreen>,
}

impl Menu
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		if state.options.play_music
		{
			state.sfx.set_music_file("data/smoothsea.xm");
			state.sfx.play_music()?;
		}

		state.cache_sprite("data/title.cfg")?;
		state.paused = false;
		state.hide_mouse = false;
		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;

		let seed = state
			.options
			.seed
			.unwrap_or_else(|| thread_rng().gen::<u16>() as u64);
		dbg!(seed);

		Ok(Self {
			display_width: display_width,
			display_height: display_height,
			subscreens: vec![ui::SubScreen::MainMenu(ui::MainMenu::new(
				display_width,
				display_height,
			))],
			switch_time: 0.,
			seed: seed,
			do_switch: false,
		})
	}

	pub fn input(
		&mut self, event: &Event, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		if let Event::KeyDown {
			keycode: KeyCode::Escape,
			..
		} = event
		{
			if self.subscreens.len() > 1
			{
				state.sfx.play_sound("data/ui2.ogg").unwrap();
				self.subscreens.pop().unwrap();
				self.do_switch = false;
				return Ok(None);
			}
		}
		if let Some(action) = self.subscreens.last_mut().unwrap().input(state, event)
		{
			match action
			{
				ui::Action::MainMenu =>
				{
					self.do_switch = false;
					self.subscreens
						.push(ui::SubScreen::MainMenu(ui::MainMenu::new(
							self.display_width,
							self.display_height,
						)));
				}
				ui::Action::LevelMenu =>
				{
					self.do_switch = true;
					self.switch_time = state.time();
					self.subscreens
						.push(ui::SubScreen::LevelMenu(ui::LevelMenu::new(
							self.display_width,
							self.display_height,
							self.seed,
							state,
						)));
				}
				ui::Action::ControlsMenu =>
				{
					self.do_switch = true;
					self.switch_time = state.time();
					self.subscreens
						.push(ui::SubScreen::ControlsMenu(ui::ControlsMenu::new(
							self.display_width,
							self.display_height,
							state,
						)));
				}
				ui::Action::OptionsMenu =>
				{
					self.do_switch = true;
					self.switch_time = state.time();
					self.subscreens
						.push(ui::SubScreen::OptionsMenu(ui::OptionsMenu::new(
							self.display_width,
							self.display_height,
							state,
						)));
				}
				ui::Action::Start =>
				{
					return Ok(Some(game_state::NextScreen::Game {
						seed: self.seed,
						restart_music: true,
					}))
				}
				ui::Action::Quit => return Ok(Some(game_state::NextScreen::Quit)),
				ui::Action::Back =>
				{
					self.do_switch = false;
					self.subscreens.pop().unwrap();
				}
				_ => (),
			}
		}
		Ok(None)
	}

	pub fn draw(&mut self, state: &game_state::GameState) -> Result<()>
	{
		state.core.clear_to_color(Color::from_rgb_f(0., 0., 0.));
		let sprite = "data/title.cfg";
		let sprite = state
			.get_sprite(sprite)
			.expect(&format!("Could not find sprite: {}", sprite));

		let h = 64.;
		if self.do_switch
		{
			let f = 1. - utils::clamp((state.time() - self.switch_time) / 0.25, 0., 1.) as f32;
			sprite.draw(
				Point2::new(self.display_width / 2., h),
				1,
				Color::from_rgba_f(f, f, f, f),
				state,
			);
		}
		else
		{
			sprite.draw(
				Point2::new(self.display_width / 2., h),
				0,
				Color::from_rgb_f(1., 1., 1.),
				state,
			);
		}
		self.subscreens.last().unwrap().draw(state);
		Ok(())
	}
}
