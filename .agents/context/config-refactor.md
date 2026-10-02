# 路径与配置系统重构 —— 决策与现状存档

> 目的：上下文压缩/新会话恢复时，先读本文件再动手。最后更新：阶段 3 提交 `5aff44dd` 之后。
> 恢复步骤：读本文件 → `git log --oneline -5` 核对 HEAD → `git status` 核对工作区 → 按「下一步」执行。

## 0. 当前状态快照

- 分支：`main`，直接提交（用户指令，不走 PR）。
- HEAD：`b4873e48`（阶段 5）。其前 `5b753508`（阶段 4）、`1a28c578`（审查清理批）、`5aff44dd`（阶段 3）、`d962d490`（阶段 2）、`62bcdb26`（阶段 1）、`a60cb25b`（#1064 基线）。
- 工作区：干净。`1.txt` 是未跟踪文件，**不属于本任务，勿动**。
- 版本号 1.4.0。

## 1. 目标架构

三层目录 + 每实例 `sl.json`，不做任何旧数据迁移：

- **主配置目录**（单例）：`settings.json`（应用设置 + 实例注册表分区）
- **主资源目录**（默认=配置目录，可覆盖）：`backups/`、`resources/`（server_core/resource_market/uploaded/temp）、`plugins/`、`instances/`（容器层，下面是实例目录，可改名）
- **附加服务器目录**：可挂多个，每个目录本身就是一台服务器，不嵌套
- **`<服务器目录>/sl.json`**：该实例的权威配置

## 2. 已定决策（不要推翻；编号供引用）

- **D1** 三层目录模型如上；`instances/` 容器层目录名用户可改、创建时可指定。
- **D2** `sl.json` 字段：`schema_version`(必填) / `id` / `name` / `core{type,version}` / `required_java`(Option<u32>) / `startup{mode,target(相对),jvm_args,java_path,memory_mib{min,max},port}` / `cron[]`（含 last_run_at/next_run_at/last_error，状态不分离）。
- **D3** `schema_version` 缺失=判「无有效配置」不解析不写；超前=拒绝加载提示升级程序；落后=锁内迁移+备份+原子写回。这是新格式自身版本管理，非兼容逻辑。
- **D4** **不做任何实例数据迁移**：`sea_lantern_servers.json`、旧 `<srv>/SL.json`、`data_dir.json` 定位器机制全部作废（代码已删）。注：`SL.json` 撞名问题已关闭调查——仅 2026-05-26~07-08 nightly 写过，正式版 v1.3.x 只读不写，main 已由 #847 删除；用户明确"不用考虑 1.3.x"。
- **D5** 信任模型：不判断「谁创建」，只判断「是否已登记」。`create_instance`/`import_modpack` 成功后自动 `trust_instance`（登记动作本身即导入）；仅外来目录（附加目录、拷来的）需显式信任。三态：信任/未标记(提示待处理)/忽略(黑名单防反复提示)。**信任状态存 settings.json（机器级），绝不写进 sl.json**（否则随目录拷贝绕过冲突检测）。
- **D6** 检查分两层：**身份检查**（sl.json 可解析/id 有效/id 本机唯一）导入时硬性，不过不能导入；**运行前置检查**（java 路径可用/java 版本≥required_java/端口/可写）导入时提示、启动时才硬性。
- **D7** 重复 id「只报告不自动去重」：按 id 分组报告，选项=标记为副本/重新分配 id/删除/忽略；冲突检测独立于信任状态。
- **D8** 无中心化锁，锁边界=数据边界。`FileLock::try_acquire(<dir>/sl.json)` → 锁文件 `<dir>/sl.json.lock` 天然在实例目录内（陈旧锁：PID+15min 兜底）。`ConfigFile<T>`（`infra/persistence/config.rs`）直接复用：原子写、锁内读-改-写、10MiB 上限、`.bak-<ts>-<uuid>` 备份。
- **D9** settings.json 双写路径互不染指：`update_partial(PartialAppSettings)` 改偏好 / `update_registry(闭包)` 改注册表分区；共用文件与锁。`PartialAppSettings` 刻意不含 `registry`。
- **D10** 目录规则唯一来源 = `infra::platform::locations::AppLayout`（纯数据：`new(config_dir, resource_override)`+`native()`+`config_file/resource_file/instances_dir/backups_dir/resources_dir/plugins_dir`；注意 `platform/mod.rs` 的 locations 是私有模块，需显式 `pub use` 重导出否则 dead_code lint）。`plugins/` 归 resource_dir、`backup_settings/` 归 config_dir。
- **D11** `cron_tasks.json` 将消失，cron 元素=现 `CronTask` 去掉 `server_id`（`server_id` 在 sl.json 内冗余、位置隐含归属；契约层保留）。调度器更新须走 `update_persisted_if_changed` 防覆盖用户同时写的配置。
- **D12** `instances.json` 并入 `settings.json`（用户要求减少文件与版本号维护）。实例级路径不存（可推导）。
- **D13** discovery 层纯只读：绝不顺手修复、绝不自动信任。
- **D14** `AppSettings` 的 `default_java_path`/`default_*_memory`/`default_port` 仅作创建时初始值来源，不参与运行时回落。
- **D15**（审查裁决）信任键=id（方案 B），**否决按路径**：因为允许改名实例目录、允许改主资源目录，按路径会让两者都丢信任；且 ignored 必须按路径（被忽略的目录可能无有效 id）、两者键不一致是语义使然。配套实现规则：①删除实例(含移除保目录→转 ignored)必须清理 `trusted_instances`；②扫描时做孤儿检测（名单有 id、扫描无对应实例→报告/清理）；③跨机拷贝天然安全（settings.json 机器本地）。
- **D16** 服务器级 `java_path`/`memory`/`port` 单层存 sl.json，**取消中心 server_bindings 覆盖层**（开发组普遍赞同）。
- **D17** `sl.json` 内去掉 `server_id` 字段（位置隐含归属），契约层类型保留。

