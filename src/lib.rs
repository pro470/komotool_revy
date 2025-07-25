#![allow(rustdoc::bare_urls)]
#![allow(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;

// ---

// TODO(cmc): support for bug report mode (buffering + kickoff)

pub struct RerunPlugin {
    pub rec: RecordingStream,
}

impl Plugin for RerunPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RerunSyncPlugin {
            rec: self.rec.clone(),
        });
    }
}

// ---

mod conversions;
mod default_loggers;
mod entity_path;
mod rerun_logger;
mod sync;

pub use self::conversions::ToRerun;
pub use self::default_loggers::DefaultRerunComponentLoggers;
pub use self::entity_path::{ancestors_from_world, compute_entity_path};
pub use self::rerun_logger::{
    RerunComponentLoggers, RerunLogger, RerunLoggerFn, get_component_logger,
};

pub(crate) use self::sync::RerunSyncPlugin;

pub use rerun::{RecordingStream, RecordingStreamBuilder}; // convenience

pub mod external {
    pub use rerun;
}
