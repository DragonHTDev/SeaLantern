pub mod server;
pub mod settings;

pub use crate::models::{
    AppSettings, InstanceList, JavaInfo, NullablePatch, PartialAppSettings, SettingsGroup,
    UpdateResult,
};
pub use settings::InstanceRegistry;
pub use settings::{SettingsError, SettingsManager};

pub use server::{ConfigEntry, ServerProperties, ServerPropertiesError, ServerPropertiesManager};
