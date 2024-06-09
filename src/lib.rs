use rl_ball_sym::{Ball, Game, Vec3A};
use std::sync::RwLock;

const TPS: usize = 120;
const DT: f32 = 1.0 / TPS as f32;

static GAME: RwLock<Option<Game>> = RwLock::new(None);
static BALL: RwLock<Ball> = RwLock::new(Ball::const_default());

fn set_game_and_ball((game, ball): (Game, Ball)) {
    let mut game_lock = GAME.write().unwrap();
    *game_lock = Some(game);

    let mut ball_lock = BALL.write().unwrap();
    *ball_lock = ball;
}

#[no_mangle]
pub extern "C" fn load_standard() {
    set_game_and_ball(rl_ball_sym::load_standard());
}

#[no_mangle]
pub extern "C" fn load_dropshot() {
    set_game_and_ball(rl_ball_sym::load_dropshot());
}

#[no_mangle]
pub extern "C" fn load_hoops() {
    set_game_and_ball(rl_ball_sym::load_hoops());
}

#[no_mangle]
pub extern "C" fn load_standard_throwback() {
    set_game_and_ball(rl_ball_sym::load_standard_throwback());
}

#[repr(C)]
#[derive(Default)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for Vec3A {
    #[inline]
    fn from(v: Vec3) -> Vec3A {
        Vec3A::new(v.x, v.y, v.z)
    }
}

impl From<Vec3A> for Vec3 {
    #[inline]
    fn from(v: Vec3A) -> Vec3 {
        Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct BallSlice {
    pub time: f32,
    pub location: Vec3,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl From<Ball> for BallSlice {
    #[inline]
    fn from(ball: Ball) -> BallSlice {
        BallSlice {
            time: ball.time,
            location: ball.location.into(),
            linear_velocity: ball.velocity.into(),
            angular_velocity: ball.angular_velocity.into(),
        }
    }
}

#[no_mangle]
pub extern "C" fn step(current_ball: BallSlice) -> BallSlice {
    let game_lock = GAME.read().unwrap();
    let Some(game) = game_lock.as_ref() else {
        println!("Warning: No game loaded");
        return BallSlice::default();
    };

    let mut ball = *BALL.read().unwrap();
    ball.update(
        current_ball.time,
        current_ball.location.into(),
        current_ball.linear_velocity.into(),
        current_ball.angular_velocity.into(),
    );

    ball.step(game, DT);
    ball.into()
}
