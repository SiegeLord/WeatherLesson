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
pub enum DrawableKind
{
	Oriented
	{
		sprite: String
	},
	Animated
	{
		sprite: String,
		start_time: f64,
		total_duration: f64,
		once: bool,
	},
	Mushroom
	{
		sprite: String, variant: i32
	},
}

#[derive(Debug, Clone)]
pub struct Drawable
{
	pub kind: DrawableKind,
}

#[derive(Debug, Clone)]
pub struct TimeToDie
{
	pub time_to_die: f64,
}

#[derive(Debug, Clone)]
pub enum ParticleKind
{
	Stationary,
	Fire,
}

#[derive(Debug, Clone)]
pub struct ParticleSpawner
{
	pub offset: Vector3<f32>,
	pub kind: ParticleKind,
	pub spawn_delay: f64,
	pub time_to_spawn: f64,
	pub duration: f64,
	pub sprite: String,
}

#[derive(Debug, Clone)]
pub struct ParticleSpawners
{
	pub spawners: Vec<ParticleSpawner>,
}

#[derive(Debug, Copy, Clone)]
pub enum ExplosionKind
{
	Explosion,
	Splash,
}

#[derive(Debug, Clone)]
pub struct ExplodeOnCollision;

#[derive(Debug, Clone)]
pub struct AffectedByGravity;

#[derive(Debug, Clone)]
pub struct AffectedByFriction;

#[derive(Debug, Clone)]
pub struct WaterCollector
{
	pub time_to_splash: f64,
	pub time_to_drop: f64,
	pub water_amount: i32,
}

#[derive(Debug, Clone)]
pub struct Mushroom
{
	pub on_fire: bool,
	pub health: f32,
}

#[derive(Debug, Clone)]
pub enum OnDeathEffect
{
	Explosion
	{
		kind: ExplosionKind,
	},
	SplashWater,
}

#[derive(Debug, Clone)]
pub struct OnDeathEffects
{
	pub effects: Vec<OnDeathEffect>,
}
