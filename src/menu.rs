use crate::error::Result;
use crate::{components, controls, game_state, map, ui, utils};

use allegro::*;
use allegro_sys::*;
use nalgebra::Matrix4;
use rand::prelude::*;

pub struct Menu
{
	display_width: f32,
	display_height: f32,
	next_level: String,

	subscreens: Vec<ui::SubScreen>,
}

impl Menu
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		//~ if state.options.play_music
		//~ {
		//~ state.sfx.set_music_file("data/evil_minded.mod");
		//~ state.sfx.play_music()?;
		//~ }
		state.paused = false;
		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;

		Ok(Self {
			display_width: display_width,
			display_height: display_height,
			subscreens: vec![ui::SubScreen::MainMenu(ui::MainMenu::new(
				display_width,
				display_height,
			))],
			next_level: "".into(),
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
				return Ok(None);
			}
		}
		if let Some(action) = self.subscreens.last_mut().unwrap().input(state, event)
		{
			match action
			{
				ui::Action::MainMenu =>
				{
					self.subscreens
						.push(ui::SubScreen::MainMenu(ui::MainMenu::new(
							self.display_width,
							self.display_height,
						)));
				}
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
				ui::Action::Start =>
				{
					return Ok(Some(game_state::NextScreen::Game {
						seed: thread_rng().gen(),
					}))
				}
				ui::Action::Quit => return Ok(Some(game_state::NextScreen::Quit)),
				ui::Action::Back =>
				{
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
		self.subscreens.last().unwrap().draw(state);
		Ok(())
	}
}
