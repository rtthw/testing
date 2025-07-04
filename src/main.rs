


use bog::prelude::*;



fn main() {
    let mut game_state = GameState {
        player_position: Vec2::ZERO,
        player_stats: PlayerStats {
            move_speed: 100.0,
        },

        movement_procs: vec![Box::new(SwiftnessShoes {})],
    };

    let mut player_movement = PlayerMovement {
        speed: game_state.player_stats.move_speed,
    };
    player_movement.process(&mut game_state);

    assert!(player_movement.speed.round() == 120.0);
    println!("WORKS");
}



pub trait Process<State> {
    fn process(&mut self, state: &mut State);
}



pub struct GameState {
    pub player_position: Vec2,
    pub player_stats: PlayerStats,
    pub movement_procs: Vec<Box<dyn Process<PlayerMovement>>>,
}

pub struct PlayerStats {
    pub move_speed: f32,
}

pub struct PlayerMovement {
    pub speed: f32,
}

impl Process<GameState> for PlayerMovement {
    fn process(&mut self, state: &mut GameState) {
        for proc in state.movement_procs.iter_mut() {
            proc.process(self);
        }
    }
}

pub struct SwiftnessShoes {}

impl Process<PlayerMovement> for SwiftnessShoes {
    fn process(&mut self, state: &mut PlayerMovement) {
        state.speed *= 1.2;
    }
}
