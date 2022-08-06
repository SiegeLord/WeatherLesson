use nalgebra as na;
use na::{Point2, Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Position
{
	pub pos: Point3<f32>,
	pub dir: f32,
}
