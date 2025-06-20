// TAO Infrastructure - High-level TAO interface and operations

pub mod tao_interface;
pub mod tao_operations;
pub mod viewer;

// Re-export main interface
pub use tao_interface::{TaoInterface, create_tao_router};
pub use viewer::Viewer;