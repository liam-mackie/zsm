mod session;
mod tab;
mod traits;

#[cfg(test)]
mod mock;

pub use session::SessionTarget;
pub use tab::TabTarget;
pub use traits::Target;

#[cfg(test)]
pub use mock::MockTarget;

use crate::domain::TargetMode;

pub fn create_target(mode: TargetMode) -> Box<dyn Target> {
    match mode {
        TargetMode::Session => Box::new(SessionTarget),
        TargetMode::Tab => Box::new(TabTarget),
    }
}
