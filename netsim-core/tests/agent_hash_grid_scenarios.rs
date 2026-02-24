use netsim_core::{AgentHashGrid, AgentSoA};

#[test]
fn agent_hash_grid_rebuild_inserts_alive_agents() {
    let mut agents = AgentSoA::new(3);
    agents.pos_x[0] = 0.1;
    agents.pos_y[0] = 0.2;
    agents.pos_x[1] = 1.4;
    agents.pos_y[1] = 0.6;
    agents.pos_x[2] = 3.1;
    agents.pos_y[2] = 0.1;

    let mut grid = AgentHashGrid::new(4, 1, 1.0);
    let skipped = grid.rebuild(&agents);

    assert_eq!(skipped, 0);
    let bucket_0 = grid.bucket(0, 0).unwrap();
    let bucket_1 = grid.bucket(1, 0).unwrap();
    let bucket_3 = grid.bucket(3, 0).unwrap();

    assert!(bucket_0.contains(&0));
    assert!(bucket_1.contains(&1));
    assert!(bucket_3.contains(&2));
}

#[test]
fn agent_hash_grid_skips_inactive_agents() {
    let mut agents = AgentSoA::new(2);
    agents.alive[1] = false;
    agents.pos_x[0] = 0.2;
    agents.pos_y[0] = 0.2;
    agents.pos_x[1] = 0.8;
    agents.pos_y[1] = 0.2;

    let mut grid = AgentHashGrid::new(1, 1, 1.0);
    let skipped = grid.rebuild(&agents);

    assert_eq!(skipped, 0);
    let bucket = grid.bucket(0, 0).unwrap();
    assert_eq!(bucket.len(), 1);
    assert_eq!(bucket[0], 0);
}

#[test]
fn agent_hash_grid_reports_out_of_bounds_agents() {
    let mut agents = AgentSoA::new(2);
    agents.pos_x[0] = -1.0;
    agents.pos_y[0] = 0.2;
    agents.pos_x[1] = 2.1;
    agents.pos_y[1] = 0.2;

    let mut grid = AgentHashGrid::new(2, 1, 1.0);
    let skipped = grid.rebuild(&agents);

    assert_eq!(skipped, 2);
}

#[test]
fn agent_hash_grid_insert_rejects_negative_coordinates() {
    let mut grid = AgentHashGrid::new(2, 2, 1.0);

    assert!(!grid.insert(0, -0.1, 0.0));
    assert!(!grid.insert(1, 0.0, -0.1));
}

#[test]
fn agent_hash_grid_handles_zero_cell_size() {
    let mut grid = AgentHashGrid::new(2, 2, 0.0);

    assert!(!grid.insert(0, 0.5, 0.5));
    assert!(grid.world_to_cell(0.5, 0.5).is_none());
}
