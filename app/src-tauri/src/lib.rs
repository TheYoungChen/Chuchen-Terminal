use once_cell::sync::Lazy;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{MemoryRefreshKind, System};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceGitInfo {
    is_repo: bool,
    branch: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SystemStatusPayload {
    cpu: String,
    memory: String,
    gpu: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EnvironmentCheckItem {
    name: String,
    value: String,
    status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TerminalSessionOpened {
    session_id: String,
    shell_label: String,
    working_directory: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TerminalOutputPayload {
    session_id: String,
    chunk: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TerminalExitPayload {
    session_id: String,
    exit_code: i32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DetectedProviderProfile {
    provider_kind: String,
    name: String,
    profile_name: String,
    config_path: String,
    config_scope: String,
    managed_by: String,
    auth_source: String,
    switch_command: String,
    default_model: String,
    tool_targets: Vec<String>,
    status: String,
    is_active: bool,
    color: String,
    note: String,
    detected_source: String,
    config_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_stats: Option<DetectedProviderUsageStats>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DetectedProviderUsageStats {
    summary: DetectedProviderUsageSummary,
    trends: Vec<DetectedProviderUsageTrendPoint>,
    request_logs: Vec<DetectedProviderRequestLog>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DetectedProviderUsageSummary {
    total_requests: u64,
    total_cost_usd: f64,
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_cache_read_tokens: u64,
    total_cache_creation_tokens: u64,
    cache_hit_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DetectedProviderUsageTrendPoint {
    timestamp: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DetectedProviderRequestLog {
    id: String,
    app_type: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
    status_code: u16,
    duration_ms: u64,
    data_source: String,
    created_at: String,
}

#[derive(Clone)]
struct CcSwitchProviderEntry {
    app: String,
    provider_id: String,
    provider_name: String,
    is_current: bool,
    default_model: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CcSwitchDatabaseProvider {
    app: String,
    provider_id: String,
    provider_name: String,
    settings_config: String,
    is_current: bool,
    usage_stats: Option<DetectedProviderUsageStats>,
}

struct ManagedTerminalSession {
    writer: Box<dyn Write + Send>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    working_directory: String,
    shell_label: String,
}

type SessionStore = Arc<Mutex<HashMap<String, ManagedTerminalSession>>>;

static SESSION_STORE: Lazy<SessionStore> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
static SYSTEM_INFO: Lazy<Mutex<System>> = Lazy::new(|| {
    let mut system = System::new();
    system.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
    system.refresh_cpu_usage();
    Mutex::new(system)
});
static GPU_NAME_CACHE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static GPU_NAME_LOADING: AtomicBool = AtomicBool::new(false);

fn detect_gpu_name() -> String {
    if !cfg!(target_os = "windows") {
        return String::from("当前平台未接入");
    }

    let output = Command::new("powershell.exe")
    .args([
      "-NoProfile",
      "-Command",
      "$gpus = Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name; ($gpus | Where-Object { $_ -notmatch 'Virtual|Basic|Remote|GameViewer|Display Adapter' -and $_ -match 'NVIDIA|AMD|Radeon|Intel|Arc|GeForce|RTX|GTX' } | Select-Object -First 1) ?? ($gpus | Where-Object { $_ -notmatch 'Virtual|Basic|Remote|GameViewer|Display Adapter' } | Select-Object -First 1) ?? ($gpus | Select-Object -First 1)",
    ])
    .output();

    match output {
        Ok(result) if result.status.success() => {
            let value = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if value.is_empty() {
                String::from("未检测")
            } else {
                value
            }
        }
        _ => String::from("未检测"),
    }
}

fn read_cached_gpu_name() -> String {
    if let Ok(cache) = GPU_NAME_CACHE.lock() {
        if let Some(value) = cache.as_ref() {
            return value.clone();
        }
    }

    if !cfg!(target_os = "windows") {
        return String::from("当前平台未接入");
    }

    if GPU_NAME_LOADING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        thread::spawn(|| {
            let detected = detect_gpu_name();
            if let Ok(mut cache) = GPU_NAME_CACHE.lock() {
                *cache = Some(detected);
            }
            GPU_NAME_LOADING.store(false, Ordering::Release);
        });
    }

    String::from("检测中")
}

fn environment_check_names() -> Vec<&'static str> {
    vec![
        "Node.js",
        "Python",
        "Java",
        "Go",
        "Rust",
        "PowerShell",
        "Git / GitHub",
    ]
}

fn run_program_output_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().ok()?;
                if !status.success() {
                    return None;
                }

                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let value = if !stdout.is_empty() { stdout } else { stderr };

                return Some(if value.is_empty() {
                    String::from("已安装")
                } else {
                    value
                });
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(_) => return None,
        }
    }
}

fn run_program_output(program: &str, args: &[&str]) -> Option<String> {
    run_program_output_with_timeout(program, args, Duration::from_millis(1800))
}

fn run_powershell_output(script: &str) -> Option<String> {
    let timeout = Duration::from_millis(2600);
    run_program_output_with_timeout(
        "pwsh.exe",
        &["-NoLogo", "-NoProfile", "-Command", script],
        timeout,
    )
    .or_else(|| {
        run_program_output_with_timeout(
            "powershell.exe",
            &["-NoLogo", "-NoProfile", "-Command", script],
            timeout,
        )
    })
}

fn run_environment_check(name: &str) -> EnvironmentCheckItem {
    let output = match name {
        "Node.js" => run_program_output("node", &["-v"]),
        "Python" => run_program_output("python", &["--version"]),
        "Java" => run_program_output("java", &["-version"]),
        "Go" => run_program_output("go", &["version"]),
        "Rust" => run_program_output("rustc", &["--version"]),
        "PowerShell" => run_powershell_output("$PSVersionTable.PSVersion.ToString()"),
        "Git / GitHub" => run_program_output("git", &["--version"]),
        _ => None,
    };

    match output {
        Some(value) => EnvironmentCheckItem {
            name: name.to_string(),
            value: compact_env_value(
                name,
                if value.is_empty() {
                    "已安装"
                } else {
                    &value
                },
            ),
            status: String::from("ok"),
        },
        _ => EnvironmentCheckItem {
            name: name.to_string(),
            value: String::from("未安装"),
            status: String::from("pending"),
        },
    }
}

fn compact_env_value(name: &str, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::from("已安装");
    }

    match name {
        "Node.js" => trimmed.lines().next().unwrap_or(trimmed).to_string(),
        "Python" => trimmed.replace("Python ", "Python "),
        "Java" => {
            if let Some(version) = trimmed.lines().find(|line| line.contains("version")) {
                let normalized = version.replace('"', "");
                if normalized.contains("17.") {
                    return String::from("JDK 17");
                }
                if normalized.contains("21.") {
                    return String::from("JDK 21");
                }
                return normalized;
            }
            String::from("已安装")
        }
        "Go" => trimmed
            .lines()
            .next()
            .unwrap_or(trimmed)
            .replace("go version ", ""),
        "Rust" => trimmed
            .lines()
            .next()
            .unwrap_or(trimmed)
            .replace("rustc ", ""),
        "PowerShell" => format!("PowerShell {}", trimmed),
        "Git / GitHub" => trimmed
            .lines()
            .next()
            .unwrap_or(trimmed)
            .replace("git version ", "git "),
        _ => trimmed.lines().next().unwrap_or(trimmed).to_string(),
    }
}

fn normalize_working_directory(input: Option<String>) -> Result<PathBuf, String> {
    match input.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => {
            let path = PathBuf::from(&value);
            if path.is_dir() {
                Ok(path)
            } else {
                Err(format!("工作目录不存在：{}", value))
            }
        }
        _ => std::env::current_dir().map_err(|error| format!("无法获取当前目录：{error}")),
    }
}

fn display_path(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn display_home_path(path: &Path) -> String {
    if let Some(home) = home_dir() {
        if let Ok(relative) = path.strip_prefix(&home) {
            let suffix = relative.to_string_lossy().replace('\\', "/");
            return if suffix.is_empty() {
                String::from("~")
            } else {
                format!("~/{}", suffix)
            };
        }
    }

    path.to_string_lossy().to_string()
}

fn read_small_text_file(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 {
        return None;
    }

    std::fs::read_to_string(path).ok()
}

fn extract_json_string_field(raw: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\"", field);
    let start = raw.find(&needle)?;
    let after_field = raw[start + needle.len()..].find(':')? + start + needle.len() + 1;
    let rest = raw[after_field..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }

    let mut escaped = false;
    let mut value = String::new();
    for ch in rest[1..].chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Some(value);
        }
        value.push(ch);
    }

    None
}

fn extract_toml_string_field(raw: &str, field: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != field {
            continue;
        }

        let value = value.trim().trim_matches('"').trim_matches('\'').trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn cc_switch_config_dirs(home: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![home.join(".cc-switch")];

    #[cfg(windows)]
    if let Some(home_env) = std::env::var_os("HOME") {
        let legacy_dir = PathBuf::from(home_env).join(".cc-switch");
        if !dirs.contains(&legacy_dir) {
            dirs.push(legacy_dir);
        }
    }

    dirs
}

fn cc_switch_config_path(home: &Path) -> PathBuf {
    cc_switch_config_dirs(home)
        .into_iter()
        .map(|dir| dir.join("config.json"))
        .find(|path| path.exists())
        .unwrap_or_else(|| home.join(".cc-switch").join("config.json"))
}

fn cc_switch_database_paths(home: &Path) -> Vec<PathBuf> {
    cc_switch_config_dirs(home)
        .into_iter()
        .map(|dir| dir.join("cc-switch.db"))
        .collect()
}

fn cli_default_paths(home: &Path, provider_kind: &str) -> Vec<PathBuf> {
    match provider_kind {
        "codex" => vec![home.join(".codex").join("config.toml")],
        "claude-code" => vec![
            home.join(".claude.json"),
            home.join(".claude").join("settings.json"),
        ],
        "gemini-cli" => vec![
            home.join(".gemini").join("settings.json"),
            home.join(".gemini").join("config.json"),
        ],
        "opencode" => vec![
            home.join(".config").join("opencode").join("opencode.json"),
            home.join(".config").join("opencode").join("config.json"),
        ],
        _ => vec![home.join(".cc-switch").join("config.json")],
    }
}

fn provider_color(provider_kind: &str) -> &'static str {
    match provider_kind {
        "claude-code" => "#d97706",
        "gemini-cli" => "#0f9f6e",
        "opencode" => "#2563eb",
        "custom-cli" => "#475569",
        _ => "#4b83ff",
    }
}

fn provider_tool_target(provider_kind: &str) -> &'static str {
    match provider_kind {
        "claude-code" => "claude",
        "gemini-cli" => "gemini",
        "opencode" => "opencode",
        "custom-cli" => "generic",
        _ => "codex",
    }
}

fn provider_label(provider_kind: &str) -> &'static str {
    match provider_kind {
        "claude-code" => "Claude Code",
        "gemini-cli" => "Gemini CLI",
        "opencode" => "OpenCode",
        "custom-cli" => "自定义 CLI",
        _ => "Codex CLI",
    }
}

