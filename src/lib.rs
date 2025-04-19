use rl_ball_sym::{Ball, Game, Vec3A};
use std::{io::{stdout, Write}, sync::RwLock};

const TPS: usize = 120;
const DT: f32 = 1.0 / TPS as f32;

struct State {
    game: Option<Game>,
    ball: Ball,
    heatseeker: bool,
}

static GAME_AND_BALL: RwLock<State> = RwLock::new(State {
    game: None,
    ball: Ball::DEFAULT_STANDARD,
    heatseeker: false,
});

fn set_game_and_ball((game, ball): (Game, Ball), heatseeker: bool) {
    *GAME_AND_BALL.write().unwrap() = State {
        game: Some(game),
        ball,
        heatseeker,
    };
}

#[no_mangle]
pub extern "C" fn load_heatseeker() {
    set_game_and_ball(rl_ball_sym::load_standard_heatseeker(), true);
}

#[no_mangle]
pub extern "C" fn load_standard() {
    set_game_and_ball(rl_ball_sym::load_standard(), false);
}

#[no_mangle]
pub extern "C" fn load_dropshot() {
    set_game_and_ball(rl_ball_sym::load_dropshot(), false);
}

#[no_mangle]
pub extern "C" fn load_hoops() {
    set_game_and_ball(rl_ball_sym::load_hoops(), false);
}

#[no_mangle]
pub extern "C" fn load_standard_throwback() {
    set_game_and_ball(rl_ball_sym::load_standard_throwback(), false);
}

#[repr(C)]
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
pub extern "C" fn get_heatseeker_target_y() -> f32 {
    let state = GAME_AND_BALL.read().unwrap();
    state.ball.get_heatseeker_target().y
}

#[no_mangle]
pub extern "C" fn set_heatseeker_target(blue_goal: u8) {
    let mut state = GAME_AND_BALL.write().unwrap();
    state.ball.set_heatseeker_target(blue_goal == 1);
}

#[no_mangle]
pub extern "C" fn reset_heatseeker_target() {
    let mut state = GAME_AND_BALL.write().unwrap();
    state.ball.reset_heatseeker_target();
}

#[no_mangle]
pub extern "C" fn set_gravity(gravity: f32) {
    let mut state = GAME_AND_BALL.write().unwrap();
    let Some(game) = state.game.as_mut() else {
        // Don't use println!() to avoid bringing in the `fmt` module
        let out = stdout();
        let mut handle = out.lock();
        handle.write_all(b"Warning: No game loaded\n").unwrap();
        handle.flush().unwrap();

        return;
    };

    game.gravity.z = gravity;
}

#[no_mangle]
pub extern "C" fn step(current_ball: BallSlice, ticks: u16) -> *mut BallSlice {
    let state = GAME_AND_BALL.read().unwrap();
    let Some(game) = state.game.as_ref() else {
        // Don't use println!() to avoid bringing in the `fmt` module
        let out = stdout();
        let mut handle = out.lock();
        handle.write_all(b"Warning: No game loaded\n").unwrap();
        handle.flush().unwrap();

        return std::ptr::null_mut();
    };

    let mut ball = state.ball;
    let mut balls = Vec::with_capacity(ticks as usize);

    ball.update(
        current_ball.time,
        current_ball.location.into(),
        current_ball.linear_velocity.into(),
        current_ball.angular_velocity.into(),
    );

    for _ in 0..ticks {
        if state.heatseeker {
            ball.step_heatseeker(game, DT);
        } else {
            ball.step(game, DT);
        }

        balls.push(ball.into());
    }

    // Remove excess capacity so the length is exactly `ticks`
    balls.shrink_to_fit();
    // Turn into pointer
    let ptr = balls.as_mut_ptr();
    // Prevent the memory from being deallocated
    std::mem::forget(balls);

    ptr
}

/// # Safety
/// 
/// Ensure that the pointer is valid and that the memory it points to is not deallocated.
#[no_mangle]
pub unsafe extern "C" fn free_ball_slices(ptr: *mut BallSlice, size: u16) {
    let length = size as usize;
    Vec::from_raw_parts(ptr, length, length);
}
