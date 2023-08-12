use std::env;
use crate::env_names::{ENV_SOLUTION_THRESHOLD};

pub fn get_solution_threshold() -> usize {
    env::var(ENV_SOLUTION_THRESHOLD).unwrap().trim().parse::<usize>().unwrap()
}

#[test]
fn test_get_solution_threshold() {
    env::set_var(ENV_SOLUTION_THRESHOLD, " 21 ");

    assert_eq!(21, get_solution_threshold());
}