fn provider_switch_command(provider_kind: &str, profile_name: &str) -> String {
    match provider_kind {
        "claude-code" => format!("cc-switch claude use {profile_name}"),
        "gemini-cli" => format!("cc-switch gemini use {profile_name}"),
        "opencode" => format!("cc-switch opencode use {profile_name}"),
        "custom-cli" => format!("cc-switch use {profile_name}"),
        _ => format!("cc-switch codex use {profile_name}"),
    }
}

fn build_detected_provider(
    provider_kind: &str,
    name: String,
    profile_name: String,
    config_path: PathBuf,
    managed_by: &str,
    auth_source: String,
    default_model: String,
    is_active: bool,
    note: String,
    detected_source: &str,
) -> DetectedProviderProfile {
    let config_exists = config_path.exists();
    DetectedProviderProfile {
        provider_kind: provider_kind.to_string(),
        name,
        profile_name: profile_name.clone(),
        config_path: display_home_path(&config_path),
        config_scope: String::from("global"),
        managed_by: managed_by.to_string(),
        auth_source,
        switch_command: provider_switch_command(provider_kind, &profile_name),
        default_model,
        tool_targets: vec![provider_tool_target(provider_kind).to_string()],
        status: if is_active && config_exists {
            String::from("active")
        } else if config_exists {
            String::from("available")
        } else {
            String::from("missing")
        },
        is_active: is_active && config_exists,
        color: provider_color(provider_kind).to_string(),
        note,
        detected_source: detected_source.to_string(),
        config_exists,
        usage_stats: None,
    }
}