## 3. 已完成阶段

| commit              | 内容                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `62bcdb26` 阶段1    | 删 `data_migration.rs`(429行)/`types.rs`/`store.rs` legacy 迁移/`LegacyServerInstance`/7 个零使用兼容类型；`config/mod.rs` 改从 `crate::models` re-export；净 -825 行                                                                                                                                                                                                                                                     |
| `d962d490` 阶段2    | `git mv sealantern/ settings/`；新增 `AppLayout`（locations.rs，3 测试）；`platform/mod.rs` 重导出；SettingsManager/BackupManager/BackupSettingsManager 改走 AppLayout                                                                                                                                                                                                                                                    |
| `5aff44dd` 阶段3    | `contract/settings/registry.rs`=`InstanceRegistrySection{main_resource_dir,extra_server_dirs,trusted_instances,ignored_dirs}`(serde default,3测试)；`AppSettings.registry`；`models/mod.rs` 重导出；`SettingsManager::update_registry`（走 ConfigFile::update_persisted_if_changed，与 update_partial 共用锁只碰 registry，2 个互不染指测试）；`AppLayout::plugins_dir()`；plugin/service.rs:84 与 services.rs:254 改走它 |
| `1a28c578` 审查清理 | 删 `models/compatibility.rs`（三 deprecated 类型零使用）与 `resolve_data_dir`；`InstanceRegistrySection::validate()`（空路径/空 id/重复条目拒绝）+ `update_registry` 换 `try_update_persisted_if_changed` 带错误通道；设置文件名 `sea_lantern_settings.json`→`settings.json` + `load_default` 一次性 rename 旧文件（新文件存在则不动）                                                                                    |
| `5b753508` 阶段4    | `config/instance/`：`document.rs`（`InstanceDocument`/`CoreSpec`/`StartupSpec`/`MemorySpec`/`InstanceCronEntry`/`DocumentStore`/`read_document_bytes`）+ `error.rs`（`DocumentError{Storage,UnsupportedSchemaVersion,AlreadyExists,Invalid}`）。`LocalLaunch::normalize_and_validate` 放开 `pub`                                                                                                                          |
| `b4873e48` 阶段5    | `config/instance/discovery.rs`：`discover(layout,registry)->DiscoveryReport`（永不整体失败）；`DiscoveryProblem` 六变体（UnreadableDir/MissingDocument/InvalidDocument/UnsupportedSchemaVersion/DuplicateId 按id分组/DuplicateDirAlias）；`normalize_dir`/`dir_key` 纯词法规范化（pub(crate)，零 IO，供阶段6名单匹配复用）；先读 schema_version 再解析                                                                    |

栈 `#1058` rebase 已完成（3 冲突已解：instance.rs 保留 derive\_\*+resolve_instance_directory 两者；service/server_config.rs 删已迁走函数；handlers/server_config.rs 保留 PathBuf 版+`map_err(HttpError::from)?`），7 层连续性 OK，随后 7 个 PR 全部 `gh pr ready --undo` 转 draft（用户计划颠覆其内容，属历史包袱已关闭）。

## 4. 剩余阶段

