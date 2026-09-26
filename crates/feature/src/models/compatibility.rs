//! 旧公开模型路径的兼容定义。

#![allow(deprecated)]

use serde::{Deserialize, Serialize};

/// 旧服务器实例信息。
#[deprecated(note = "请使用 sealantern_core::instance::Instance")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub core_type: Option<String>,
    #[serde(default)]
    pub mc_version: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

/// 旧服务器运行时状态。
#[deprecated(note = "请在实际运行时 API 边界定义状态模型")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[allow(deprecated)]
impl ServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Error => "error",
        }
    }
}

/// 旧启动模式。
#[deprecated(note = "请使用 sealantern_core::instance::StartupMode")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StartupMode {
    Jar,
    Bat,
    Sh,
    Ps1,
    Starter,
    Custom,
}

#[allow(deprecated)]
impl StartupMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jar => "jar",
            Self::Bat => "bat",
            Self::Sh => "sh",
            Self::Ps1 => "ps1",
            Self::Starter => "starter",
            Self::Custom => "custom",
        }
    }
}