fn detect_default_cli_profile(home: &Path, provider_kind: &str) -> DetectedProviderProfile {
    let config_path = cli_default_paths(home, provider_kind)
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| match provider_kind {
            "codex" => home.join(".codex").join("config.toml"),
            "claude-code" => home.join(".claude.json"),
            "gemini-cli" => home.join(".gemini").join("settings.json"),
            "opencode" => home.join(".config").join("opencode").join("opencode.json"),
            _ => home.join(".cc-switch").join("config.json"),
        });
    let raw = read_small_text_file(&config_path).unwrap_or_default();
    let default_model = match provider_kind {
        "codex" => extract_toml_string_field(&raw, "model").unwrap_or_else(|| String::from("gpt-5")),
        "claude-code" => extract_json_string_field(&raw, "model").unwrap_or_else(|| String::from("claude-sonnet-4")),
        "gemini-cli" => extract_json_string_field(&raw, "model").unwrap_or_else(|| String::from("gemini-2.5-pro")),
        _ => extract_json_string_field(&raw, "model").unwrap_or_default(),
    };
    let has_sensitive_auth = raw.contains("api_key")
        || raw.contains("API_KEY")
        || raw.contains("auth_token")
        || raw.contains("access_token")
        || raw.contains("refresh_token");
    let auth_source = if config_path.exists() {
        if has_sensitive_auth {
            format!("{} 本地配置，检测到认证字段（未读取明文）", provider_label(provider_kind))
        } else {
            format!("{} 本地配置", provider_label(provider_kind))
        }
    } else {
        String::from("未检测到配置文件")
    };
    let note = if config_path.exists() {
        String::from("从 CLI 默认配置路径只读检测；导入和启用只更新 Chuchen-Terminal 的档案记录。")
    } else {
        String::from("未发现默认配置文件；保留路径方便后续创建或手动登记。")
    };

    build_detected_provider(
        provider_kind,
        format!("{} Default", provider_label(provider_kind)),
        String::from("default"),
        config_path,
        "cli-config",
        auth_source,
        default_model,
        false,
        note,
        "cli-config",
    )
}

fn kind_from_cc_switch_app(app: &str) -> Option<&'static str> {
    match app {
        "codex" => Some("codex"),
        "claude" | "claude-code" => Some("claude-code"),
        "gemini" => Some("gemini-cli"),
        "opencode" => Some("opencode"),
        _ => None,
    }
}

