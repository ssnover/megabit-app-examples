use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, simple::*},
    kv_store,
};
use rand::prelude::*;

type BoardState = [[u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let mut state_a = [[0u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];
    let state_b = [[0u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];

    let mut rng = rand::thread_rng();
    for row in state_a.iter_mut() {
        for elem in row {
            for bit in 0..8 {
                let val = rng.gen_range(0..4);
                if val == 3 {
                    *elem |= 1 << bit;
                }
            }
        }
    }

    kv_store::write("state_a", state_a)?;
    kv_store::write("state_b", state_b)?;
    kv_store::write("show_state_a", true)?;

    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    let mut state_a: BoardState = kv_store::read("state_a")?.unwrap();
    let mut state_b: BoardState = kv_store::read("state_b")?.unwrap();
    let show_state_a: bool = kv_store::read("show_state_a")?.unwrap();

    // Show the last state calculated
    let (shown_state, working_state) = if show_state_a {
        (&state_a, &mut state_b)
    } else {
        (&state_b, &mut state_a)
    };
    let shown_state_data = shown_state
        .iter()
        .cloned()
        .map(|row| row.into_iter())
        .flatten()
        .collect::<Vec<u8>>();
    display::write_region(
        (0, 0),
        (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
        shown_state_data,
    )?;
    display::render((0..SCREEN_HEIGHT as u8).into_iter().collect())?;

    // Calculate the next state
    step(working_state, shown_state);

    kv_store::write("state_a", state_a)?;
    kv_store::write("state_b", state_b)?;
    kv_store::write("show_state_a", !show_state_a)?;

    Ok(())
}

fn get_cell(state: &BoardState, col: usize, row: usize) -> bool {
    (state[row][col / 8] & (1 << (col % 8))) != 0
}

fn set_cell(state: &mut BoardState, col: usize, row: usize, val: bool) {
    if val {
        state[row][col / 8] |= 1 << (col % 8);
    } else {
        state[row][col / 8] &= !(1 << (col % 8));
    }
}

fn toggle_cell(state: &mut BoardState, col: usize, row: usize) {
    set_cell(state, col, row, !get_cell(state, col, row));
}

fn step(next: &mut BoardState, prev: &BoardState) {
    for col in 0..SCREEN_WIDTH {
        for row in 0..SCREEN_HEIGHT {
            match (get_cell(prev, col, row), neighbors_alive(prev, col, row)) {
                (true, 2..=3) => set_cell(next, col, row, true),
                (false, 3) => set_cell(next, col, row, true),
                (true, _) => set_cell(next, col, row, false),
                (false, _) => set_cell(next, col, row, false),
            }
        }
    }
}

fn neighbors_alive(state: &BoardState, col: usize, row: usize) -> u8 {
    let lower_x = col.saturating_sub(1);
    let upper_x = core::cmp::min(col + 1, SCREEN_WIDTH - 1);
    let lower_y = row.saturating_sub(1);
    let upper_y = core::cmp::min(row + 1, SCREEN_HEIGHT - 1);

    let mut count = 0;
    for x in lower_x..=upper_x {
        for y in lower_y..=upper_y {
            if col == x && row == y {
                continue;
            }
            if get_cell(state, x, y) {
                count += 1;
            }
        }
    }
    count
}
