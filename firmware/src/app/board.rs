use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use slint::{
    PlatformError,
    platform::{Platform, WindowAdapter, software_renderer::MinimalSoftwareWindow},
};

/// Slint's view of the board: one fixed window per panel, and a clock.
pub(super) struct Board {
    /// One per panel. Slint takes these in order, one per UI that gets created.
    pub(super) windows: [Rc<MinimalSoftwareWindow>; 2],
    /// How many of `windows` Slint has already taken.
    pub(super) handed_out: Cell<usize>,
    /// Boot time, the zero for Slint's timers and animations.
    pub(super) start: Instant,
}

impl Platform for Board {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        let taken = self.handed_out.get();
        let window = self.windows.get(taken).ok_or_else(|| {
            PlatformError::Other(format!("only 2 panels, but UI {taken} asked for a window"))
        })?;
        self.handed_out.set(taken + 1);
        Ok(window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start.elapsed()
    }
}