fn push_cc_switch_provider_entries_from_manager(
    entries: &mut Vec<CcSwitchProviderEntry>,
    app: &str,
    manager: &serde_json::Map<String, Value>,
) {
    let current = manager
        .get("current")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(providers) = manager.get("providers").and_then(Value::as_object) else {
        return;
    };

    for (provider_id, provider) in providers {
        let provider_name = provider
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(provider_id)
            .to_string();
        entries.push(CcSwitchProviderEntry {
            app: app.to_string(),
            provider_id: provider_id.to_string(),
            provider_name,
            is_current: !current.is_empty() && current == provider_id,
            default_model: extract_provider_model(provider),
        });
    }
}

fn extract_cc_switch_provider_entries(raw: &str) -> Vec<CcSwitchProviderEntry> {
    let Ok(root) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    let known = ["codex", "claude", "gemini", "opencode"];

    if let Some(apps) = root.get("apps").and_then(Value::as_object) {
        for app in known {
            let Some(manager) = apps.get(app).and_then(Value::as_object) else {
                continue;
            };
            push_cc_switch_provider_entries_from_manager(&mut entries, app, manager);
        }
    }

    if let Some(root_object) = root.as_object() {
        for app in known {
            let Some(manager) = root_object.get(app).and_then(Value::as_object) else {
                continue;
            };
            if manager.get("providers").is_some() {
                push_cc_switch_provider_entries_from_manager(&mut entries, app, manager);
            }
        }
    }

    if !entries.is_empty() {
        return entries;
    }

    if let (Some(providers), Some(current)) = (
        root.get("providers").and_then(Value::as_object),
        root.get("current").and_then(Value::as_str),
    ) {
        for (provider_id, provider) in providers {
            let provider_name = provider
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or(provider_id)
                .to_string();
            entries.push(CcSwitchProviderEntry {
                app: String::from("claude"),
                provider_id: provider_id.to_string(),
                provider_name,
                is_current: current == provider_id,
                default_model: extract_provider_model(provider),
            });
        }
    }

    entries
}

fn extract_provider_model(provider: &Value) -> String {
    let settings = provider.get("settingsConfig").unwrap_or(provider);
    for pointer in [
        "/env/ANTHROPIC_MODEL",
        "/env/ANTHROPIC_DEFAULT_SONNET_MODEL",
        "/env/GEMINI_MODEL",
        "/model",
        "/defaultModel",
    ] {
        if let Some(value) = settings.pointer(pointer).and_then(Value::as_str) {
            if !value.trim().is_empty() {
                return value.trim().to_string();
            }
        }
    }

    if let Some(config_text) = settings.get("config").and_then(Value::as_str) {
        if let Some(model) = extract_toml_string_field(config_text, "model") {
            return model;
        }
    }

    String::new()
}

fn extract_provider_model_from_settings_config(settings_config: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(settings_config) {
        let model = extract_provider_model(&value);
        if !model.is_empty() {
            return model;
        }
    }

    extract_toml_string_field(settings_config, "model").unwrap_or_default()
}

const CC_SWITCH_SQLITE_EXPORT_SCRIPT: &str = r#"
import json
import sqlite3
import sys

db_path = sys.argv[1]
conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
conn.row_factory = sqlite3.Row

def has_table(name):
    row = conn.execute(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1",
        (name,),
    ).fetchone()
    return row is not None

def table_columns(name):
    if not has_table(name):
        return set()
    return {row["name"] for row in conn.execute(f"PRAGMA table_info({name})")}

provider_columns = table_columns("providers")
log_columns = table_columns("proxy_request_logs")

def provider_expr(name, fallback):
    return name if name in provider_columns else fallback

def log_expr(name, fallback):
    return name if name in log_columns else fallback

def log_time_expr(name, fallback="''"):
    if name not in log_columns:
        return fallback
    return (
        f"CASE "
        f"WHEN {name} IS NULL THEN '' "
        f"WHEN typeof({name}) IN ('integer', 'real') THEN strftime('%Y-%m-%dT%H:%M:%SZ', {name}, 'unixepoch') "
        f"ELSE CAST({name} AS TEXT) "
        f"END"
    )

def log_day_expr(name):
    if name not in log_columns:
        return None
    return (
        f"CASE "
        f"WHEN {name} IS NULL THEN '' "
        f"WHEN typeof({name}) IN ('integer', 'real') THEN strftime('%Y-%m-%dT00:00:00Z', {name}, 'unixepoch') "
        f"ELSE substr(CAST({name} AS TEXT), 1, 10) || 'T00:00:00Z' "
        f"END"
    )

def number(value):
    try:
        return float(value or 0)
    except Exception:
        return 0

def integer(value):
    try:
        return max(0, int(value or 0))
    except Exception:
        return 0

def hit_rate(input_tokens, cache_read_tokens, cache_creation_tokens):
    cacheable = input_tokens + cache_read_tokens + cache_creation_tokens
    return cache_read_tokens / cacheable if cacheable else 0

