mod session;
mod tab;
mod traits;

pub use session::SessionTarget;
pub use tab::TabTarget;
pub use traits::Target;

use crate::domain::TargetMode;

pub fn create_target(mode: TargetMode) -> Box<dyn Target> {
    match mode {
        TargetMode::Session => Box::new(SessionTarget),
        TargetMode::Tab => Box::new(TabTarget),
    }
}
