//! 实例注册表设置分区。
//!
//! 记录主资源目录、附加服务器目录与信任状态。实例本身的权威配置位于各服务器
//! 目录内的 `sl.json`；本分区只保存「哪些目录被本机管理」这类机器级信息。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 实例注册表：主资源目录、附加服务器目录与信任状态。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstanceRegistrySection {
    /// 主资源目录；`None` 表示与主配置目录同址。
    pub main_resource_dir: Option<PathBuf>,

    /// 附加服务器目录；每个目录本身就是一个服务器。
    pub extra_server_dirs: Vec<PathBuf>,

    /// 已信任的实例标识。
    pub trusted_instances: Vec<String>,

    /// 用户明确忽略、不再提示的目录。
    pub ignored_dirs: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let registry: InstanceRegistrySection =
            serde_json::from_str("{}").expect("empty registry should deserialize");

        assert_eq!(registry, InstanceRegistrySection::default());
    }

    #[test]
    fn registry_round_trips_through_json() {
        let registry = InstanceRegistrySection {
            main_resource_dir: Some(PathBuf::from("D:\\SeaLanternData")),
            extra_server_dirs: vec![PathBuf::from("E:\\MyServers\\paper-a")],
            trusted_instances: vec!["a1b2c3".to_string()],
            ignored_dirs: vec![PathBuf::from("F:\\skip-me")],
        };

        let json = serde_json::to_string(&registry).expect("registry should serialize");
        let restored: InstanceRegistrySection =
            serde_json::from_str(&json).expect("registry should deserialize");

        assert_eq!(registry, restored);
    }

    #[test]
    fn settings_without_a_registry_section_still_load() {
        let settings: crate::settings::AppSettings =
            serde_json::from_str("{}").expect("settings without a registry should load");

        assert_eq!(settings.registry, InstanceRegistrySection::default());
    }
}