def usage_stats(app, provider_id):
    if not has_table("proxy_request_logs"):
        return None
    if "provider_id" not in log_columns or "app_type" not in log_columns:
        return None

    summary_row = conn.execute(
        f"""
        SELECT
            COUNT(*) AS total_requests,
            COALESCE(SUM({log_expr("input_tokens", "0")}), 0) AS input_tokens,
            COALESCE(SUM({log_expr("output_tokens", "0")}), 0) AS output_tokens,
            COALESCE(SUM({log_expr("cache_read_tokens", "0")}), 0) AS cache_read_tokens,
            COALESCE(SUM({log_expr("cache_creation_tokens", "0")}), 0) AS cache_creation_tokens,
            COALESCE(SUM(CAST({log_expr("total_cost_usd", "0")} AS REAL)), 0) AS total_cost_usd
        FROM proxy_request_logs
        WHERE provider_id = ? AND app_type = ?
        """,
        (provider_id, app),
    ).fetchone()

    input_tokens = integer(summary_row["input_tokens"])
    cache_read_tokens = integer(summary_row["cache_read_tokens"])
    cache_creation_tokens = integer(summary_row["cache_creation_tokens"])
    summary = {
        "totalRequests": integer(summary_row["total_requests"]),
        "totalCostUsd": number(summary_row["total_cost_usd"]),
        "totalInputTokens": input_tokens,
        "totalOutputTokens": integer(summary_row["output_tokens"]),
        "totalCacheReadTokens": cache_read_tokens,
        "totalCacheCreationTokens": cache_creation_tokens,
        "cacheHitRate": hit_rate(input_tokens, cache_read_tokens, cache_creation_tokens),
    }

    day_expr = log_day_expr("created_at")
    trends = []
    if day_expr:
        trends = [
            {
                "timestamp": row["timestamp"],
                "inputTokens": integer(row["input_tokens"]),
                "outputTokens": integer(row["output_tokens"]),
                "cacheReadTokens": integer(row["cache_read_tokens"]),
                "cacheCreationTokens": integer(row["cache_creation_tokens"]),
                "costUsd": number(row["cost_usd"]),
            }
            for row in conn.execute(
                f"""
                SELECT *
                FROM (
                    SELECT
                        {day_expr} AS timestamp,
                        COALESCE(SUM({log_expr("input_tokens", "0")}), 0) AS input_tokens,
                        COALESCE(SUM({log_expr("output_tokens", "0")}), 0) AS output_tokens,
                        COALESCE(SUM({log_expr("cache_read_tokens", "0")}), 0) AS cache_read_tokens,
                        COALESCE(SUM({log_expr("cache_creation_tokens", "0")}), 0) AS cache_creation_tokens,
                        COALESCE(SUM(CAST({log_expr("total_cost_usd", "0")} AS REAL)), 0) AS cost_usd
                    FROM proxy_request_logs
                    WHERE provider_id = ? AND app_type = ?
                    GROUP BY timestamp
                    ORDER BY timestamp DESC
                    LIMIT 14
                )
                ORDER BY timestamp ASC
                """,
                (provider_id, app),
            )
        ]

    request_id_expr = log_expr("request_id", "rowid")
    request_order_expr = "created_at DESC" if "created_at" in log_columns else "rowid DESC"

    request_logs = [
        {
            "id": row["id"],
            "appType": row["app_type"],
            "model": row["model"] or "unknown",
            "inputTokens": integer(row["input_tokens"]),
            "outputTokens": integer(row["output_tokens"]),
            "cacheReadTokens": integer(row["cache_read_tokens"]),
            "cacheCreationTokens": integer(row["cache_creation_tokens"]),
            "costUsd": number(row["total_cost_usd"]),
            "statusCode": integer(row["status_code"]),
            "durationMs": integer(row["duration_ms"]),
            "dataSource": row["data_source"] or "proxy",
            "createdAt": row["created_at"],
        }
        for row in conn.execute(
            f"""
            SELECT
                CAST({request_id_expr} AS TEXT) AS id,
                app_type,
                {log_expr("model", "'unknown'")} AS model,
                {log_expr("input_tokens", "0")} AS input_tokens,
                {log_expr("output_tokens", "0")} AS output_tokens,
                {log_expr("cache_read_tokens", "0")} AS cache_read_tokens,
                {log_expr("cache_creation_tokens", "0")} AS cache_creation_tokens,
                {log_expr("total_cost_usd", "0")} AS total_cost_usd,
                {log_expr("status_code", "0")} AS status_code,
                COALESCE({log_expr("duration_ms", "NULL")}, {log_expr("latency_ms", "0")}, 0) AS duration_ms,
                COALESCE({log_expr("data_source", "NULL")}, 'proxy') AS data_source,
                {log_time_expr("created_at")} AS created_at
            FROM proxy_request_logs
            WHERE provider_id = ? AND app_type = ?
            ORDER BY {request_order_expr}
            LIMIT 80
            """,
            (provider_id, app),
        )
    ]

    return {
        "summary": summary,
        "trends": trends,
        "requestLogs": request_logs,
    }

