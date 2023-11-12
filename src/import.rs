//! Import circuits from various formats

#![allow(missing_debug_implementations)]

#[cfg(feature = "yosys-import")]
pub mod yosys;

use crate::{HashMap, WireId};
use std::sync::Arc;

/// The list of connections of an imported module
#[derive(Debug, Default)]
pub struct ModuleConnections {
    /// The inputs of the module
    pub inputs: HashMap<Arc<str>, WireId>,
    /// The outputs of the module
    pub outputs: HashMap<Arc<str>, WireId>,
}

/// Imports a module into a simulation
pub trait ModuleImporter {
    /// An error that can occur while importing
    type Error;

    /// The name of the module
    fn module_name(&self) -> &str;

    /// Imports the module into the given simulation
    fn import_into(
        &self,
        builder: &mut crate::SimulatorBuilder,
    ) -> Result<ModuleConnections, Self::Error>;
}
