pub mod server;
pub mod settings;

#[allow(deprecated)]
pub use crate::models::compatibility::{ServerStatus, StartupMode};
pub use crate::models::{
    AppSettings, InstanceList, JavaInfo, NullablePatch, PartialAppSettings, SettingsGroup,
    UpdateResult,
};
pub use settings::InstanceRegistry;
pub use settings::{SettingsError, SettingsManager};

pub use server::{ConfigEntry, ServerProperties, ServerPropertiesError, ServerPropertiesManager};

/// 解析应用数据目录，优先使用环境变量 `SEALANTERN_DATA_DIR`。
pub use sealantern_infra::platform::get_app_data_dir as resolve_data_dir;
