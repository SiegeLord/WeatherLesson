use crate::sprite;
use na::{Point2, Point3, Vector3};
use nalgebra as na;

#[derive(Debug, Copy, Clone)]
pub struct Position
{
	pub pos: Point3<f32>,
	pub dir: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity
{
	pub vel: Vector3<f32>,
	pub dir_vel: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct FixedEngine
{
	pub power: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct CastsShadow;

#[derive(Debug, Clone)]
pub struct Drawable
{
	pub sprite: sprite::Sprite,
}