providers = []
if has_table("providers") and "id" in provider_columns and "app_type" in provider_columns:
    provider_name_expr = provider_expr("name", "id")
    provider_settings_expr = provider_expr("settings_config", "'{}'")
    provider_current_expr = provider_expr("is_current", "0")
    provider_order_parts = ["app_type"]
    if "sort_index" in provider_columns:
        provider_order_parts.append("COALESCE(sort_index, 0)")
    provider_order_parts.append("provider_name")
    for row in conn.execute(
        f"""
        SELECT
            id,
            app_type,
            {provider_name_expr} AS provider_name,
            {provider_settings_expr} AS settings_config,
            {provider_current_expr} AS is_current
        FROM providers
        WHERE app_type IN ('claude', 'codex', 'gemini', 'opencode')
        ORDER BY {", ".join(provider_order_parts)}
        """
    ):
        providers.append({
            "providerId": row["id"],
            "app": row["app_type"],
            "providerName": row["provider_name"],
            "settingsConfig": row["settings_config"] or "{}",
            "isCurrent": bool(row["is_current"]),
            "usageStats": usage_stats(row["app_type"], row["id"]),
        })

print(json.dumps(providers, ensure_ascii=False))
"#;

fn export_cc_switch_database_with_python(db_path: &Path) -> Option<Vec<CcSwitchDatabaseProvider>> {
    if !db_path.exists() {
        return None;
    }

    let db_path_text = db_path.to_string_lossy().to_string();
    let attempts: [(&str, Vec<&str>); 3] = [
        ("python", vec!["-c"]),
        ("py", vec!["-3", "-c"]),
        ("python3", vec!["-c"]),
    ];

    for (program, mut args) in attempts {
        args.push(CC_SWITCH_SQLITE_EXPORT_SCRIPT);
        args.push(&db_path_text);

        let output = Command::new(program)
            .args(args)
            .env("PYTHONIOENCODING", "utf-8")
            .output();

        let Ok(output) = output else {
            continue;
        };
        if !output.status.success() {
            continue;
        }

        if let Ok(rows) = serde_json::from_slice::<Vec<CcSwitchDatabaseProvider>>(&output.stdout) {
            return Some(rows);
        }
    }

    None
}

fn read_cc_switch_database_providers(db_path: &Path) -> Vec<DetectedProviderProfile> {
    if !db_path.exists() {
        return Vec::new();
    }

    let Some(rows) = export_cc_switch_database_with_python(db_path) else {
        return Vec::new();
    };

    rows.into_iter()
        .filter_map(|entry| {
            let provider_kind = kind_from_cc_switch_app(&entry.app)?;
            let profile_name = if entry.provider_id.trim().is_empty() {
                String::from("default")
            } else {
                entry.provider_id.trim().to_string()
            };
            let mut profile = build_detected_provider(
                provider_kind,
                entry.provider_name,
                profile_name,
                db_path.to_path_buf(),
                "cc-switch",
                String::from("CC Switch SQLite 档案（未读取密钥明文）"),
                extract_provider_model_from_settings_config(&entry.settings_config),
                entry.is_current,
                format!(
                    "从 {} 的 providers 表只读发现；使用统计来自 proxy_request_logs。",
                    display_home_path(db_path)
                ),
                "cc-switch-db",
            );
            profile.usage_stats = entry.usage_stats;
            Some(profile)
        })
        .collect()
}

fn remove_session(session_id: &str) {
    if let Ok(mut sessions) = SESSION_STORE.lock() {
        sessions.remove(session_id);
    }
}

fn take_session(session_id: &str) -> Option<ManagedTerminalSession> {
    SESSION_STORE
        .lock()
        .ok()
        .and_then(|mut sessions| sessions.remove(session_id))
}

fn cleanup_session_async(session: ManagedTerminalSession) {
    thread::spawn(move || {
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        drop(session);
    });
}

fn emit_output(app: &AppHandle, session_id: &str, chunk: String) {
    let _ = app.emit(
        "terminal://output",
        TerminalOutputPayload {
            session_id: session_id.to_string(),
            chunk,
        },
    );
}

fn emit_exit(app: &AppHandle, session_id: &str, exit_code: i32) {
    let _ = app.emit(
        "terminal://exit",
        TerminalExitPayload {
            session_id: session_id.to_string(),
            exit_code,
        },
    );
}

