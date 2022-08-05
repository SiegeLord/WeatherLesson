use crate::game_state;
use crate::error::Result;

use allegro::*;

pub struct Map
{
	
}

impl Map
{
	pub fn new(state: &mut game_state::GameState, display_width: f32, display_height: f32) -> Result<Self>
	{
		Ok(Self
		{
			
		})
	}
	
	pub fn logic(&self, state: &mut game_state::GameState) -> Result<Option<game_state::NextScreen>>
	{
		Ok(None)
	}
	
	pub fn input(&self, event: &Event, state: &mut game_state::GameState) -> Result<Option<game_state::NextScreen>>
	{
		Ok(None)
	}
	
	pub fn draw(&self, state: &game_state::GameState) -> Result<()>
	{
		Ok(())
	}

}