- ~~阶段 4~~ 已完成（`5b753508`）：`config/instance/`。`InstanceDocument` 字段比 D2 多（对齐 `InstanceSpec` 全集：`aliases`/`mc_version`/`created_at`/`last_started_at`/`server_metadata`）。`StartupSpec`：serde 名 `mode/target/jvm_args/java_path/memory_mib{min,max}/port/custom_*`；`target` 存相对路径，`to_spec(dir)` 换算回完整路径、`try_from_instance` strip_prefix。`InstanceCronEntry`=`CronTask` 去掉 `server_id`，`to_cron_task/from_cron_task` 双向转换已备好（供阶段 7 cron 接线）。`DocumentStore::load`：NotFound/损坏→Storage 上抛（发现层分类）、超前→UnsupportedSchemaVersion、落后→锁内迁移+备份；`update` 强制 schema_version+写前 validate；`create` 锁内独占（AlreadyExists）。`read_document_bytes` 已预留给发现层探测。
- ~~阶段 5~~ 已完成（`b4873e48`）：`discovery.rs`。规范化为**词法档**（已定案，否决 canonicalize）：`normalize_dir`/`dir_key` 为 `pub(crate)`，阶段 6 的 `ignored_dirs`/`extra_server_dirs` 匹配**必须复用同一函数**。目录别名（符号链接等）走 `DuplicateId` 如实报告。`DiscoveryReport{instances,problems}`；`DiscoveredInstance{dir,origin(Managed/Extra),document}`。
- **阶段 6** `trust.rs`：三态分类+`trust_instance`+名单垃圾回收（D15 配套规则）；**前置阻塞=P3-5**（必须先有 layout 出口，否则 main_resource_dir 可设后各服务写错目录）。
- **阶段 7** 接线 `application/src/service/instance.rs`：换掉 `InstanceRegistry`（JSON 中心化）→ 新 discover/trust/save；同步改注释措辞（P3-4）；application 层组装 `AppLayout` 注入各服务。

## 5. 外部审查结论与本龙裁决（全部已处置）

- P2-1 trusted 按 id vs 路径 → **已定案 D15**（方案 B + 3 条配套规则）。
- P2-2 文件名 → **已落地**（`1a28c578`）：`SETTINGS_FILE_NAME="settings.json"`，`load_default` 里 `rename_legacy_settings_file` 一次性重命名 `sea_lantern_settings.json`（新文件存在/旧文件缺失则不动；rename 失败报错不静默回落）。
- P3-1 compatibility 三类型 → **已删**（`1a28c578`）。
- P3-2 `resolve_data_dir` → **已删**（`1a28c578`）。
- P3-3 `update_registry` 校验 → **已落地**（`1a28c578`）：`InstanceRegistrySection::validate()` + `try_update_persisted_if_changed` 错误通道；`SettingsValidationError::new` 开放为 `pub(crate)`。
- P3-4 `instance.rs:34` 注释措辞 → 阶段 7 一并改。
- P3-5 `SettingsManager` 无 AppLayout 出口 → **未做**，阶段 6 阻塞项：`SettingsManager::layout()`（`path.parent()`+`registry.main_resource_dir` 现构造）+ 各服务改为注入 AppLayout。

## 6. 工程约定（必须遵守）

- 全程中文思考/回复；注释与文档中文；**代码里一切字符串英文**（expect/assert 消息、`#[error]`、`format!`、测试夹具、回落值）。
- 只改本任务涉及文件，不顺手重构无关代码；别人写的/既有中文 format! 不碰。
- 验证三件套（提交前必须全绿）：
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace -- --skip market::fetcher`（market::fetcher 打真实 API，flaky，固定跳过）
- git：直接提交 main；不跳 hooks（husky/lint-staged 会跑 `node ./scripts/backend-fmt.mjs` 校验 .rs，可能改写暂存文件，提交后复查 `git status`）；不擅自 force push/建分支；提交格式 `type(scope): 中文描述` + 详细中文 body。
- 环境坑：PowerShell `Set-Content -Encoding UTF8` 会写 BOM（写文件用 Write 工具）；`Edit` 的 oldString 需精确（fmt 会重排 import）；cargo 编译缓存必要时 `cargo clean -p`。

## 7. 已知待办/待定（非阻塞）

- `crates/infra/src/persistence/instance_registry.rs`：SQLite 中心化实例注册表，零外部调用（死代码，方向与去中心化设计相反）。**用户明确暂不动**。注意 `persistence/sqlite.rs` 必须保留（plugin/policy.rs、console.rs SqlValue、server/log/store.rs+writer.rs 在用）。
- `application/src/service/online.rs:448` 的 `get_app_data_dir` 用途未查。
- `backups/` 归属待定：集中 vs 跟实例走（本龙倾向跟走）。
- 配置 schema 重构（「加一个字段改 6 处」）独立课题。
- cron 服务改造（读 sl.json）独立任务。
- 前端契约：`config.ts` 走 `tauriInvoke`+`serverPath`，axum 端点未接，待阶段 7 后对齐。
- `platform/mod.rs` 的 `locations` 为私有模块：新增类型需显式 `pub use` 重导出。

## 8. 下一步（按当前优先级）

1. **阶段 6 阻塞**：P3-5 `SettingsManager::layout()`（`path.parent()`+`registry.main_resource_dir` 现构造）+ 各服务（`BackupManager`/`CorePluginService`/`services.rs` 等）改为注入 `AppLayout`，不再各自 `AppLayout::native(None)`。
2. **阶段 6**：`trust.rs` 三态分类 + `trust_instance` + 孤儿清理（D15）；`ignored_dirs` 匹配复用 `dir_key`。
3. **阶段 7**：application 接线 + cron 服务改造（`InstanceCronEntry`/`to_cron_task` 已备好）+ `instance.rs:34` 注释。