#[tauri::command]
fn ping_tauri() -> String {
    format!(
        "Rust/Tauri 已连通 · {} · {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )
}

#[tauri::command]
fn open_terminal_session(
    app: AppHandle,
    session_id: String,
    working_directory: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<TerminalSessionOpened, String> {
    if let Ok(sessions) = SESSION_STORE.lock() {
        if let Some(existing) = sessions.get(&session_id) {
            return Ok(TerminalSessionOpened {
                session_id,
                shell_label: existing.shell_label.clone(),
                working_directory: existing.working_directory.clone(),
            });
        }
    }

    let working_directory = normalize_working_directory(working_directory)?;
    let working_directory_label = display_path(&working_directory);

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: rows.max(10),
            cols: cols.max(20),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("无法创建 PTY：{error}"))?;

    let mut command = if cfg!(target_os = "windows") {
        CommandBuilder::new("pwsh.exe")
    } else {
        CommandBuilder::new("bash")
    };
    command.cwd(working_directory.clone());
    if cfg!(target_os = "windows") {
        command.arg("-NoLogo");
        command.arg("-NoProfile");
        command.env("POWERSHELL_UPDATECHECK", "Off");
    }

    let child = match pair.slave.spawn_command(command) {
        Ok(child) => child,
        Err(error) => {
            let mut fallback = CommandBuilder::new("powershell.exe");
            fallback.cwd(working_directory.clone());
            fallback.arg("-NoLogo");
            fallback.arg("-NoProfile");
            pair.slave
                .spawn_command(fallback)
                .map_err(|fallback_error| {
                    format!("无法启动终端进程：{fallback_error}; fallback: {error}")
                })?
        }
    };

    let shell_label = if cfg!(target_os = "windows") {
        String::from("PowerShell 7")
    } else {
        String::from("Bash")
    };

    let child = Arc::new(Mutex::new(child));

    let reader_session_id = session_id.clone();
    let reader_app = app.clone();
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| format!("无法创建 PTY 读取器：{error}"))?;

    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    break;
                }
                Ok(size) => {
                    let chunk = String::from_utf8_lossy(&buffer[..size]).to_string();
                    emit_output(&reader_app, &reader_session_id, chunk);
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    let waiter_session_id = session_id.clone();
    let waiter_app = app.clone();
    let waiter_child = Arc::clone(&child);
    thread::spawn(move || {
        let exit_code = if let Ok(mut child) = waiter_child.lock() {
            match child.wait() {
                Ok(status) => status.exit_code() as i32,
                Err(_) => -1,
            }
        } else {
            -1
        };

        emit_exit(&waiter_app, &waiter_session_id, exit_code);
        remove_session(&waiter_session_id);
    });

    let writer = pair
        .master
        .take_writer()
        .map_err(|error| format!("无法获取 PTY 写入器：{error}"))?;

    let session = ManagedTerminalSession {
        writer,
        child,
        master: pair.master,
        working_directory: working_directory_label.clone(),
        shell_label: shell_label.clone(),
    };

    let mut sessions = SESSION_STORE
        .lock()
        .map_err(|_| String::from("会话状态锁定失败。"))?;
    sessions.insert(session_id.clone(), session);

    Ok(TerminalSessionOpened {
        session_id,
        shell_label,
        working_directory: working_directory_label,
    })
}

#[tauri::command]
fn write_terminal_input(session_id: String, input: String) -> Result<(), String> {
    let mut sessions = SESSION_STORE
        .lock()
        .map_err(|_| String::from("会话状态锁定失败。"))?;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| String::from("终端会话不存在。"))?;

    session
        .writer
        .write_all(input.as_bytes())
        .map_err(|error| format!("写入终端失败：{error}"))?;
    session
        .writer
        .flush()
        .map_err(|error| format!("刷新终端失败：{error}"))
}

#[tauri::command]
fn resize_terminal(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    let mut sessions = SESSION_STORE
        .lock()
        .map_err(|_| String::from("会话状态锁定失败。"))?;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| String::from("终端会话不存在。"))?;

    session
        .master
        .resize(PtySize {
            rows: rows.max(10),
            cols: cols.max(20),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("调整终端尺寸失败：{error}"))
}

#[tauri::command]
fn close_terminal_session(session_id: String) -> Result<(), String> {
    if let Some(session) = take_session(&session_id) {
        cleanup_session_async(session);
    }
    Ok(())
}

#[tauri::command]
async fn read_workspace_git_info(path: String) -> Result<WorkspaceGitInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let target = PathBuf::from(path.trim());
        if !target.exists() {
            return Err(String::from("工作目录不存在。"));
        }

        let output = Command::new("git")
            .arg("-C")
            .arg(&target)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let branch = String::from_utf8_lossy(&result.stdout).trim().to_string();
                Ok(WorkspaceGitInfo {
                    is_repo: true,
                    branch: if branch.is_empty() {
                        None
                    } else {
                        Some(branch)
                    },
                })
            }
            Ok(_) => Ok(WorkspaceGitInfo {
                is_repo: false,
                branch: None,
            }),
            Err(error) => Err(format!("读取 Git 信息失败：{error}")),
        }
    })
    .await
    .map_err(|error| format!("读取 Git 信息任务失败：{error}"))?
}

