#[derive(Debug, Clone, Copy)]
pub enum Cell {
  Empty,
  Energy,
  Wall1,
  Wall2,
  Wall3,
  Wall4,
  Diamond,
  DroneDown,
  DroneLeft,
  DroneRight,
  DroneUp,
}