#[tauri::command]
async fn read_system_status() -> Result<SystemStatusPayload, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (cpu, memory) = if let Ok(mut system) = SYSTEM_INFO.lock() {
            system.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
            system.refresh_cpu_usage();

            let cpu = format!("{:.0}%", system.global_cpu_info().cpu_usage());
            let used_memory_gb = system.used_memory() as f64 / 1024_f64 / 1024_f64 / 1024_f64;
            let total_memory_gb = system.total_memory() as f64 / 1024_f64 / 1024_f64 / 1024_f64;
            let memory = format!("{:.1}/{:.1} GB", used_memory_gb, total_memory_gb);
            (cpu, memory)
        } else {
            let mut fallback = System::new();
            fallback.refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
            fallback.refresh_cpu_usage();
            let cpu = format!("{:.0}%", fallback.global_cpu_info().cpu_usage());
            let used_memory_gb = fallback.used_memory() as f64 / 1024_f64 / 1024_f64 / 1024_f64;
            let total_memory_gb = fallback.total_memory() as f64 / 1024_f64 / 1024_f64 / 1024_f64;
            let memory = format!("{:.1}/{:.1} GB", used_memory_gb, total_memory_gb);
            (cpu, memory)
        };

        let gpu = read_cached_gpu_name();

        Ok(SystemStatusPayload { cpu, memory, gpu })
    })
    .await
    .map_err(|error| format!("读取系统状态任务失败：{error}"))?
}

#[tauri::command]
async fn read_environment_checks() -> Result<Vec<EnvironmentCheckItem>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut items = Vec::new();

        if cfg!(target_os = "windows") {
            for name in environment_check_names() {
                items.push(run_environment_check(name));
            }
        }

        Ok(items)
    })
    .await
    .map_err(|error| format!("读取环境检测任务失败：{error}"))?
}

#[tauri::command]
async fn read_environment_check(name: String) -> Result<EnvironmentCheckItem, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(String::from("环境项名称不能为空。"));
        }

        let spec = environment_check_names()
            .into_iter()
            .find(|item_name| *item_name == trimmed)
            .ok_or_else(|| format!("未注册的环境项：{trimmed}"))?;

        Ok(run_environment_check(spec))
    })
    .await
    .map_err(|error| format!("读取环境项任务失败：{error}"))?
}

#[tauri::command]
async fn detect_local_provider_profiles() -> Result<Vec<DetectedProviderProfile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let mut profiles = Vec::new();
        let cc_switch_path = cc_switch_config_path(&home);
        let mut found_cc_switch_database_profiles = false;

        for db_path in cc_switch_database_paths(&home) {
            let mut database_profiles = read_cc_switch_database_providers(&db_path);
            if !database_profiles.is_empty() {
                found_cc_switch_database_profiles = true;
                profiles.append(&mut database_profiles);
                break;
            }
        }

        if !found_cc_switch_database_profiles {
            if let Some(raw) = read_small_text_file(&cc_switch_path) {
            for entry in extract_cc_switch_provider_entries(&raw) {
                let Some(provider_kind) = kind_from_cc_switch_app(&entry.app) else {
                    continue;
                };
                let profile_name = if entry.provider_id.trim().is_empty() {
                    String::from("default")
                } else {
                    entry.provider_id.trim().to_string()
                };
                profiles.push(build_detected_provider(
                    provider_kind,
                    entry.provider_name,
                    profile_name,
                    cc_switch_path.clone(),
                    "cc-switch",
                    String::from("CC Switch 本地配置档案"),
                    entry.default_model,
                    entry.is_current,
                    format!(
                        "从 {} 只读发现。Chuchen 仅登记档案和切换命令，不写入真实配置。",
                        display_home_path(&cc_switch_path)
                    ),
                    "cc-switch",
                ));
            }
            }
        }

        let existing_cli_kinds = profiles
            .iter()
            .map(|profile| profile.provider_kind.clone())
            .collect::<std::collections::HashSet<_>>();

        for provider_kind in ["codex", "claude-code", "gemini-cli", "opencode"] {
            let detected = detect_default_cli_profile(&home, provider_kind);
            if detected.config_exists || !existing_cli_kinds.contains(provider_kind) {
                profiles.push(detected);
            }
        }

        let mut seen = std::collections::HashSet::new();
        profiles.retain(|profile| {
            let key = format!(
                "{}:{}:{}",
                profile.provider_kind, profile.profile_name, profile.config_path
            );
            seen.insert(key)
        });

        Ok(profiles)
    })
    .await
    .map_err(|error| format!("扫描 Provider 配置任务失败：{error}"))?
}

#[tauri::command]
fn open_directory(path: String) -> Result<(), String> {
    let target = PathBuf::from(path.trim());
    if !target.exists() {
        return Err(String::from("目录不存在。"));
    }

    if cfg!(target_os = "windows") {
        Command::new("explorer.exe")
            .arg(target)
            .spawn()
            .map_err(|error| format!("打开目录失败：{error}"))?;
        return Ok(());
    }

    Err(String::from("当前平台暂未实现打开目录。"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            ping_tauri,
            open_terminal_session,
            write_terminal_input,
            resize_terminal,
            close_terminal_session,
            open_directory,
            read_workspace_git_info,
            read_system_status,
            read_environment_check,
            read_environment_checks,
            detect_local_provider_profiles,
        ])
        .setup(|app| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.set_zoom(1.0);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
