use chrono::Datelike;
use once_cell::sync::Lazy;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
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
    identity_key: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_payload: Option<String>,
    detected_source: String,
    config_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_stats: Option<DetectedProviderUsageStats>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderReconciliationInput {
    id: String,
    #[serde(default)]
    identity_key: Option<String>,
    provider_kind: String,
    name: String,
    profile_name: String,
    managed_by: String,
    #[serde(default)]
    default_model: String,
    #[serde(default)]
    request_base_url: Option<String>,
    #[serde(default)]
    config_payload: Option<String>,
    #[serde(default)]
    auth_payload: Option<String>,
    #[serde(default)]
    updated_at: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderReconciliationAction {
    existing_id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_identity_key: Option<String>,
    reason_code: String,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct ProviderReconciliationSummary {
    existing_count: usize,
    canonical_count: usize,
    add_count: usize,
    update_count: usize,
    remove_duplicate_count: usize,
    remove_stale_count: usize,
    preserve_count: usize,
    review_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderReconciliationPreview {
    summary: ProviderReconciliationSummary,
    canonical_profiles: Vec<DetectedProviderProfile>,
    actions: Vec<ProviderReconciliationAction>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AppliedProviderProfileResult {
    provider_kind: String,
    profile_name: String,
    applied_command: String,
    stdout: String,
    stderr: String,
    requires_restart: bool,
    restart_hint: String,
    affected_session_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedMcpServer {
    id: String,
    name: String,
    app_targets: Vec<String>,
    transport_type: String,
    config: Value,
    source_kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedPromptProfile {
    tool_kind: String,
    name: String,
    prompt_file_path: String,
    content: String,
    source_kind: String,
    is_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsagePricing {
    input_cost_per_million: f64,
    output_cost_per_million: f64,
    cache_read_cost_per_million: f64,
    cache_creation_cost_per_million: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedModelPricingEntry {
    model_id: String,
    display_name: String,
    vendor: String,
    input_cost_per_million: f64,
    output_cost_per_million: f64,
    cache_read_cost_per_million: f64,
    cache_creation_cost_per_million: f64,
    completion_cost_per_million: f64,
    image_cost_per_million: f64,
    video_cost_per_million: f64,
    audio_cost_per_million: f64,
    source: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageRequestLog {
    id: String,
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    pricing_model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    input_cost_usd: f64,
    output_cost_usd: f64,
    cache_read_cost_usd: f64,
    cache_creation_cost_usd: f64,
    total_cost_usd: f64,
    first_token_ms: Option<u64>,
    duration_ms: Option<u64>,
    status_code: u16,
    data_source: String,
    created_at: String,
    pricing: Option<ManagedUsagePricing>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageChannelStat {
    channel_id: String,
    app_type: String,
    data_source: String,
    total_requests: u64,
    total_cost_usd: f64,
    total_input_tokens: u64,
    total_output_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageTrendPoint {
    timestamp: String,
    request_count: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageProviderStat {
    provider_id: String,
    provider_name: String,
    app_type: String,
    total_requests: u64,
    total_cost_usd: f64,
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_cache_read_tokens: u64,
    total_cache_creation_tokens: u64,
    cache_hit_rate: f64,
    model_count: usize,
    models: Vec<String>,
    data_sources: Vec<String>,
    last_activity_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageModelStat {
    model: String,
    total_requests: u64,
    total_cost_usd: f64,
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_cache_read_tokens: u64,
    total_cache_creation_tokens: u64,
    provider_count: usize,
    provider_names: Vec<String>,
    app_types: Vec<String>,
    last_activity_at: String,
}

struct ManagedUsageProviderAccumulator {
    stat: ManagedUsageProviderStat,
    models: std::collections::BTreeSet<String>,
    data_sources: std::collections::BTreeSet<String>,
}

struct ManagedUsageModelAccumulator {
    stat: ManagedUsageModelStat,
    provider_names: std::collections::BTreeSet<String>,
    app_types: std::collections::BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageCursor {
    created_at: String,
    id: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageQueryFilters {
    app_type: Option<String>,
    provider_name: Option<String>,
    provider_ids: Option<Vec<String>>,
    model: Option<String>,
    data_source: Option<String>,
    start_at: Option<String>,
    end_at: Option<String>,
    bucket: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ManagedUsageQueryResult {
    summary: DetectedProviderUsageSummary,
    channels: Vec<ManagedUsageChannelStat>,
    trends: Vec<ManagedUsageTrendPoint>,
    provider_stats: Vec<ManagedUsageProviderStat>,
    model_stats: Vec<ManagedUsageModelStat>,
    request_logs: Vec<ManagedUsageRequestLog>,
    next_cursor: Option<String>,
    has_more: bool,
    total: usize,
    updated_at: String,
}

const MANAGED_USAGE_DEDUP_WINDOW_SECONDS: i64 = 10 * 60;

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
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
    status_code: u16,
    first_token_ms: Option<u64>,
    duration_ms: u64,
    data_source: String,
    created_at: String,
}

#[derive(Clone, Copy)]
struct LocalModelPricing {
    model_id: &'static str,
    input_cost_per_million: f64,
    output_cost_per_million: f64,
    cache_read_cost_per_million: f64,
    cache_creation_cost_per_million: f64,
}

const LOCAL_MODEL_PRICING: &[LocalModelPricing] = &[
    LocalModelPricing {
        model_id: "claude-opus-4-8",
        input_cost_per_million: 5.0,
        output_cost_per_million: 25.0,
        cache_read_cost_per_million: 0.50,
        cache_creation_cost_per_million: 6.25,
    },
    LocalModelPricing {
        model_id: "claude-opus-4-6-20260206",
        input_cost_per_million: 5.0,
        output_cost_per_million: 25.0,
        cache_read_cost_per_million: 0.50,
        cache_creation_cost_per_million: 6.25,
    },
    LocalModelPricing {
        model_id: "claude-sonnet-4-6-20260217",
        input_cost_per_million: 3.0,
        output_cost_per_million: 15.0,
        cache_read_cost_per_million: 0.30,
        cache_creation_cost_per_million: 3.75,
    },
    LocalModelPricing {
        model_id: "claude-sonnet-4-5-20250929",
        input_cost_per_million: 3.0,
        output_cost_per_million: 15.0,
        cache_read_cost_per_million: 0.30,
        cache_creation_cost_per_million: 3.75,
    },
    LocalModelPricing {
        model_id: "claude-haiku-4-5-20251001",
        input_cost_per_million: 1.0,
        output_cost_per_million: 5.0,
        cache_read_cost_per_million: 0.10,
        cache_creation_cost_per_million: 1.25,
    },
    LocalModelPricing {
        model_id: "gpt-5.5",
        input_cost_per_million: 5.0,
        output_cost_per_million: 30.0,
        cache_read_cost_per_million: 0.50,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gpt-5.4",
        input_cost_per_million: 2.50,
        output_cost_per_million: 15.0,
        cache_read_cost_per_million: 0.25,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gpt-5.4-mini",
        input_cost_per_million: 0.75,
        output_cost_per_million: 4.50,
        cache_read_cost_per_million: 0.075,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gpt-5.4-nano",
        input_cost_per_million: 0.20,
        output_cost_per_million: 1.25,
        cache_read_cost_per_million: 0.02,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gpt-5.2",
        input_cost_per_million: 1.75,
        output_cost_per_million: 14.0,
        cache_read_cost_per_million: 0.175,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gpt-5.2-codex",
        input_cost_per_million: 1.75,
        output_cost_per_million: 14.0,
        cache_read_cost_per_million: 0.175,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gemini-3.5-flash",
        input_cost_per_million: 1.50,
        output_cost_per_million: 9.0,
        cache_read_cost_per_million: 0.15,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gemini-3-pro-preview",
        input_cost_per_million: 2.0,
        output_cost_per_million: 12.0,
        cache_read_cost_per_million: 0.20,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gemini-2.5-pro",
        input_cost_per_million: 1.25,
        output_cost_per_million: 10.0,
        cache_read_cost_per_million: 0.125,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gemini-2.5-flash",
        input_cost_per_million: 0.30,
        output_cost_per_million: 2.50,
        cache_read_cost_per_million: 0.03,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "gemini-2.5-flash-lite",
        input_cost_per_million: 0.10,
        output_cost_per_million: 0.40,
        cache_read_cost_per_million: 0.01,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "glm-5.1",
        input_cost_per_million: 1.4,
        output_cost_per_million: 4.4,
        cache_read_cost_per_million: 0.26,
        cache_creation_cost_per_million: 0.0,
    },
    LocalModelPricing {
        model_id: "glm-5.2",
        input_cost_per_million: 1.4,
        output_cost_per_million: 4.4,
        cache_read_cost_per_million: 0.26,
        cache_creation_cost_per_million: 0.0,
    },
];

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CcSwitchUsageRow {
    id: String,
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    pricing_model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    input_cost_usd: f64,
    output_cost_usd: f64,
    cache_read_cost_usd: f64,
    cache_creation_cost_usd: f64,
    total_cost_usd: f64,
    first_token_ms: Option<u64>,
    duration_ms: Option<u64>,
    status_code: u16,
    data_source: String,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CcSwitchUsageAggregateRow {
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    request_count: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    total_cost_usd: f64,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CcSwitchUsageExport {
    logs: Vec<CcSwitchUsageRow>,
    rollups: Vec<CcSwitchUsageAggregateRow>,
}

#[derive(Clone)]
struct ManagedUsageAggregateRow {
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    request_count: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    total_cost_usd: f64,
    data_source: String,
    created_at: String,
}

#[derive(Default)]
struct CcSwitchUsageData {
    logs: Vec<ManagedUsageRequestLog>,
    rollups: Vec<ManagedUsageAggregateRow>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HermesUsageRow {
    id: String,
    provider_id: String,
    provider_name: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    duration_ms: Option<u64>,
    status_code: u16,
    created_at: String,
}

struct ManagedTerminalSession {
    writer: Box<dyn Write + Send>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    working_directory: String,
    shell_label: String,
}

struct ManagedUsageLogCache {
    key: String,
    refreshed_at: Instant,
    logs: Vec<ManagedUsageRequestLog>,
    rollups: Vec<ManagedUsageAggregateRow>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NativeUsageSnapshot {
    files: BTreeMap<String, NativeUsageFileSnapshot>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NativeUsageFileSnapshot {
    size: u64,
    modified_nanos: u128,
    #[serde(default)]
    offset: u64,
    #[serde(default)]
    offset_probe: Vec<u8>,
    #[serde(default)]
    parser_state: Option<NativeUsageParserState>,
    logs: Vec<ManagedUsageRequestLog>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "state", rename_all = "snake_case")]
enum NativeUsageParserState {
    Codex(CodexNativeParseState),
    Claude(ClaudeNativeParseState),
    Gemini { message_count: usize },
}

#[derive(Default)]
struct NativeUsageScanStats {
    reused_files: usize,
    tail_parsed_files: usize,
    reset_files: usize,
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
static MANAGED_USAGE_LOG_CACHE: Lazy<Mutex<Option<ManagedUsageLogCache>>> =
    Lazy::new(|| Mutex::new(None));
static NATIVE_USAGE_SNAPSHOT_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static MODEL_PRICING_OVERRIDES: Lazy<Mutex<HashMap<String, ManagedModelPricingEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static MODEL_PRICING_LOADED: AtomicBool = AtomicBool::new(false);

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

fn normalize_model_id_for_pricing(raw: &str) -> String {
    raw.rsplit_once('/')
        .map_or(raw, |(_, tail)| tail)
        .split(':')
        .next()
        .unwrap_or(raw)
        .trim()
        .replace('@', "-")
        .to_ascii_lowercase()
        .trim_end_matches("[1m]")
        .trim()
        .to_string()
}

fn push_unique_model_candidate(candidates: &mut Vec<String>, candidate: String) -> bool {
    if candidate.is_empty() || candidates.iter().any(|existing| existing == &candidate) {
        return false;
    }
    candidates.push(candidate);
    true
}

fn strip_known_model_namespace(model_id: &str) -> Option<String> {
    if let Some(position) = model_id.rfind("claude-") {
        if position > 0 {
            return Some(model_id[position..].to_string());
        }
    }

    for marker in [
        "openai.",
        "anthropic.",
        "google.",
        "moonshot.",
        "moonshotai.",
        "bedrock.",
        "global.",
    ] {
        if let Some(stripped) = model_id.strip_prefix(marker) {
            return Some(stripped.to_string());
        }
    }
    None
}

fn strip_claude_desktop_non_anthropic_prefix(model_id: &str) -> Option<String> {
    const MARKERS: &[&str] = &[
        "abab",
        "ark-code",
        "arctic",
        "astron",
        "codex",
        "command-r",
        "deepseek",
        "doubao",
        "ernie",
        "gemini",
        "gemma",
        "glm",
        "gpt",
        "grok",
        "hermes",
        "hy3",
        "hunyuan",
        "jamba",
        "kimi",
        "lfm",
        "llama",
        "longcat",
        "mercury",
        "mimo",
        "minimax",
        "mistral",
        "mixtral",
        "moonshot",
        "nemotron",
        "nova-",
        "openai",
        "qianfan",
        "qwen",
        "seed-",
        "solar",
        "stepfun",
    ];
    let rest = model_id.strip_prefix("claude-")?;
    MARKERS
        .iter()
        .any(|marker| rest.starts_with(marker))
        .then(|| rest.to_string())
}

fn strip_bedrock_model_version_suffix(model_id: &str) -> Option<String> {
    let (base, suffix) = model_id.rsplit_once("-v")?;
    (!base.is_empty() && !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
        .then(|| base.to_string())
}

fn strip_model_date_suffix(model_id: &str) -> Option<String> {
    let bytes = model_id.as_bytes();
    if bytes.len() > 11 {
        let start = bytes.len() - 11;
        let suffix = &bytes[start..];
        let is_iso_date = suffix[0] == b'-'
            && suffix[1..5].iter().all(|b| b.is_ascii_digit())
            && suffix[5] == b'-'
            && suffix[6..8].iter().all(|b| b.is_ascii_digit())
            && suffix[8] == b'-'
            && suffix[9..11].iter().all(|b| b.is_ascii_digit());
        if is_iso_date && model_id.is_char_boundary(start) {
            return Some(model_id[..start].to_string());
        }
    }

    let (base, suffix) = model_id.rsplit_once('-')?;
    if base.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if suffix.len() == 8 {
        return Some(base.to_string());
    }
    if suffix.len() == 6 {
        let month: u32 = suffix[2..4].parse().unwrap_or(0);
        let day: u32 = suffix[4..6].parse().unwrap_or(0);
        if (1..=12).contains(&month) && (1..=31).contains(&day) {
            return Some(base.to_string());
        }
    }
    None
}

fn strip_reasoning_effort_suffix(model_id: &str) -> Option<String> {
    for suffix in ["-minimal", "-low", "-medium", "-high", "-xhigh"] {
        if let Some(stripped) = model_id.strip_suffix(suffix) {
            if !stripped.is_empty() {
                return Some(stripped.to_string());
            }
        }
    }
    None
}

fn model_pricing_candidates(model_id: &str) -> Vec<String> {
    let cleaned = normalize_model_id_for_pricing(model_id);
    if cleaned.is_empty() || matches!(cleaned.as_str(), "unknown" | "null" | "none") {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let mut queue = vec![cleaned];
    while let Some(candidate) = queue.pop() {
        if !push_unique_model_candidate(&mut candidates, candidate.clone()) {
            continue;
        }
        if let Some(stripped) = strip_known_model_namespace(&candidate) {
            queue.push(stripped);
        }
        if let Some(stripped) = strip_claude_desktop_non_anthropic_prefix(&candidate) {
            queue.push(stripped);
        }
        if let Some(stripped) = strip_bedrock_model_version_suffix(&candidate) {
            queue.push(stripped);
        }
        if let Some(stripped) = strip_model_date_suffix(&candidate) {
            queue.push(stripped);
        }
        if let Some(stripped) = strip_reasoning_effort_suffix(&candidate) {
            queue.push(stripped);
        }
        if candidate.starts_with("claude-") && candidate.contains('.') {
            queue.push(candidate.replace('.', "-"));
        }
    }
    candidates
}

fn should_try_pricing_prefix_match(model_id: &str) -> bool {
    let dash_count = model_id.matches('-').count();
    if model_id.starts_with("claude-") {
        return dash_count >= 3;
    }
    if ["o1", "o3", "o4", "o5"]
        .iter()
        .any(|prefix| model_id.starts_with(prefix))
    {
        return dash_count >= 1;
    }
    [
        "gpt-",
        "gemini-",
        "deepseek-",
        "qwen-",
        "glm-",
        "kimi-",
        "minimax-",
    ]
    .iter()
    .any(|prefix| model_id.starts_with(prefix))
        && dash_count >= 2
}

fn pricing_from_entry(entry: &ManagedModelPricingEntry) -> ManagedUsagePricing {
    ManagedUsagePricing {
        input_cost_per_million: entry.input_cost_per_million,
        output_cost_per_million: entry.output_cost_per_million,
        cache_read_cost_per_million: entry.cache_read_cost_per_million,
        cache_creation_cost_per_million: entry.cache_creation_cost_per_million,
    }
}

fn find_local_model_pricing(model: &str) -> Option<ManagedUsagePricing> {
    let candidates = model_pricing_candidates(model);
    if candidates.is_empty() {
        return None;
    }
    let overrides = MODEL_PRICING_OVERRIDES.lock().ok();

    for candidate in &candidates {
        if let Some(entry) = overrides.as_ref().and_then(|entries| {
            entries
                .values()
                .find(|entry| normalize_model_id_for_pricing(&entry.model_id) == *candidate)
        }) {
            return Some(pricing_from_entry(entry));
        }
        if let Some(entry) = LOCAL_MODEL_PRICING
            .iter()
            .find(|entry| normalize_model_id_for_pricing(entry.model_id) == *candidate)
        {
            return Some(ManagedUsagePricing {
                input_cost_per_million: entry.input_cost_per_million,
                output_cost_per_million: entry.output_cost_per_million,
                cache_read_cost_per_million: entry.cache_read_cost_per_million,
                cache_creation_cost_per_million: entry.cache_creation_cost_per_million,
            });
        }
    }

    for candidate in &candidates {
        if !should_try_pricing_prefix_match(candidate) {
            continue;
        }
        let prefix = format!("{candidate}-");
        let mut matches = Vec::new();
        if let Some(entries) = overrides.as_ref() {
            matches.extend(entries.values().filter_map(|entry| {
                let id = normalize_model_id_for_pricing(&entry.model_id);
                id.starts_with(&prefix)
                    .then(|| (id.len(), true, pricing_from_entry(entry)))
            }));
        }
        matches.extend(LOCAL_MODEL_PRICING.iter().filter_map(|entry| {
            let id = normalize_model_id_for_pricing(entry.model_id);
            id.starts_with(&prefix).then(|| {
                (
                    id.len(),
                    false,
                    ManagedUsagePricing {
                        input_cost_per_million: entry.input_cost_per_million,
                        output_cost_per_million: entry.output_cost_per_million,
                        cache_read_cost_per_million: entry.cache_read_cost_per_million,
                        cache_creation_cost_per_million: entry.cache_creation_cost_per_million,
                    },
                )
            })
        }));
        matches.sort_by_key(|(length, is_user, _)| (*length, !*is_user));
        if let Some((_, _, pricing)) = matches.into_iter().next() {
            return Some(pricing);
        }
    }
    None
}

fn compute_local_usage_cost(
    app_type: &str,
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
) -> f64 {
    let Some(pricing) = find_local_model_pricing(model) else {
        return 0.0;
    };

    let fresh_input = if app_type == "codex" {
        input_tokens.saturating_sub(cache_read_tokens + cache_creation_tokens)
    } else {
        input_tokens
    };
    let million = 1_000_000.0_f64;
    (fresh_input as f64 * pricing.input_cost_per_million / million)
        + (output_tokens as f64 * pricing.output_cost_per_million / million)
        + (cache_read_tokens as f64 * pricing.cache_read_cost_per_million / million)
        + (cache_creation_tokens as f64 * pricing.cache_creation_cost_per_million / million)
}

fn find_local_pricing_info(model: &str) -> Option<ManagedUsagePricing> {
    find_local_model_pricing(model)
}

fn model_pricing_vendor(model_id: &str) -> String {
    let normalized = normalize_model_id_for_pricing(model_id);
    for (prefix, vendor) in [
        ("claude", "claude"),
        ("gpt", "openai"),
        ("o1", "openai"),
        ("o3", "openai"),
        ("gemini", "gemini"),
        ("grok", "grok"),
        ("deepseek", "deepseek"),
        ("glm", "glm"),
        ("minimax", "minimax"),
        ("kimi", "kimi"),
        ("qwen", "qwen"),
    ] {
        if normalized.starts_with(prefix) {
            return vendor.to_string();
        }
    }
    String::from("other")
}

fn seed_model_pricing_entries() -> Vec<ManagedModelPricingEntry> {
    LOCAL_MODEL_PRICING
        .iter()
        .map(|entry| ManagedModelPricingEntry {
            model_id: entry.model_id.to_string(),
            display_name: entry.model_id.to_string(),
            vendor: model_pricing_vendor(entry.model_id),
            input_cost_per_million: entry.input_cost_per_million,
            output_cost_per_million: entry.output_cost_per_million,
            cache_read_cost_per_million: entry.cache_read_cost_per_million,
            cache_creation_cost_per_million: entry.cache_creation_cost_per_million,
            completion_cost_per_million: 0.0,
            image_cost_per_million: 0.0,
            video_cost_per_million: 0.0,
            audio_cost_per_million: 0.0,
            source: String::from("seed"),
            updated_at: String::new(),
        })
        .collect()
}

fn model_pricing_database_path(home: &Path) -> PathBuf {
    home.join(".chuchen-terminal").join("usage.db")
}

const MODEL_PRICING_DB_SCRIPT: &str = r#"
import json
import sqlite3
import sys
from pathlib import Path

db_path, operation, payload_text = sys.argv[1:4]
Path(db_path).parent.mkdir(parents=True, exist_ok=True)
conn = sqlite3.connect(db_path, timeout=3)
conn.execute("""CREATE TABLE IF NOT EXISTS model_pricing (
    model_id TEXT PRIMARY KEY, display_name TEXT NOT NULL, vendor TEXT NOT NULL DEFAULT 'other',
    input_cost_per_million REAL NOT NULL, output_cost_per_million REAL NOT NULL,
    cache_read_cost_per_million REAL NOT NULL DEFAULT 0,
    cache_creation_cost_per_million REAL NOT NULL DEFAULT 0,
    completion_cost_per_million REAL NOT NULL DEFAULT 0,
    image_cost_per_million REAL NOT NULL DEFAULT 0,
    video_cost_per_million REAL NOT NULL DEFAULT 0,
    audio_cost_per_million REAL NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'user', updated_at TEXT NOT NULL
)""")
payload = json.loads(payload_text or "null")
if operation == "upsert":
    conn.execute("""INSERT OR REPLACE INTO model_pricing VALUES (
        :modelId, :displayName, :vendor, :inputCostPerMillion, :outputCostPerMillion,
        :cacheReadCostPerMillion, :cacheCreationCostPerMillion,
        :completionCostPerMillion, :imageCostPerMillion, :videoCostPerMillion,
        :audioCostPerMillion, 'user', :updatedAt
    )""", payload)
elif operation == "delete":
    conn.execute("DELETE FROM model_pricing WHERE model_id=?", (str(payload),))
conn.commit()
rows = []
for row in conn.execute("""SELECT model_id, display_name, vendor,
    input_cost_per_million, output_cost_per_million, cache_read_cost_per_million,
    cache_creation_cost_per_million, completion_cost_per_million, image_cost_per_million,
    video_cost_per_million, audio_cost_per_million, source, updated_at
    FROM model_pricing ORDER BY display_name"""):
    rows.append({
        "modelId": row[0], "displayName": row[1], "vendor": row[2],
        "inputCostPerMillion": row[3], "outputCostPerMillion": row[4],
        "cacheReadCostPerMillion": row[5], "cacheCreationCostPerMillion": row[6],
        "completionCostPerMillion": row[7], "imageCostPerMillion": row[8],
        "videoCostPerMillion": row[9], "audioCostPerMillion": row[10],
        "source": row[11], "updatedAt": row[12],
    })
print(json.dumps(rows, ensure_ascii=False))
"#;

fn execute_model_pricing_db(
    home: &Path,
    operation: &str,
    payload: &Value,
) -> Result<Vec<ManagedModelPricingEntry>, String> {
    let db_path = model_pricing_database_path(home);
    let arguments = [
        db_path.to_string_lossy().to_string(),
        operation.to_string(),
        serde_json::to_string(payload).map_err(|error| error.to_string())?,
    ];
    let attempts: [(&str, &[&str]); 3] = [
        ("python", &["-c"]),
        ("py", &["-3", "-c"]),
        ("python3", &["-c"]),
    ];
    for (program, prefix) in attempts {
        let output = Command::new(program)
            .args(prefix)
            .arg(MODEL_PRICING_DB_SCRIPT)
            .args(&arguments)
            .env("PYTHONIOENCODING", "utf-8")
            .output();
        let Ok(output) = output else {
            continue;
        };
        if output.status.success() {
            if let Ok(entries) =
                serde_json::from_slice::<Vec<ManagedModelPricingEntry>>(&output.stdout)
            {
                return Ok(entries);
            }
        }
    }
    Err(String::from("无法访问本地模型价格数据库。"))
}

fn refresh_model_pricing_overrides(home: &Path) -> Result<Vec<ManagedModelPricingEntry>, String> {
    let entries = execute_model_pricing_db(home, "list", &Value::Null)?;
    if let Ok(mut overrides) = MODEL_PRICING_OVERRIDES.lock() {
        *overrides = entries
            .iter()
            .cloned()
            .map(|entry| (normalize_model_id_for_pricing(&entry.model_id), entry))
            .collect();
    }
    MODEL_PRICING_LOADED.store(true, Ordering::Release);
    Ok(entries)
}

fn ensure_model_pricing_loaded(home: &Path) {
    if !MODEL_PRICING_LOADED.load(Ordering::Acquire) {
        if refresh_model_pricing_overrides(home).is_err() {
            MODEL_PRICING_LOADED.store(true, Ordering::Release);
        }
    }
}

fn all_model_pricing_entries(home: &Path) -> Result<Vec<ManagedModelPricingEntry>, String> {
    let overrides = refresh_model_pricing_overrides(home)?;
    let mut entries = seed_model_pricing_entries()
        .into_iter()
        .map(|entry| (normalize_model_id_for_pricing(&entry.model_id), entry))
        .collect::<BTreeMap<_, _>>();
    for entry in overrides {
        entries.insert(normalize_model_id_for_pricing(&entry.model_id), entry);
    }
    Ok(entries.into_values().collect())
}

fn validate_model_pricing(entry: &ManagedModelPricingEntry) -> Result<(), String> {
    if entry.model_id.trim().is_empty() || entry.display_name.trim().is_empty() {
        return Err(String::from("模型 ID 和显示名称不能为空。"));
    }
    for value in [
        entry.input_cost_per_million,
        entry.output_cost_per_million,
        entry.cache_read_cost_per_million,
        entry.cache_creation_cost_per_million,
        entry.completion_cost_per_million,
        entry.image_cost_per_million,
        entry.video_cost_per_million,
        entry.audio_cost_per_million,
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(String::from("模型价格必须是非负有限数值。"));
        }
    }
    Ok(())
}

fn bucket_usage_timestamp(timestamp: &str, bucket: &str) -> String {
    let trimmed = timestamp.trim();
    if trimmed.len() < 10 {
        return String::from("unknown");
    }

    match bucket {
        "minute" => trimmed.chars().take(16).collect::<String>() + ":00Z",
        "hour" => trimmed.chars().take(13).collect::<String>() + ":00:00Z",
        "week" => chrono::NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d")
            .ok()
            .map(|date| {
                let monday =
                    date - chrono::Duration::days(i64::from(date.weekday().num_days_from_monday()));
                format!("{}T00:00:00Z", monday.format("%Y-%m-%d"))
            })
            .unwrap_or_else(|| String::from("unknown")),
        "month" => {
            if trimmed.len() >= 7 {
                format!("{}-01T00:00:00Z", &trimmed[..7])
            } else {
                String::from("unknown")
            }
        }
        "all" => String::from("all"),
        _ => format!("{}T00:00:00Z", &trimmed[..10]),
    }
}

fn timestamp_in_range(timestamp: &str, start_at: Option<&str>, end_at: Option<&str>) -> bool {
    let trimmed = timestamp.trim();
    if let Some(start_at) = start_at {
        let start_at = start_at.trim();
        if !start_at.is_empty()
            && match (
                chrono::DateTime::parse_from_rfc3339(trimmed),
                chrono::DateTime::parse_from_rfc3339(start_at),
            ) {
                (Ok(timestamp), Ok(start_at)) => timestamp < start_at,
                _ => trimmed < start_at,
            }
        {
            return false;
        }
    }
    if let Some(end_at) = end_at {
        let end_at = end_at.trim();
        if !end_at.is_empty()
            && match (
                chrono::DateTime::parse_from_rfc3339(trimmed),
                chrono::DateTime::parse_from_rfc3339(end_at),
            ) {
                (Ok(timestamp), Ok(end_at)) => timestamp > end_at,
                _ => trimmed > end_at,
            }
        {
            return false;
        }
    }
    true
}

fn parse_iso_timestamp_seconds(timestamp: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(timestamp.trim())
        .ok()
        .map(|dt| dt.timestamp())
}

fn duration_between_timestamps_ms(start: &str, end: &str) -> Option<u64> {
    let start = chrono::DateTime::parse_from_rfc3339(start.trim()).ok()?;
    let end = chrono::DateTime::parse_from_rfc3339(end.trim()).ok()?;
    let millis = end
        .timestamp_millis()
        .saturating_sub(start.timestamp_millis());
    Some(millis.max(0) as u64)
}

fn usage_log_model_matches(left: &str, right: &str) -> bool {
    let left = left.trim().to_lowercase();
    let right = right.trim().to_lowercase();
    left == right || left == "unknown" || right == "unknown"
}

fn should_keep_usage_log(
    log: &ManagedUsageRequestLog,
    all_logs: &[ManagedUsageRequestLog],
) -> bool {
    let is_session_source = matches!(
        log.data_source.as_str(),
        "session_log" | "codex_session" | "gemini_session" | "opencode_session" | "hermes_session"
    );
    if !is_session_source {
        return true;
    }

    let Some(log_ts) = parse_iso_timestamp_seconds(&log.created_at) else {
        return true;
    };
    let allow_missing_cache_creation = matches!(
        log.app_type.as_str(),
        "codex" | "gemini" | "opencode" | "hermes"
    ) && log.cache_creation_tokens == 0;

    !all_logs.iter().any(|candidate| {
        if candidate.data_source != "proxy" {
            return false;
        }
        if candidate.app_type != log.app_type {
            return false;
        }
        if candidate.status_code < 200 || candidate.status_code >= 300 {
            return false;
        }
        if candidate.input_tokens != log.input_tokens
            || candidate.output_tokens != log.output_tokens
            || candidate.cache_read_tokens != log.cache_read_tokens
        {
            return false;
        }
        if !allow_missing_cache_creation
            && candidate.cache_creation_tokens != log.cache_creation_tokens
        {
            return false;
        }
        if !usage_log_model_matches(&candidate.model, &log.model) {
            return false;
        }
        let Some(candidate_ts) = parse_iso_timestamp_seconds(&candidate.created_at) else {
            return false;
        };
        (candidate_ts - log_ts).abs() <= MANAGED_USAGE_DEDUP_WINDOW_SECONDS
    })
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

fn run_command_capture_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(bool, String, String), String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动命令：{error}"))?;
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| format!("读取命令输出失败：{error}"))?;
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Ok((output.status.success(), stdout, stderr));
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(String::from("命令执行超时。"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(error) => return Err(format!("等待命令结束失败：{error}")),
        }
    }
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

fn origin_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let after_scheme = trimmed.find("://")?;
    let scheme_and_host = &trimmed[..after_scheme + 3];
    let rest = &trimmed[after_scheme + 3..];
    let host = rest.split('/').next().unwrap_or_default().trim();
    if host.is_empty() {
        return None;
    }
    Some(format!("{scheme_and_host}{host}"))
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

fn json_path_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn json_first_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    paths.iter().find_map(|path| json_path_string(value, path))
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

fn extract_toml_section(raw: &str, section_name: &str) -> String {
    let header = format!("[{section_name}]");
    let mut in_section = false;
    let mut section = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section {
                break;
            }
            in_section = trimmed == header;
            continue;
        }
        if in_section {
            section.push_str(line);
            section.push('\n');
        }
    }

    section
}

fn parse_env_text(raw: &str) -> HashMap<String, String> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            Some((key.trim().to_string(), value.to_string()))
        })
        .collect()
}

fn hermes_dir(home: &Path) -> PathBuf {
    if let Some(raw) = std::env::var_os("HERMES_HOME") {
        let value = raw.to_string_lossy();
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    #[cfg(windows)]
    {
        let local_app_data = std::env::var_os("LOCALAPPDATA")
            .map(|value| value.to_string_lossy().trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData").join("Local"));
        return local_app_data.join("hermes");
    }

    #[cfg(not(windows))]
    {
        home.join(".hermes")
    }
}

fn hermes_config_path(home: &Path) -> PathBuf {
    hermes_dir(home).join("config.yaml")
}

fn claude_prompt_path(home: &Path) -> PathBuf {
    home.join(".claude").join("CLAUDE.md")
}

fn codex_prompt_path(home: &Path) -> PathBuf {
    home.join(".codex").join("AGENTS.md")
}

fn gemini_prompt_path(home: &Path) -> PathBuf {
    home.join(".gemini").join("GEMINI.md")
}

fn hermes_prompt_path(home: &Path) -> PathBuf {
    hermes_dir(home).join("AGENTS.md")
}

fn prompt_path_for_tool(home: &Path, tool_kind: &str) -> Result<PathBuf, String> {
    match tool_kind {
        "claude" | "claude-code" => Ok(claude_prompt_path(home)),
        "codex" => Ok(codex_prompt_path(home)),
        "gemini" | "gemini-cli" => Ok(gemini_prompt_path(home)),
        "hermes" => Ok(hermes_prompt_path(home)),
        other => Err(format!("暂不支持 {other} 的 Prompt 文件管理。")),
    }
}

fn is_yaml_top_level_key_line(line: &str) -> bool {
    let trimmed_end = line.trim_end_matches('\r');
    if trimmed_end.is_empty() {
        return false;
    }

    let first = trimmed_end.as_bytes()[0];
    if first == b' ' || first == b'\t' || first == b'#' || first == b'-' {
        return false;
    }

    let Some((key, rest)) = trimmed_end.split_once(':') else {
        return false;
    };
    !key.trim().is_empty() && (rest.is_empty() || rest.starts_with([' ', '\t']))
}

fn extract_yaml_section(raw: &str, section_name: &str) -> String {
    let mut in_section = false;
    let mut section = String::new();

    for line in raw.lines() {
        let normalized = line.trim_end_matches('\r');
        if is_yaml_top_level_key_line(normalized) {
            let key = normalized
                .split_once(':')
                .map(|(key, _)| key.trim())
                .unwrap_or_default();
            if in_section && key != section_name {
                break;
            }
            if !in_section {
                in_section = key == section_name;
            }
        }

        if in_section {
            section.push_str(normalized);
            section.push('\n');
        }
    }

    section
}

fn extract_yaml_string_field(raw: &str, field: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() != field {
            continue;
        }

        let value = value
            .split('#')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn json_value_from_toml(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(v) => Value::String(v.clone()),
        toml::Value::Integer(v) => Value::Number((*v).into()),
        toml::Value::Float(v) => serde_json::Number::from_f64(*v)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Datetime(v) => Value::String(v.to_string()),
        toml::Value::Array(values) => {
            Value::Array(values.iter().map(json_value_from_toml).collect())
        }
        toml::Value::Table(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), json_value_from_toml(value)))
                .collect(),
        ),
    }
}

fn toml_value_from_json(value: &Value) -> Option<toml::Value> {
    match value {
        Value::Null => None,
        Value::Bool(v) => Some(toml::Value::Boolean(*v)),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Some(toml::Value::Integer(i))
            } else {
                v.as_f64().map(toml::Value::Float)
            }
        }
        Value::String(v) => Some(toml::Value::String(v.clone())),
        Value::Array(values) => Some(toml::Value::Array(
            values.iter().filter_map(toml_value_from_json).collect(),
        )),
        Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (key, value) in map {
                if let Some(value) = toml_value_from_json(value) {
                    table.insert(key.clone(), value);
                }
            }
            Some(toml::Value::Table(table))
        }
    }
}

fn normalize_mcp_transport_type(config: &Value) -> String {
    config
        .get("type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            if config.get("command").is_some() {
                Some(String::from("stdio"))
            } else if config.get("httpUrl").is_some() {
                Some(String::from("http"))
            } else if config.get("url").is_some() {
                Some(String::from("sse"))
            } else {
                None
            }
        })
        .unwrap_or_else(|| String::from("stdio"))
}

fn normalize_mcp_server(
    id: &str,
    config: Value,
    source_kind: &str,
    app_targets: &[&str],
) -> ManagedMcpServer {
    ManagedMcpServer {
        id: id.to_string(),
        name: id.to_string(),
        app_targets: app_targets
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        transport_type: normalize_mcp_transport_type(&config),
        config,
        source_kind: source_kind.to_string(),
    }
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

fn provider_color(provider_kind: &str) -> &'static str {
    match provider_kind {
        "claude-code" => "#d97706",
        "gemini-cli" => "#0f9f6e",
        "hermes" => "#0891b2",
        "opencode" => "#2563eb",
        "custom-cli" => "#475569",
        _ => "#4b83ff",
    }
}

fn provider_tool_target(provider_kind: &str) -> &'static str {
    match provider_kind {
        "claude-code" => "claude",
        "gemini-cli" => "gemini",
        "hermes" => "generic",
        "opencode" => "opencode",
        "custom-cli" => "generic",
        _ => "codex",
    }
}

fn provider_switch_command(provider_kind: &str, profile_name: &str) -> String {
    let _ = (provider_kind, profile_name);
    String::new()
}

fn apply_provider_command_text(
    provider_kind: &str,
    profile_name: &str,
    switch_command: Option<String>,
) -> String {
    let command = switch_command.unwrap_or_default().trim().to_string();
    if !command.is_empty() {
        return command;
    }
    provider_switch_command(provider_kind, profile_name)
}

fn run_provider_switch_command(command_text: &str) -> Result<(String, String), String> {
    let timeout = Duration::from_millis(15_000);
    let parts = parse_simple_command(command_text)?;
    let program = parts
        .first()
        .ok_or_else(|| String::from("没有可执行的切换命令。"))?;
    if !command_exists(program) {
        return Err(format!(
            "未找到切换命令「{program}」。cc-switch 桌面版没有稳定 CLI；请先安装可执行命令，或改用只读导入。"
        ));
    }
    let args = parts.iter().skip(1).map(String::as_str).collect::<Vec<_>>();

    let result = run_command_capture_with_timeout(program, &args, timeout)?;

    let (success, stdout, stderr) = result;
    if success {
        Ok((stdout, stderr))
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(String::from("切换命令执行失败。"))
    }
}

fn parse_simple_command(command_text: &str) -> Result<Vec<String>, String> {
    let trimmed = command_text.trim();
    if trimmed.is_empty() {
        return Err(String::from("没有可执行的切换命令。"));
    }
    if trimmed.contains(['|', '&', ';', '<', '>']) {
        return Err(String::from(
            "切换命令只支持单个可执行文件和参数，不支持 shell 管道或重定向。",
        ));
    }
    Ok(trimmed
        .split_whitespace()
        .map(ToString::to_string)
        .collect())
}

fn command_exists(program: &str) -> bool {
    if Path::new(program).exists() {
        return true;
    }

    #[cfg(target_os = "windows")]
    let output = Command::new("cmd").args(["/C", "where", program]).output();

    #[cfg(not(target_os = "windows"))]
    {
        return std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|path| path.join(program).exists()))
            .unwrap_or(false);
    }

    #[cfg(target_os = "windows")]
    output
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn count_active_terminal_sessions() -> usize {
    SESSION_STORE
        .lock()
        .map(|sessions| sessions.len())
        .unwrap_or(0)
}

fn write_codex_profile_to_live_config(home: &Path, profile_name: &str) -> Result<String, String> {
    let config_path = home.join(".codex").join("config.toml");
    let raw = read_small_text_file(&config_path)
        .ok_or_else(|| format!("未找到 Codex 配置文件：{}", display_home_path(&config_path)))?;
    if extract_toml_section(&raw, &format!("model_providers.{profile_name}"))
        .trim()
        .is_empty()
    {
        return Err(format!(
            "Codex 配置中不存在 model_provider「{profile_name}」，当前只支持切换已存在的 provider。"
        ));
    }

    let mut replaced = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            if key.trim() == "model_provider" {
                lines.push(format!("model_provider = \"{profile_name}\""));
                replaced = true;
                continue;
            }
        }
        lines.push(line.to_string());
    }
    if !replaced {
        lines.insert(0, format!("model_provider = \"{profile_name}\""));
    }

    std::fs::write(&config_path, lines.join("\n"))
        .map_err(|error| format!("写入 Codex config.toml 失败：{error}"))?;
    Ok(display_home_path(&config_path))
}

fn write_codex_payload_to_live_config(
    home: &Path,
    config_payload: &str,
    auth_payload: Option<&str>,
) -> Result<String, String> {
    if config_payload.trim().is_empty() {
        return Err(String::from("Codex 配置快照为空，无法写入。"));
    }

    let auth_payload = auth_payload.filter(|payload| !payload.trim().is_empty());
    if let Some(payload) = auth_payload {
        serde_json::from_str::<Value>(payload)
            .map_err(|error| format!("Codex auth.json 快照不是合法 JSON，已停止写入：{error}"))?;
    }

    let config_path = home.join(".codex").join("config.toml");
    let parent = config_path
        .parent()
        .ok_or_else(|| String::from("无法定位 Codex 配置目录。"))?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建 Codex 配置目录失败：{error}"))?;
    std::fs::write(&config_path, config_payload)
        .map_err(|error| format!("写入 Codex config.toml 失败：{error}"))?;

    if let Some(payload) = auth_payload {
        let auth_path = home.join(".codex").join("auth.json");
        std::fs::write(&auth_path, payload)
            .map_err(|error| format!("写入 Codex auth.json 失败：{error}"))?;
    }

    Ok(display_home_path(&config_path))
}

fn write_gemini_payload_to_live_config(
    home: &Path,
    config_payload: &str,
    auth_payload: Option<&str>,
) -> Result<String, String> {
    let gemini_dir = home.join(".gemini");
    std::fs::create_dir_all(&gemini_dir)
        .map_err(|error| format!("创建 Gemini 配置目录失败：{error}"))?;

    let settings_path = gemini_dir.join("settings.json");
    let config_text = if config_payload.trim().is_empty() {
        String::from("{\n  \"security\": {\n    \"auth\": {\n      \"selectedType\": \"oauth-personal\"\n    }\n  }\n}")
    } else {
        config_payload.trim().to_string()
    };
    serde_json::from_str::<Value>(&config_text)
        .map_err(|error| format!("Gemini settings.json 快照不是合法 JSON：{error}"))?;
    std::fs::write(&settings_path, config_text)
        .map_err(|error| format!("写入 Gemini settings.json 失败：{error}"))?;

    let env_path = gemini_dir.join(".env");
    if let Some(payload) = auth_payload
        .map(str::trim)
        .filter(|payload| !payload.is_empty())
    {
        std::fs::write(&env_path, payload)
            .map_err(|error| format!("写入 Gemini .env 失败：{error}"))?;
    }

    Ok(display_home_path(&settings_path))
}

fn write_claude_payload_to_live_config(
    home: &Path,
    config_payload: &str,
    auth_payload: Option<&str>,
) -> Result<String, String> {
    let claude_dir = home.join(".claude");
    std::fs::create_dir_all(&claude_dir)
        .map_err(|error| format!("创建 Claude 配置目录失败：{error}"))?;

    let settings_path = claude_dir.join("settings.json");
    let config_text = if config_payload.trim().is_empty() {
        String::from("{}")
    } else {
        config_payload.trim().to_string()
    };
    serde_json::from_str::<Value>(&config_text)
        .map_err(|error| format!("Claude settings.json 快照不是合法 JSON：{error}"))?;
    std::fs::write(&settings_path, config_text)
        .map_err(|error| format!("写入 Claude settings.json 失败：{error}"))?;

    if let Some(payload) = auth_payload
        .map(str::trim)
        .filter(|payload| !payload.is_empty())
    {
        serde_json::from_str::<Value>(payload)
            .map_err(|error| format!("Claude ~/.claude.json 快照不是合法 JSON：{error}"))?;
        let global_path = home.join(".claude.json");
        std::fs::write(&global_path, payload)
            .map_err(|error| format!("写入 Claude ~/.claude.json 失败：{error}"))?;
    }

    Ok(display_home_path(&settings_path))
}

fn write_hermes_payload_to_live_config(
    home: &Path,
    config_payload: &str,
) -> Result<String, String> {
    if config_payload.trim().is_empty() {
        return Err(String::from("Hermes config.yaml 快照为空，无法写回。"));
    }

    let config_path = hermes_config_path(home);
    let parent = config_path
        .parent()
        .ok_or_else(|| String::from("无法定位 Hermes 配置目录。"))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建 Hermes 配置目录失败：{error}"))?;
    std::fs::write(&config_path, config_payload)
        .map_err(|error| format!("写入 Hermes config.yaml 失败：{error}"))?;

    Ok(display_home_path(&config_path))
}

fn read_claude_mcp_servers(home: &Path) -> Result<Vec<ManagedMcpServer>, String> {
    let path = home.join(".claude.json");
    let Some(raw) = read_small_text_file(&path) else {
        return Ok(Vec::new());
    };
    let root = serde_json::from_str::<Value>(&raw)
        .map_err(|error| format!("Claude MCP 配置不是合法 JSON：{error}"))?;
    let servers = root
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Ok(servers
        .into_iter()
        .map(|(id, config)| normalize_mcp_server(&id, config, "native-live", &["claude"]))
        .collect())
}

fn write_claude_mcp_servers(home: &Path, servers: &[ManagedMcpServer]) -> Result<String, String> {
    let path = home.join(".claude.json");
    let mut root = read_small_text_file(&path)
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let object = root
        .as_object_mut()
        .ok_or_else(|| String::from("Claude ~/.claude.json 根必须是对象。"))?;
    let mut out = serde_json::Map::new();
    for server in servers {
        if !server.app_targets.iter().any(|target| target == "claude") {
            continue;
        }
        out.insert(server.id.clone(), server.config.clone());
    }
    object.insert(String::from("mcpServers"), Value::Object(out));
    let text = serde_json::to_string_pretty(&root)
        .map_err(|error| format!("序列化 Claude MCP 配置失败：{error}"))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("无法定位 Claude MCP 配置目录。"))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建 Claude MCP 配置目录失败：{error}"))?;
    std::fs::write(&path, text).map_err(|error| format!("写入 Claude MCP 配置失败：{error}"))?;
    Ok(display_home_path(&path))
}

fn read_gemini_mcp_servers(home: &Path) -> Result<Vec<ManagedMcpServer>, String> {
    let path = home.join(".gemini").join("settings.json");
    let Some(raw) = read_small_text_file(&path) else {
        return Ok(Vec::new());
    };
    let root = serde_json::from_str::<Value>(&raw)
        .map_err(|error| format!("Gemini settings.json 不是合法 JSON：{error}"))?;
    let mut servers = root
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for config in servers.values_mut() {
        if let Some(object) = config.as_object_mut() {
            if let Some(http_url) = object.remove("httpUrl") {
                object.insert(String::from("url"), http_url);
                object.insert(String::from("type"), Value::String(String::from("http")));
            } else if object.get("type").is_none() {
                if object.get("command").is_some() {
                    object.insert(String::from("type"), Value::String(String::from("stdio")));
                } else if object.get("url").is_some() {
                    object.insert(String::from("type"), Value::String(String::from("sse")));
                }
            }
        }
    }

    Ok(servers
        .into_iter()
        .map(|(id, config)| normalize_mcp_server(&id, config, "native-live", &["gemini"]))
        .collect())
}

fn write_gemini_mcp_servers(home: &Path, servers: &[ManagedMcpServer]) -> Result<String, String> {
    let path = home.join(".gemini").join("settings.json");
    let mut root = read_small_text_file(&path)
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let object = root
        .as_object_mut()
        .ok_or_else(|| String::from("Gemini settings.json 根必须是对象。"))?;
    let mut out = serde_json::Map::new();
    for server in servers {
        if !server.app_targets.iter().any(|target| target == "gemini") {
            continue;
        }
        let mut config = server.config.clone();
        if let Some(map) = config.as_object_mut() {
            let transport = map
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("stdio")
                .to_string();
            if transport == "http" {
                if let Some(url_value) = map.remove("url") {
                    map.insert(String::from("httpUrl"), url_value);
                }
            }
            map.remove("type");
        }
        out.insert(server.id.clone(), config);
    }
    object.insert(String::from("mcpServers"), Value::Object(out));
    let text = serde_json::to_string_pretty(&root)
        .map_err(|error| format!("序列化 Gemini MCP 配置失败：{error}"))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("无法定位 Gemini MCP 配置目录。"))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建 Gemini MCP 配置目录失败：{error}"))?;
    std::fs::write(&path, text).map_err(|error| format!("写入 Gemini MCP 配置失败：{error}"))?;
    Ok(display_home_path(&path))
}

fn read_codex_mcp_servers(home: &Path) -> Result<Vec<ManagedMcpServer>, String> {
    let path = home.join(".codex").join("config.toml");
    let Some(raw) = read_small_text_file(&path) else {
        return Ok(Vec::new());
    };
    let root = raw
        .parse::<toml::Value>()
        .map_err(|error| format!("Codex config.toml 不是合法 TOML：{error}"))?;
    let servers = root
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    Ok(servers
        .into_iter()
        .map(|(id, config)| {
            normalize_mcp_server(
                &id,
                json_value_from_toml(&config),
                "native-live",
                &["codex"],
            )
        })
        .collect())
}

fn write_codex_mcp_servers(home: &Path, servers: &[ManagedMcpServer]) -> Result<String, String> {
    let path = home.join(".codex").join("config.toml");
    let raw = read_small_text_file(&path).unwrap_or_default();
    let mut root = if raw.trim().is_empty() {
        toml::Value::Table(toml::map::Map::new())
    } else {
        raw.parse::<toml::Value>()
            .map_err(|error| format!("Codex config.toml 不是合法 TOML：{error}"))?
    };
    let table = root
        .as_table_mut()
        .ok_or_else(|| String::from("Codex config.toml 根必须是对象。"))?;
    let mut mcp_table = toml::map::Map::new();
    for server in servers {
        if !server.app_targets.iter().any(|target| target == "codex") {
            continue;
        }
        let Some(config) = toml_value_from_json(&server.config) else {
            continue;
        };
        mcp_table.insert(server.id.clone(), config);
    }
    table.insert(String::from("mcp_servers"), toml::Value::Table(mcp_table));
    let text = toml::to_string_pretty(&root)
        .map_err(|error| format!("序列化 Codex MCP 配置失败：{error}"))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("无法定位 Codex MCP 配置目录。"))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建 Codex MCP 配置目录失败：{error}"))?;
    std::fs::write(&path, text).map_err(|error| format!("写入 Codex MCP 配置失败：{error}"))?;
    Ok(display_home_path(&path))
}

fn read_hermes_mcp_servers(home: &Path) -> Result<Vec<ManagedMcpServer>, String> {
    let path = hermes_config_path(home);
    let Some(raw) = read_small_text_file(&path) else {
        return Ok(Vec::new());
    };
    let root = serde_yaml::from_str::<serde_yaml::Value>(&raw)
        .map_err(|error| format!("Hermes config.yaml 不是合法 YAML：{error}"))?;
    let servers = root
        .get("mcp_servers")
        .and_then(serde_yaml::Value::as_mapping)
        .cloned()
        .unwrap_or_default();
    let mut result = Vec::new();
    for (key, value) in servers {
        let Some(id) = key.as_str().map(str::to_string) else {
            continue;
        };
        let config = serde_yaml::from_value::<Value>(value)
            .map_err(|error| format!("Hermes MCP 项 {id} 无法转成 JSON：{error}"))?;
        let mut normalized = config;
        if let Some(map) = normalized.as_object_mut() {
            if map.get("type").is_none() {
                if map.get("command").is_some() {
                    map.insert(String::from("type"), Value::String(String::from("stdio")));
                } else if map.get("url").is_some() {
                    map.insert(String::from("type"), Value::String(String::from("sse")));
                }
            }
        }
        result.push(normalize_mcp_server(
            &id,
            normalized,
            "native-live",
            &["hermes"],
        ));
    }
    Ok(result)
}

fn write_hermes_mcp_servers(home: &Path, servers: &[ManagedMcpServer]) -> Result<String, String> {
    let path = hermes_config_path(home);
    let raw = read_small_text_file(&path).unwrap_or_default();
    let mut root = if raw.trim().is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str::<serde_yaml::Value>(&raw)
            .map_err(|error| format!("Hermes config.yaml 不是合法 YAML：{error}"))?
    };
    let mapping = root
        .as_mapping_mut()
        .ok_or_else(|| String::from("Hermes config.yaml 根必须是对象。"))?;
    let mut out = serde_yaml::Mapping::new();
    for server in servers {
        if !server.app_targets.iter().any(|target| target == "hermes") {
            continue;
        }
        let mut config = server.config.clone();
        if let Some(map) = config.as_object_mut() {
            map.remove("type");
        }
        let yaml = serde_yaml::to_value(config)
            .map_err(|error| format!("序列化 Hermes MCP 项失败：{error}"))?;
        out.insert(serde_yaml::Value::String(server.id.clone()), yaml);
    }
    mapping.insert(
        serde_yaml::Value::String(String::from("mcp_servers")),
        serde_yaml::Value::Mapping(out),
    );
    let text = serde_yaml::to_string(&root)
        .map_err(|error| format!("序列化 Hermes MCP 配置失败：{error}"))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("无法定位 Hermes MCP 配置目录。"))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建 Hermes MCP 配置目录失败：{error}"))?;
    std::fs::write(&path, text).map_err(|error| format!("写入 Hermes MCP 配置失败：{error}"))?;
    Ok(display_home_path(&path))
}

fn read_managed_prompt_profiles_for_tools(
    home: &Path,
    tool_kinds: &[String],
) -> Result<Vec<ManagedPromptProfile>, String> {
    let mut result = Vec::new();
    for tool_kind in tool_kinds {
        let path = prompt_path_for_tool(home, tool_kind)?;
        let content = read_small_text_file(&path).unwrap_or_default();
        result.push(ManagedPromptProfile {
            tool_kind: tool_kind.clone(),
            name: format!("{} Prompt", tool_kind),
            prompt_file_path: display_home_path(&path),
            content,
            source_kind: String::from("native-live"),
            is_active: path.exists(),
        });
    }
    Ok(result)
}

fn write_managed_prompt_profile(
    home: &Path,
    tool_kind: &str,
    content: &str,
) -> Result<String, String> {
    let path = prompt_path_for_tool(home, tool_kind)?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("无法定位 Prompt 文件目录。"))?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建 Prompt 目录失败：{error}"))?;
    std::fs::write(&path, content).map_err(|error| format!("写入 Prompt 文件失败：{error}"))?;
    Ok(display_home_path(&path))
}

fn managed_usage_log(
    id: String,
    provider_id: String,
    provider_name: String,
    app_type: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    first_token_ms: Option<u64>,
    duration_ms: Option<u64>,
    status_code: u16,
    data_source: String,
    created_at: String,
) -> ManagedUsageRequestLog {
    let pricing_model = normalize_model_id_for_pricing(&model);
    let pricing = find_local_pricing_info(&model);
    let input_cost_usd = pricing
        .as_ref()
        .map(|p| input_tokens as f64 * p.input_cost_per_million / 1_000_000.0)
        .unwrap_or(0.0);
    let output_cost_usd = pricing
        .as_ref()
        .map(|p| output_tokens as f64 * p.output_cost_per_million / 1_000_000.0)
        .unwrap_or(0.0);
    let cache_read_cost_usd = pricing
        .as_ref()
        .map(|p| cache_read_tokens as f64 * p.cache_read_cost_per_million / 1_000_000.0)
        .unwrap_or(0.0);
    let cache_creation_cost_usd = pricing
        .as_ref()
        .map(|p| cache_creation_tokens as f64 * p.cache_creation_cost_per_million / 1_000_000.0)
        .unwrap_or(0.0);
    let total_cost_usd =
        input_cost_usd + output_cost_usd + cache_read_cost_usd + cache_creation_cost_usd;

    ManagedUsageRequestLog {
        id,
        provider_id,
        provider_name,
        app_type,
        model,
        pricing_model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        input_cost_usd,
        output_cost_usd,
        cache_read_cost_usd,
        cache_creation_cost_usd,
        total_cost_usd,
        first_token_ms,
        duration_ms,
        status_code,
        data_source,
        created_at,
        pricing,
    }
}

fn managed_logs_from_detected_stats(
    stats: Option<DetectedProviderUsageStats>,
) -> Vec<ManagedUsageRequestLog> {
    stats
        .map(|stats| {
            stats
                .request_logs
                .into_iter()
                .map(|log| {
                    managed_usage_log(
                        log.id,
                        log.provider_id,
                        log.provider_name,
                        log.app_type,
                        log.model,
                        log.input_tokens,
                        log.output_tokens,
                        log.cache_read_tokens,
                        log.cache_creation_tokens,
                        log.first_token_ms,
                        Some(log.duration_ms),
                        log.status_code,
                        log.data_source,
                        log.created_at,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_native_usage_file(home: &Path, path: &Path) -> Vec<ManagedUsageRequestLog> {
    let file = vec![path.to_path_buf()];
    if path.starts_with(home.join(".codex")) {
        return managed_logs_from_detected_stats(read_codex_native_usage_stats_from_files(
            file, None,
        ));
    }
    if path.starts_with(home.join(".claude")) {
        return managed_logs_from_detected_stats(read_claude_native_usage_stats_from_files(
            file, None,
        ));
    }
    if path.starts_with(home.join(".gemini")) {
        return managed_logs_from_detected_stats(read_gemini_native_usage_stats_from_files(
            file, None,
        ));
    }
    Vec::new()
}

fn native_usage_snapshot_path(home: &Path) -> PathBuf {
    home.join(".chuchen-terminal")
        .join("native-usage-snapshot.json")
}

fn native_usage_inventory(home: &Path) -> Vec<PathBuf> {
    let mut files = collect_codex_usage_files(home);
    files.extend(collect_claude_usage_files(home));
    files.extend(collect_gemini_usage_files(home));
    files.sort();
    files.dedup();
    files
}

fn usage_monitor_inventory(home: &Path) -> Vec<PathBuf> {
    let mut files = native_usage_inventory(home);
    let hermes_db = hermes_dir(home).join("state.db");
    if hermes_db.exists() {
        files.push(hermes_db);
    }
    files.sort();
    files.dedup();
    files
}

fn native_usage_inventory_fingerprint(files: &[PathBuf]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for path in files {
        path.hash(&mut hasher);
        if let Ok(metadata) = std::fs::metadata(path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.duration_since(std::time::UNIX_EPOCH) {
                    elapsed.as_secs().hash(&mut hasher);
                    elapsed.subsec_nanos().hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

fn write_native_usage_snapshot(path: &Path, snapshot: &NativeUsageSnapshot) {
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(payload) = serde_json::to_vec(snapshot) else {
        return;
    };
    let _ = std::fs::write(path, payload);
}

fn native_usage_file_marker(path: &Path) -> Option<(u64, u128)> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified_nanos = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((metadata.len(), modified_nanos))
}

fn native_usage_offset_probe(path: &Path, offset: u64) -> Vec<u8> {
    let probe_len = offset.min(64) as usize;
    if probe_len == 0 {
        return Vec::new();
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    if file
        .seek(SeekFrom::Start(offset.saturating_sub(probe_len as u64)))
        .is_err()
    {
        return Vec::new();
    }
    let mut probe = vec![0; probe_len];
    if file.read_exact(&mut probe).is_err() {
        return Vec::new();
    }
    probe
}

fn native_usage_probe_matches(path: &Path, offset: u64, expected: &[u8]) -> bool {
    !expected.is_empty() && native_usage_offset_probe(path, offset) == expected
}

fn read_jsonl_values_from_offset(
    path: &Path,
    start_offset: u64,
    mut consume: impl FnMut(Value),
) -> Option<u64> {
    let mut file = std::fs::File::open(path).ok()?;
    file.seek(SeekFrom::Start(start_offset)).ok()?;
    let mut reader = BufReader::new(file);
    let mut offset = start_offset;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).ok()?;
        if bytes_read == 0 {
            break;
        }
        if line.trim().is_empty() {
            offset += bytes_read as u64;
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(value) => {
                consume(value);
                offset += bytes_read as u64;
            }
            Err(_) if !line.ends_with('\n') => break,
            Err(_) => offset += bytes_read as u64,
        }
    }

    Some(offset)
}

fn parse_claude_native_value(state: &mut ClaudeNativeParseState, value: &Value) {
    if value.get("type").and_then(|item| item.as_str()) == Some("system")
        && value.get("subtype").and_then(|item| item.as_str()) == Some("turn_duration")
    {
        if let Some(session_id) = value.get("sessionId").and_then(|item| item.as_str()) {
            let duration_ms = value
                .get("durationMs")
                .and_then(|item| item.as_u64())
                .unwrap_or(0);
            if duration_ms > 0 {
                state
                    .session_durations
                    .entry(session_id.to_string())
                    .and_modify(|existing| *existing = (*existing).max(duration_ms))
                    .or_insert(duration_ms);
            }
        }
        return;
    }
    if value.get("type").and_then(|item| item.as_str()) != Some("assistant") {
        return;
    }
    let Some(message) = value.get("message") else {
        return;
    };
    let Some(message_id) = message.get("id").and_then(|item| item.as_str()) else {
        return;
    };
    let Some(usage) = message.get("usage") else {
        return;
    };
    let parsed = ClaudeNativeAssistantUsage {
        message_id: message_id.to_string(),
        session_id: value
            .get("sessionId")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
        model: message
            .get("model")
            .and_then(|item| item.as_str())
            .unwrap_or("unknown")
            .to_string(),
        input_tokens: usage
            .get("input_tokens")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        output_tokens: usage
            .get("output_tokens")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        cache_read_tokens: usage
            .get("cache_read_input_tokens")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        cache_creation_tokens: usage
            .get("cache_creation_input_tokens")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        stop_reason: message
            .get("stop_reason")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
        timestamp: value
            .get("timestamp")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
    };
    let should_replace = match state.messages.get(message_id) {
        None => true,
        Some(existing) => {
            (parsed.stop_reason.is_some() && existing.stop_reason.is_none())
                || (parsed.stop_reason.is_some() == existing.stop_reason.is_some()
                    && parsed.output_tokens > existing.output_tokens)
        }
    };
    if should_replace {
        state.messages.insert(message_id.to_string(), parsed);
    }
}

fn claude_native_logs_from_state(state: &ClaudeNativeParseState) -> Vec<ManagedUsageRequestLog> {
    state
        .messages
        .values()
        .filter(|msg| {
            msg.input_tokens > 0
                || msg.output_tokens > 0
                || msg.cache_read_tokens > 0
                || msg.cache_creation_tokens > 0
        })
        .map(|msg| {
            managed_usage_log(
                format!("claude-native:{}", msg.message_id),
                String::from("_session"),
                String::from("Claude (Session)"),
                String::from("claude"),
                msg.model.clone(),
                msg.input_tokens,
                msg.output_tokens,
                msg.cache_read_tokens,
                msg.cache_creation_tokens,
                None,
                msg.session_id
                    .as_deref()
                    .and_then(|session_id| state.session_durations.get(session_id).copied()),
                200,
                String::from("session_log"),
                msg.timestamp.clone().unwrap_or_default(),
            )
        })
        .collect()
}

fn parse_codex_native_value(
    state: &mut CodexNativeParseState,
    logs: &mut Vec<ManagedUsageRequestLog>,
    value: &Value,
) {
    let Some(event_type) = value.get("type").and_then(|item| item.as_str()) else {
        return;
    };
    match event_type {
        "session_meta" if state.session_id.is_none() => {
            state.session_id = value
                .get("payload")
                .and_then(|payload| {
                    payload
                        .get("session_id")
                        .or_else(|| payload.get("sessionId"))
                        .or_else(|| payload.get("id"))
                })
                .and_then(|item| item.as_str())
                .map(ToString::to_string);
            state.current_provider_id = value
                .get("payload")
                .and_then(|payload| payload.get("model_provider"))
                .and_then(|item| item.as_str())
                .map(ToString::to_string);
        }
        "turn_context" => {
            state.current_turn_id = value
                .get("payload")
                .and_then(|payload| payload.get("turn_id"))
                .and_then(|item| item.as_str())
                .map(ToString::to_string);
            if let Some(model) = value
                .get("payload")
                .and_then(|payload| {
                    payload
                        .get("model")
                        .or_else(|| payload.get("info").and_then(|info| info.get("model")))
                })
                .and_then(|item| item.as_str())
            {
                state.current_model = normalize_codex_usage_model(model);
            }
        }
        "event_msg" => {
            let Some(payload) = value.get("payload") else {
                return;
            };
            let event_kind = payload
                .get("type")
                .and_then(|item| item.as_str())
                .unwrap_or("");
            if event_kind == "task_started" {
                state.current_turn_started_at =
                    payload.get("started_at").and_then(|item| item.as_i64());
                state.current_turn_first_token_ms = None;
                state.current_turn_id = payload
                    .get("turn_id")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string);
                return;
            }
            if event_kind == "task_complete" {
                let completed_turn_id = payload
                    .get("turn_id")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string);
                if completed_turn_id == state.current_turn_id {
                    if let Some(log) = state
                        .last_request_log_index
                        .and_then(|index| logs.get_mut(index))
                    {
                        log.duration_ms = payload
                            .get("duration_ms")
                            .and_then(|item| item.as_u64())
                            .or(log.duration_ms);
                        log.first_token_ms = payload
                            .get("time_to_first_token_ms")
                            .and_then(|item| item.as_u64())
                            .or(log.first_token_ms);
                    }
                }
                return;
            }
            if event_kind != "token_count" {
                return;
            }
            let Some(info) = payload.get("info").filter(|item| !item.is_null()) else {
                return;
            };
            if let Some(model) = info
                .get("model")
                .or_else(|| info.get("model_name"))
                .or_else(|| payload.get("model"))
                .and_then(|item| item.as_str())
            {
                state.current_model = normalize_codex_usage_model(model);
            }
            let (current, is_total) = if let Some(total) = info.get("total_token_usage") {
                (parse_codex_native_cumulative_tokens(total), true)
            } else if let Some(last) = info.get("last_token_usage") {
                (parse_codex_native_cumulative_tokens(last), false)
            } else {
                return;
            };
            let Some(current) = current else {
                return;
            };
            let delta = if is_total {
                let delta = compute_codex_native_delta(&state.prev_total, &current);
                state.prev_total = Some(current);
                delta
            } else {
                current
            };
            let cached_input = delta.cached_input.min(delta.input);
            if delta.input == 0 && cached_input == 0 && delta.output == 0 {
                return;
            }
            state.event_index += 1;
            let created_at = value
                .get("timestamp")
                .and_then(|item| item.as_str())
                .unwrap_or("")
                .to_string();
            let event_ts = parse_iso_timestamp_seconds(&created_at);
            let first_token_ms = match (state.current_turn_started_at, event_ts) {
                (Some(started_at), Some(event_ts))
                    if state.current_turn_first_token_ms.is_none() =>
                {
                    let delta = (event_ts.saturating_sub(started_at) * 1000).max(0) as u64;
                    state.current_turn_first_token_ms = Some(delta);
                    Some(delta)
                }
                _ => state.current_turn_first_token_ms,
            };
            logs.push(managed_usage_log(
                format!(
                    "codex-native:{}:{}",
                    state.session_id.as_deref().unwrap_or("unknown"),
                    state.event_index
                ),
                state
                    .current_provider_id
                    .clone()
                    .unwrap_or_else(|| String::from("unknown")),
                state
                    .current_provider_id
                    .clone()
                    .unwrap_or_else(|| String::from("Codex Native")),
                String::from("codex"),
                state.current_model.clone(),
                delta.input,
                delta.output,
                cached_input,
                0,
                first_token_ms,
                None,
                200,
                String::from("codex_session"),
                created_at,
            ));
            state.last_request_log_index = logs.len().checked_sub(1);
        }
        _ => {}
    }
}

fn gemini_native_message_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .and_then(|value| {
            value
                .get("messages")
                .and_then(|messages| messages.as_array())
                .map(Vec::len)
        })
        .unwrap_or(0)
}

fn parse_native_usage_file_snapshot(
    home: &Path,
    path: &Path,
    size: u64,
    modified_nanos: u128,
    cached: Option<NativeUsageFileSnapshot>,
) -> (NativeUsageFileSnapshot, bool) {
    let can_tail_read = cached.as_ref().is_some_and(|cached| {
        size > cached.size
            && cached.offset > 0
            && size >= cached.offset
            && native_usage_probe_matches(path, cached.offset, &cached.offset_probe)
    });

    if path.starts_with(home.join(".codex")) {
        let (mut logs, mut state, start_offset, tail_read) = match cached {
            Some(cached) if can_tail_read => match cached.parser_state {
                Some(NativeUsageParserState::Codex(state)) => {
                    (cached.logs, state, cached.offset, true)
                }
                _ => (
                    Vec::new(),
                    CodexNativeParseState {
                        current_model: String::from("unknown"),
                        ..Default::default()
                    },
                    0,
                    false,
                ),
            },
            _ => (
                Vec::new(),
                CodexNativeParseState {
                    current_model: String::from("unknown"),
                    ..Default::default()
                },
                0,
                false,
            ),
        };
        let offset = read_jsonl_values_from_offset(path, start_offset, |value| {
            parse_codex_native_value(&mut state, &mut logs, &value)
        })
        .unwrap_or(start_offset);
        return (
            NativeUsageFileSnapshot {
                size,
                modified_nanos,
                offset,
                offset_probe: native_usage_offset_probe(path, offset),
                parser_state: Some(NativeUsageParserState::Codex(state)),
                logs,
            },
            tail_read,
        );
    }

    if path.starts_with(home.join(".claude")) {
        let (mut state, start_offset, tail_read) = match cached {
            Some(cached) if can_tail_read => match cached.parser_state {
                Some(NativeUsageParserState::Claude(state)) => (state, cached.offset, true),
                _ => (ClaudeNativeParseState::default(), 0, false),
            },
            _ => (ClaudeNativeParseState::default(), 0, false),
        };
        let offset = read_jsonl_values_from_offset(path, start_offset, |value| {
            parse_claude_native_value(&mut state, &value)
        })
        .unwrap_or(start_offset);
        let logs = claude_native_logs_from_state(&state);
        return (
            NativeUsageFileSnapshot {
                size,
                modified_nanos,
                offset,
                offset_probe: native_usage_offset_probe(path, offset),
                parser_state: Some(NativeUsageParserState::Claude(state)),
                logs,
            },
            tail_read,
        );
    }

    let logs = parse_native_usage_file(home, path);
    (
        NativeUsageFileSnapshot {
            size,
            modified_nanos,
            offset: size,
            offset_probe: native_usage_offset_probe(path, size),
            parser_state: Some(NativeUsageParserState::Gemini {
                message_count: gemini_native_message_count(path),
            }),
            logs,
        },
        false,
    )
}

fn collect_native_usage_logs(home: &Path) -> Vec<ManagedUsageRequestLog> {
    collect_native_usage_logs_with_stats(home).0
}

fn collect_native_usage_logs_with_stats(
    home: &Path,
) -> (Vec<ManagedUsageRequestLog>, NativeUsageScanStats) {
    let _guard = NATIVE_USAGE_SNAPSHOT_LOCK.lock().ok();
    let files = native_usage_inventory(home);
    let snapshot_path = native_usage_snapshot_path(home);
    let mut previous = std::fs::read(&snapshot_path)
        .ok()
        .and_then(|payload| serde_json::from_slice::<NativeUsageSnapshot>(&payload).ok())
        .map(|snapshot| snapshot.files)
        .unwrap_or_default();
    let mut current = BTreeMap::new();
    let mut stats = NativeUsageScanStats::default();

    for path in files {
        let Some((size, modified_nanos)) = native_usage_file_marker(&path) else {
            continue;
        };
        let key = path.to_string_lossy().to_string();
        let cached = previous.remove(&key);
        let cached = match cached {
            Some(cached) if cached.size == size && cached.modified_nanos == modified_nanos => {
                stats.reused_files += 1;
                current.insert(key, cached);
                continue;
            }
            cached => cached,
        };
        let (snapshot, tail_read) =
            parse_native_usage_file_snapshot(home, &path, size, modified_nanos, cached);
        if tail_read {
            stats.tail_parsed_files += 1;
        } else {
            stats.reset_files += 1;
        }
        current.insert(key, snapshot);
    }

    let snapshot_changed =
        stats.tail_parsed_files > 0 || stats.reset_files > 0 || !previous.is_empty();
    let mut deduplicated = BTreeMap::new();
    for log in current.values().flat_map(|file| file.logs.iter()) {
        deduplicated.insert(format!("{}:{}", log.app_type, log.id), log.clone());
    }
    if snapshot_changed {
        write_native_usage_snapshot(&snapshot_path, &NativeUsageSnapshot { files: current });
    }
    let mut logs = deduplicated.into_values().collect::<Vec<_>>();
    logs.extend(read_hermes_native_usage_logs(home));
    (logs, stats)
}

fn invalidate_managed_usage_cache() {
    if let Ok(mut cache) = MANAGED_USAGE_LOG_CACHE.lock() {
        *cache = None;
    }
}

fn start_usage_file_monitor(app: AppHandle, home: PathBuf) {
    thread::spawn(move || {
        let mut previous = native_usage_inventory_fingerprint(&usage_monitor_inventory(&home));
        loop {
            thread::sleep(Duration::from_secs(10));
            let current = native_usage_inventory_fingerprint(&usage_monitor_inventory(&home));
            if current == previous {
                continue;
            }
            previous = current;
            invalidate_managed_usage_cache();
            let _ = app.emit(
                "usage-log-recorded",
                serde_json::json!({ "updatedAt": chrono::Utc::now().to_rfc3339() }),
            );
        }
    });
}

#[derive(Clone)]
struct UsageProviderCatalogEntry {
    app_type: String,
    identity_key: String,
    profile_name: String,
    name: String,
    request_host: String,
    is_active: bool,
}

fn usage_app_type_for_provider_kind(provider_kind: &str) -> &str {
    match provider_kind {
        "claude-code" => "claude",
        "gemini-cli" => "gemini",
        "hermes" => "hermes",
        "opencode" => "opencode",
        _ => "codex",
    }
}

fn provider_url_host(value: Option<&str>) -> String {
    let normalized = normalized_provider_url(value);
    normalized
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(&normalized)
        .split('/')
        .next()
        .unwrap_or_default()
        .to_string()
}

fn current_provider_catalog(home: &Path) -> Vec<UsageProviderCatalogEntry> {
    collect_detected_provider_profiles(home)
        .into_iter()
        .map(|profile| UsageProviderCatalogEntry {
            app_type: usage_app_type_for_provider_kind(&profile.provider_kind).to_string(),
            identity_key: profile.identity_key,
            profile_name: profile.profile_name,
            name: profile.name,
            request_host: provider_url_host(profile.request_base_url.as_deref()),
            is_active: profile.is_active,
        })
        .collect()
}

fn match_usage_provider_identity<'a>(
    catalog: &'a [UsageProviderCatalogEntry],
    app_type: &str,
    raw_id: &str,
    raw_name: &str,
) -> Option<&'a UsageProviderCatalogEntry> {
    let app_profiles = catalog
        .iter()
        .filter(|profile| profile.app_type == app_type)
        .collect::<Vec<_>>();
    if app_profiles.is_empty() {
        return None;
    }

    let direct_matches = app_profiles
        .iter()
        .filter(|profile| {
            profile.identity_key == raw_id
                || profile.profile_name.eq_ignore_ascii_case(raw_id)
                || profile.profile_name.eq_ignore_ascii_case(raw_name)
                || profile.name.eq_ignore_ascii_case(raw_id)
                || profile.name.eq_ignore_ascii_case(raw_name)
                || (!profile.request_host.is_empty()
                    && (raw_id.to_ascii_lowercase().contains(&profile.request_host)
                        || raw_name
                            .to_ascii_lowercase()
                            .contains(&profile.request_host)))
        })
        .copied()
        .collect::<Vec<_>>();

    let matched = if direct_matches.len() == 1 {
        direct_matches.first().copied()
    } else if direct_matches.len() > 1 {
        direct_matches.into_iter().find(|profile| profile.is_active)
    } else if raw_id.starts_with('_') || raw_id.starts_with("native:") || raw_id == "unknown" {
        app_profiles.into_iter().find(|profile| profile.is_active)
    } else {
        None
    };

    matched
}

fn enrich_usage_provider_identity_from_catalog(
    catalog: &[UsageProviderCatalogEntry],
    log: &mut ManagedUsageRequestLog,
) {
    if let Some(profile) = match_usage_provider_identity(
        catalog,
        &log.app_type,
        log.provider_id.trim(),
        log.provider_name.trim(),
    ) {
        log.provider_id = profile.identity_key.clone();
        log.provider_name = profile.name.clone();
    }
}

fn enrich_usage_aggregate_identity_from_catalog(
    catalog: &[UsageProviderCatalogEntry],
    row: &mut ManagedUsageAggregateRow,
) {
    if let Some(profile) = match_usage_provider_identity(
        catalog,
        &row.app_type,
        row.provider_id.trim(),
        row.provider_name.trim(),
    ) {
        row.provider_id = profile.identity_key.clone();
        row.provider_name = profile.name.clone();
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
        identity_key: format!("{provider_kind}:native:{profile_name}"),
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
        homepage_url: None,
        request_base_url: None,
        config_payload: None,
        auth_payload: None,
        detected_source: detected_source.to_string(),
        config_exists,
        usage_stats: None,
    }
}

fn detect_codex_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let config_path = home.join(".codex").join("config.toml");
    let auth_path = home.join(".codex").join("auth.json");
    let auth_payload = read_small_text_file(&auth_path);
    let Some(raw) = read_small_text_file(&config_path) else {
        let mut profile = build_detected_provider(
            "codex",
            String::from("Codex CLI Default"),
            String::from("default"),
            config_path,
            "cli-config",
            String::from("未检测到配置文件"),
            String::from("gpt-5"),
            false,
            String::from("未发现 ~/.codex/config.toml；保留路径方便后续创建或手动登记。"),
            "codex-config",
        );
        profile.auth_payload = auth_payload;
        return vec![profile];
    };

    let top_model =
        extract_toml_string_field(&raw, "model").unwrap_or_else(|| String::from("gpt-5"));
    let active_provider = extract_toml_string_field(&raw, "model_provider")
        .unwrap_or_else(|| String::from("default"));
    let active_section = extract_toml_section(&raw, &format!("model_providers.{active_provider}"));
    let provider_name = extract_toml_string_field(&active_section, "name")
        .unwrap_or_else(|| active_provider.clone());
    let base_url = extract_toml_string_field(&active_section, "base_url").unwrap_or_default();

    let mut profile = build_detected_provider(
        "codex",
        format!("Codex Live · {provider_name}"),
        active_provider.clone(),
        config_path,
        "cli-config",
        if active_section.contains("experimental_bearer_token")
            || active_section.contains("env_key")
            || auth_payload.is_some()
        {
            String::from("Codex 当前生效配置（检测到认证引用，未读取明文）")
        } else {
            String::from("Codex 当前生效配置")
        },
        top_model,
        true,
        if base_url.is_empty() {
            String::from("读取 ~/.codex/config.toml 与 auth.json 当前 live 配置，可作为一条完整 profile 导入。")
        } else {
            format!("读取当前 live 配置；请求地址为 {base_url}，可作为一条完整 profile 导入。")
        },
        "codex-config",
    );
    profile.homepage_url = origin_from_url(&base_url);
    profile.request_base_url = if base_url.trim().is_empty() {
        None
    } else {
        Some(base_url)
    };
    profile.config_payload = Some(raw);
    profile.auth_payload = auth_payload;
    vec![profile]
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct CodexNativeCumulativeTokens {
    input: u64,
    cached_input: u64,
    output: u64,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct CodexNativeParseState {
    session_id: Option<String>,
    current_provider_id: Option<String>,
    current_model: String,
    prev_total: Option<CodexNativeCumulativeTokens>,
    event_index: u32,
    current_turn_started_at: Option<i64>,
    current_turn_first_token_ms: Option<u64>,
    current_turn_id: Option<String>,
    last_request_log_index: Option<usize>,
}

fn normalize_codex_usage_model(raw: &str) -> String {
    let mut name = raw.to_lowercase();
    if let Some(pos) = name.rfind('/') {
        name = name[pos + 1..].to_string();
    }
    if name.len() > 11 && name.is_char_boundary(name.len() - 11) {
        let suffix = &name[name.len() - 11..];
        if suffix.is_ascii()
            && suffix.as_bytes()[0] == b'-'
            && suffix[1..5].chars().all(|c| c.is_ascii_digit())
            && suffix.as_bytes()[5] == b'-'
            && suffix[6..8].chars().all(|c| c.is_ascii_digit())
            && suffix.as_bytes()[8] == b'-'
            && suffix[9..11].chars().all(|c| c.is_ascii_digit())
        {
            name.truncate(name.len() - 11);
        }
    }
    name
}

fn parse_codex_native_cumulative_tokens(
    total_usage: &Value,
) -> Option<CodexNativeCumulativeTokens> {
    if total_usage.is_null() || !total_usage.is_object() {
        return None;
    }
    Some(CodexNativeCumulativeTokens {
        input: total_usage
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        cached_input: total_usage
            .get("cached_input_tokens")
            .or_else(|| total_usage.get("cache_read_input_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        output: total_usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    })
}

fn compute_codex_native_delta(
    prev: &Option<CodexNativeCumulativeTokens>,
    current: &CodexNativeCumulativeTokens,
) -> CodexNativeCumulativeTokens {
    match prev {
        None => current.clone(),
        Some(prev) => CodexNativeCumulativeTokens {
            input: current.input.saturating_sub(prev.input),
            cached_input: current.cached_input.saturating_sub(prev.cached_input),
            output: current.output.saturating_sub(prev.output),
        },
    }
}

fn collect_codex_usage_files(home: &Path) -> Vec<PathBuf> {
    fn collect_recursive(
        dir: &Path,
        files: &mut Vec<(std::time::SystemTime, PathBuf)>,
        depth: u32,
        max_depth: u32,
    ) {
        if depth > max_depth || !dir.is_dir() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files, depth + 1, max_depth);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                let modified = entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files.push((modified, path));
            }
        }
    }

    let mut files = Vec::new();
    collect_recursive(&home.join(".codex").join("sessions"), &mut files, 0, 4);
    collect_recursive(
        &home.join(".codex").join("archived_sessions"),
        &mut files,
        0,
        1,
    );
    files.sort_by(|left, right| right.0.cmp(&left.0));
    files.into_iter().map(|(_, path)| path).collect()
}

fn collect_claude_usage_files(home: &Path) -> Vec<PathBuf> {
    fn collect_recursive(
        dir: &Path,
        files: &mut Vec<(std::time::SystemTime, PathBuf)>,
        depth: u32,
        max_depth: u32,
    ) {
        if depth > max_depth || !dir.is_dir() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files, depth + 1, max_depth);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                let modified = entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files.push((modified, path));
            }
        }
    }

    let mut files = Vec::new();
    collect_recursive(&home.join(".claude").join("projects"), &mut files, 0, 5);
    files.sort_by(|left, right| right.0.cmp(&left.0));
    files.into_iter().map(|(_, path)| path).collect()
}

#[derive(Clone, Serialize, Deserialize)]
struct ClaudeNativeAssistantUsage {
    message_id: String,
    session_id: Option<String>,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    stop_reason: Option<String>,
    timestamp: Option<String>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct ClaudeNativeParseState {
    messages: HashMap<String, ClaudeNativeAssistantUsage>,
    session_durations: HashMap<String, u64>,
}

fn read_claude_native_usage_stats_from_files(
    files: Vec<PathBuf>,
    request_log_limit: Option<usize>,
) -> Option<DetectedProviderUsageStats> {
    if files.is_empty() {
        return None;
    }

    let mut messages: HashMap<String, ClaudeNativeAssistantUsage> = HashMap::new();
    let mut session_durations: HashMap<String, u64> = HashMap::new();

    for file_path in files {
        let Ok(file) = std::fs::File::open(&file_path) else {
            continue;
        };
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if value.get("type").and_then(|item| item.as_str()) == Some("system")
                && value.get("subtype").and_then(|item| item.as_str()) == Some("turn_duration")
            {
                if let Some(session_id) = value.get("sessionId").and_then(|item| item.as_str()) {
                    let duration_ms = value
                        .get("durationMs")
                        .and_then(|item| item.as_u64())
                        .unwrap_or(0);
                    if duration_ms > 0 {
                        session_durations
                            .entry(session_id.to_string())
                            .and_modify(|existing| *existing = (*existing).max(duration_ms))
                            .or_insert(duration_ms);
                    }
                }
                continue;
            }
            if value.get("type").and_then(|item| item.as_str()) != Some("assistant") {
                continue;
            }
            let Some(message) = value.get("message") else {
                continue;
            };
            let Some(message_id) = message.get("id").and_then(|item| item.as_str()) else {
                continue;
            };
            let Some(usage) = message.get("usage") else {
                continue;
            };

            let parsed = ClaudeNativeAssistantUsage {
                message_id: message_id.to_string(),
                session_id: value
                    .get("sessionId")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string),
                model: message
                    .get("model")
                    .and_then(|item| item.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                input_tokens: usage
                    .get("input_tokens")
                    .and_then(|item| item.as_u64())
                    .unwrap_or(0),
                output_tokens: usage
                    .get("output_tokens")
                    .and_then(|item| item.as_u64())
                    .unwrap_or(0),
                cache_read_tokens: usage
                    .get("cache_read_input_tokens")
                    .and_then(|item| item.as_u64())
                    .unwrap_or(0),
                cache_creation_tokens: usage
                    .get("cache_creation_input_tokens")
                    .and_then(|item| item.as_u64())
                    .unwrap_or(0),
                stop_reason: message
                    .get("stop_reason")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string),
                timestamp: value
                    .get("timestamp")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string),
            };

            let should_replace = match messages.get(message_id) {
                None => true,
                Some(existing) => {
                    if parsed.stop_reason.is_some() && existing.stop_reason.is_none() {
                        true
                    } else if parsed.stop_reason.is_some() == existing.stop_reason.is_some() {
                        parsed.output_tokens > existing.output_tokens
                    } else {
                        false
                    }
                }
            };
            if should_replace {
                messages.insert(message_id.to_string(), parsed);
            }
        }
    }

    if messages.is_empty() {
        return None;
    }

    let mut request_logs: Vec<DetectedProviderRequestLog> = Vec::new();
    let mut total_requests = 0_u64;
    let mut total_input_tokens = 0_u64;
    let mut total_output_tokens = 0_u64;
    let mut total_cache_read_tokens = 0_u64;
    let mut total_cache_creation_tokens = 0_u64;
    let mut total_cost_usd = 0.0_f64;
    let mut trends: BTreeMap<String, DetectedProviderUsageTrendPoint> = BTreeMap::new();

    for msg in messages.into_values() {
        let has_billable_tokens = msg.input_tokens > 0
            || msg.output_tokens > 0
            || msg.cache_read_tokens > 0
            || msg.cache_creation_tokens > 0;
        if !has_billable_tokens {
            continue;
        }

        total_requests += 1;
        total_input_tokens += msg.input_tokens;
        total_output_tokens += msg.output_tokens;
        total_cache_read_tokens += msg.cache_read_tokens;
        total_cache_creation_tokens += msg.cache_creation_tokens;

        let created_at = msg.timestamp.unwrap_or_default();
        let trend_key = if created_at.len() >= 10 {
            format!("{}T00:00:00Z", &created_at[..10])
        } else {
            String::from("unknown")
        };
        let cost_usd = compute_local_usage_cost(
            "claude",
            &msg.model,
            msg.input_tokens,
            msg.output_tokens,
            msg.cache_read_tokens,
            msg.cache_creation_tokens,
        );
        total_cost_usd += cost_usd;

        let trend = trends
            .entry(trend_key.clone())
            .or_insert(DetectedProviderUsageTrendPoint {
                timestamp: trend_key,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.0,
            });
        trend.input_tokens += msg.input_tokens;
        trend.output_tokens += msg.output_tokens;
        trend.cache_read_tokens += msg.cache_read_tokens;
        trend.cache_creation_tokens += msg.cache_creation_tokens;
        trend.cost_usd += cost_usd;

        request_logs.push(DetectedProviderRequestLog {
            id: format!("claude-native:{}", msg.message_id),
            provider_id: String::from("_session"),
            provider_name: String::from("Claude (Session)"),
            app_type: String::from("claude"),
            model: msg.model,
            input_tokens: msg.input_tokens,
            output_tokens: msg.output_tokens,
            cache_read_tokens: msg.cache_read_tokens,
            cache_creation_tokens: msg.cache_creation_tokens,
            cost_usd,
            status_code: 200,
            first_token_ms: None,
            duration_ms: msg
                .session_id
                .as_deref()
                .and_then(|session_id| session_durations.get(session_id).copied())
                .unwrap_or(0),
            data_source: String::from("session_log"),
            created_at,
        });
    }

    if total_requests == 0 {
        return None;
    }

    request_logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    if let Some(limit) = request_log_limit {
        request_logs.truncate(limit);
    }

    let cacheable = total_input_tokens + total_cache_read_tokens + total_cache_creation_tokens;
    Some(DetectedProviderUsageStats {
        summary: DetectedProviderUsageSummary {
            total_requests,
            total_cost_usd,
            total_input_tokens,
            total_output_tokens,
            total_cache_read_tokens,
            total_cache_creation_tokens,
            cache_hit_rate: if cacheable == 0 {
                0.0
            } else {
                total_cache_read_tokens as f64 / cacheable as f64
            },
        },
        trends: trends
            .into_values()
            .rev()
            .take(14)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        request_logs,
    })
}

#[derive(Clone, Copy)]
struct GeminiNativeTokens {
    input: u64,
    output: u64,
    cached: u64,
    thoughts: u64,
}

fn collect_gemini_usage_files(home: &Path) -> Vec<PathBuf> {
    let tmp_dir = home.join(".gemini").join("tmp");
    if !tmp_dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    let Ok(project_dirs) = std::fs::read_dir(&tmp_dir) else {
        return Vec::new();
    };
    for entry in project_dirs.flatten() {
        let chats_dir = entry.path().join("chats");
        if !chats_dir.is_dir() {
            continue;
        }
        let Ok(chat_files) = std::fs::read_dir(&chats_dir) else {
            continue;
        };
        for file_entry in chat_files.flatten() {
            let path = file_entry.path();
            let is_session = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("session-") && name.ends_with(".json"))
                .unwrap_or(false);
            if is_session {
                let modified = file_entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files.push((modified, path));
            }
        }
    }
    files.sort_by(|left, right| right.0.cmp(&left.0));
    files.into_iter().map(|(_, path)| path).collect()
}

fn parse_gemini_native_tokens(tokens: &Value) -> GeminiNativeTokens {
    GeminiNativeTokens {
        input: tokens
            .get("input")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        output: tokens
            .get("output")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        cached: tokens
            .get("cached")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
        thoughts: tokens
            .get("thoughts")
            .and_then(|item| item.as_u64())
            .unwrap_or(0),
    }
}

fn read_gemini_native_usage_stats_from_files(
    files: Vec<PathBuf>,
    request_log_limit: Option<usize>,
) -> Option<DetectedProviderUsageStats> {
    if files.is_empty() {
        return None;
    }

    let mut request_logs: Vec<DetectedProviderRequestLog> = Vec::new();
    let mut total_requests = 0_u64;
    let mut total_input_tokens = 0_u64;
    let mut total_output_tokens = 0_u64;
    let mut total_cache_read_tokens = 0_u64;
    let total_cache_creation_tokens = 0_u64;
    let mut total_cost_usd = 0.0_f64;
    let mut trends: BTreeMap<String, DetectedProviderUsageTrendPoint> = BTreeMap::new();

    for file_path in files {
        let Ok(content) = std::fs::read_to_string(&file_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&content) else {
            continue;
        };
        let session_id = value
            .get("sessionId")
            .and_then(|item| item.as_str())
            .unwrap_or("unknown")
            .to_string();
        let Some(messages) = value.get("messages").and_then(|item| item.as_array()) else {
            continue;
        };

        for msg in messages {
            if msg.get("type").and_then(|item| item.as_str()) != Some("gemini") {
                continue;
            }
            let Some(tokens_obj) = msg.get("tokens").filter(|item| item.is_object()) else {
                continue;
            };
            let tokens = parse_gemini_native_tokens(tokens_obj);
            if tokens.input == 0 && tokens.output == 0 && tokens.thoughts == 0 && tokens.cached == 0
            {
                continue;
            }

            let output_tokens = tokens.output + tokens.thoughts;
            let model = msg
                .get("model")
                .and_then(|item| item.as_str())
                .unwrap_or("unknown")
                .to_string();
            let created_at = msg
                .get("timestamp")
                .and_then(|item| item.as_str())
                .unwrap_or("")
                .to_string();
            let message_id = msg
                .get("id")
                .and_then(|item| item.as_str())
                .unwrap_or("unknown");
            let timing_stamps: Vec<String> = msg
                .get("thoughts")
                .and_then(|item| item.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.get("timestamp").and_then(|value| value.as_str()))
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let first_token_ms = timing_stamps
                .first()
                .and_then(|stamp| duration_between_timestamps_ms(&created_at, stamp));
            let duration_ms = timing_stamps
                .last()
                .and_then(|stamp| duration_between_timestamps_ms(&created_at, stamp))
                .or_else(|| {
                    value
                        .get("lastUpdated")
                        .and_then(|item| item.as_str())
                        .and_then(|stamp| duration_between_timestamps_ms(&created_at, stamp))
                })
                .unwrap_or(0);
            let cost_usd = compute_local_usage_cost(
                "gemini",
                &model,
                tokens.input,
                output_tokens,
                tokens.cached,
                0,
            );

            total_requests += 1;
            total_input_tokens += tokens.input;
            total_output_tokens += output_tokens;
            total_cache_read_tokens += tokens.cached;
            total_cost_usd += cost_usd;

            let trend_key = if created_at.len() >= 10 {
                format!("{}T00:00:00Z", &created_at[..10])
            } else {
                String::from("unknown")
            };
            let trend =
                trends
                    .entry(trend_key.clone())
                    .or_insert(DetectedProviderUsageTrendPoint {
                        timestamp: trend_key,
                        input_tokens: 0,
                        output_tokens: 0,
                        cache_read_tokens: 0,
                        cache_creation_tokens: 0,
                        cost_usd: 0.0,
                    });
            trend.input_tokens += tokens.input;
            trend.output_tokens += output_tokens;
            trend.cache_read_tokens += tokens.cached;
            trend.cost_usd += cost_usd;

            request_logs.push(DetectedProviderRequestLog {
                id: format!("gemini-native:{}:{}", session_id, message_id),
                provider_id: String::from("_gemini_session"),
                provider_name: String::from("Gemini (Session)"),
                app_type: String::from("gemini"),
                model,
                input_tokens: tokens.input,
                output_tokens,
                cache_read_tokens: tokens.cached,
                cache_creation_tokens: 0,
                cost_usd,
                status_code: 200,
                first_token_ms,
                duration_ms,
                data_source: String::from("gemini_session"),
                created_at,
            });
        }
    }

    if total_requests == 0 {
        return None;
    }

    request_logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    if let Some(limit) = request_log_limit {
        request_logs.truncate(limit);
    }

    let cacheable = total_input_tokens + total_cache_read_tokens + total_cache_creation_tokens;
    Some(DetectedProviderUsageStats {
        summary: DetectedProviderUsageSummary {
            total_requests,
            total_cost_usd,
            total_input_tokens,
            total_output_tokens,
            total_cache_read_tokens,
            total_cache_creation_tokens,
            cache_hit_rate: if cacheable == 0 {
                0.0
            } else {
                total_cache_read_tokens as f64 / cacheable as f64
            },
        },
        trends: trends
            .into_values()
            .rev()
            .take(14)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        request_logs,
    })
}

fn read_codex_native_usage_stats_from_files(
    files: Vec<PathBuf>,
    request_log_limit: Option<usize>,
) -> Option<DetectedProviderUsageStats> {
    if files.is_empty() {
        return None;
    }

    let mut request_logs: Vec<DetectedProviderRequestLog> = Vec::new();
    let mut total_requests = 0_u64;
    let mut total_input_tokens = 0_u64;
    let mut total_output_tokens = 0_u64;
    let mut total_cache_read_tokens = 0_u64;
    let total_cache_creation_tokens = 0_u64;
    let mut total_cost_usd = 0.0_f64;
    let mut trends: BTreeMap<String, DetectedProviderUsageTrendPoint> = BTreeMap::new();

    for file_path in files {
        let Ok(file) = std::fs::File::open(&file_path) else {
            continue;
        };
        let reader = BufReader::new(file);
        let mut state = CodexNativeParseState {
            session_id: None,
            current_provider_id: None,
            current_model: String::from("unknown"),
            prev_total: None,
            event_index: 0,
            current_turn_started_at: None,
            current_turn_first_token_ms: None,
            current_turn_id: None,
            last_request_log_index: None,
        };

        for line in reader.lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            let is_event_msg = line.contains("\"event_msg\"");
            let is_turn_context = line.contains("\"turn_context\"");
            let is_session_meta = line.contains("\"session_meta\"");
            if !is_event_msg && !is_turn_context && !is_session_meta {
                continue;
            }

            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            let Some(event_type) = value.get("type").and_then(|item| item.as_str()) else {
                continue;
            };

            match event_type {
                "session_meta" if state.session_id.is_none() => {
                    state.session_id = value
                        .get("payload")
                        .and_then(|payload| {
                            payload
                                .get("session_id")
                                .or_else(|| payload.get("sessionId"))
                                .or_else(|| payload.get("id"))
                        })
                        .and_then(|item| item.as_str())
                        .map(ToString::to_string);
                    state.current_provider_id = value
                        .get("payload")
                        .and_then(|payload| payload.get("model_provider"))
                        .and_then(|item| item.as_str())
                        .map(ToString::to_string);
                }
                "turn_context" => {
                    state.current_turn_id = value
                        .get("payload")
                        .and_then(|payload| payload.get("turn_id"))
                        .and_then(|item| item.as_str())
                        .map(ToString::to_string);
                    if let Some(model) = value
                        .get("payload")
                        .and_then(|payload| {
                            payload
                                .get("model")
                                .or_else(|| payload.get("info").and_then(|info| info.get("model")))
                        })
                        .and_then(|item| item.as_str())
                    {
                        state.current_model = normalize_codex_usage_model(model);
                    }
                }
                "event_msg" => {
                    let Some(payload) = value.get("payload") else {
                        continue;
                    };
                    let event_kind = payload
                        .get("type")
                        .and_then(|item| item.as_str())
                        .unwrap_or("");
                    if event_kind == "task_started" {
                        state.current_turn_started_at =
                            payload.get("started_at").and_then(|item| item.as_i64());
                        state.current_turn_first_token_ms = None;
                        state.current_turn_id = payload
                            .get("turn_id")
                            .and_then(|item| item.as_str())
                            .map(ToString::to_string);
                        continue;
                    }
                    if event_kind == "task_complete" {
                        let completed_turn_id = payload
                            .get("turn_id")
                            .and_then(|item| item.as_str())
                            .map(ToString::to_string);
                        if completed_turn_id == state.current_turn_id {
                            if let Some(index) = state.last_request_log_index {
                                if let Some(log) = request_logs.get_mut(index) {
                                    log.duration_ms = payload
                                        .get("duration_ms")
                                        .and_then(|item| item.as_u64())
                                        .unwrap_or(log.duration_ms);
                                    log.first_token_ms = payload
                                        .get("time_to_first_token_ms")
                                        .and_then(|item| item.as_u64())
                                        .or(log.first_token_ms);
                                }
                            }
                        }
                        continue;
                    }
                    if event_kind != "token_count" {
                        continue;
                    }
                    let Some(info) = payload.get("info").filter(|item| !item.is_null()) else {
                        continue;
                    };
                    if let Some(model) = info
                        .get("model")
                        .or_else(|| info.get("model_name"))
                        .or_else(|| payload.get("model"))
                        .and_then(|item| item.as_str())
                    {
                        state.current_model = normalize_codex_usage_model(model);
                    }

                    let (current, is_total) = if let Some(total) = info.get("total_token_usage") {
                        (parse_codex_native_cumulative_tokens(total), true)
                    } else if let Some(last) = info.get("last_token_usage") {
                        (parse_codex_native_cumulative_tokens(last), false)
                    } else {
                        continue;
                    };
                    let Some(current) = current else {
                        continue;
                    };
                    let delta = if is_total {
                        let delta = compute_codex_native_delta(&state.prev_total, &current);
                        state.prev_total = Some(current);
                        delta
                    } else {
                        current
                    };

                    let cached_input = delta.cached_input.min(delta.input);
                    if delta.input == 0 && cached_input == 0 && delta.output == 0 {
                        continue;
                    }

                    state.event_index += 1;
                    let created_at = value
                        .get("timestamp")
                        .and_then(|item| item.as_str())
                        .unwrap_or("")
                        .to_string();
                    let request_id = format!(
                        "codex-native:{}:{}",
                        state.session_id.as_deref().unwrap_or("unknown"),
                        state.event_index
                    );
                    let event_ts = parse_iso_timestamp_seconds(&created_at);
                    let first_token_ms = match (state.current_turn_started_at, event_ts) {
                        (Some(started_at), Some(event_ts))
                            if state.current_turn_first_token_ms.is_none() =>
                        {
                            let delta = (event_ts.saturating_sub(started_at) * 1000).max(0) as u64;
                            state.current_turn_first_token_ms = Some(delta);
                            Some(delta)
                        }
                        _ => state.current_turn_first_token_ms,
                    };

                    total_requests += 1;
                    total_input_tokens += delta.input;
                    total_output_tokens += delta.output;
                    total_cache_read_tokens += cached_input;

                    let trend_key = if created_at.len() >= 10 {
                        format!("{}T00:00:00Z", &created_at[..10])
                    } else {
                        String::from("unknown")
                    };
                    let cost_usd = compute_local_usage_cost(
                        "codex",
                        &state.current_model,
                        delta.input,
                        delta.output,
                        cached_input,
                        0,
                    );
                    total_cost_usd += cost_usd;

                    let trend = trends.entry(trend_key.clone()).or_insert(
                        DetectedProviderUsageTrendPoint {
                            timestamp: trend_key,
                            input_tokens: 0,
                            output_tokens: 0,
                            cache_read_tokens: 0,
                            cache_creation_tokens: 0,
                            cost_usd: 0.0,
                        },
                    );
                    trend.input_tokens += delta.input;
                    trend.output_tokens += delta.output;
                    trend.cache_read_tokens += cached_input;
                    trend.cost_usd += cost_usd;

                    request_logs.push(DetectedProviderRequestLog {
                        id: request_id,
                        provider_id: state
                            .current_provider_id
                            .clone()
                            .unwrap_or_else(|| String::from("unknown")),
                        provider_name: state
                            .current_provider_id
                            .clone()
                            .unwrap_or_else(|| String::from("Codex Native")),
                        app_type: String::from("codex"),
                        model: state.current_model.clone(),
                        input_tokens: delta.input,
                        output_tokens: delta.output,
                        cache_read_tokens: cached_input,
                        cache_creation_tokens: 0,
                        cost_usd,
                        status_code: 200,
                        first_token_ms,
                        duration_ms: 0,
                        data_source: String::from("codex_session"),
                        created_at,
                    });
                    state.last_request_log_index = request_logs.len().checked_sub(1);
                }
                _ => {}
            }
        }
    }

    if total_requests == 0 {
        return None;
    }

    request_logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    if let Some(limit) = request_log_limit {
        request_logs.truncate(limit);
    }

    let cacheable = total_input_tokens + total_cache_read_tokens + total_cache_creation_tokens;
    Some(DetectedProviderUsageStats {
        summary: DetectedProviderUsageSummary {
            total_requests,
            total_cost_usd,
            total_input_tokens,
            total_output_tokens,
            total_cache_read_tokens,
            total_cache_creation_tokens,
            cache_hit_rate: if cacheable == 0 {
                0.0
            } else {
                total_cache_read_tokens as f64 / cacheable as f64
            },
        },
        trends: trends
            .into_values()
            .rev()
            .take(14)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        request_logs,
    })
}

fn detect_claude_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let claude_dir = home.join(".claude");
    let settings_path = {
        let primary = claude_dir.join("settings.json");
        if primary.exists() {
            primary
        } else {
            let legacy = claude_dir.join("claude.json");
            if legacy.exists() {
                legacy
            } else {
                primary
            }
        }
    };
    let raw = read_small_text_file(&settings_path).unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(&raw).ok();
    let global_path = home.join(".claude.json");
    let global_raw = read_small_text_file(&global_path);
    let model = parsed
        .as_ref()
        .and_then(|value| json_first_string(value, &[&["model"], &["env", "ANTHROPIC_MODEL"]]))
        .or_else(|| extract_json_string_field(&raw, "model"))
        .unwrap_or_else(|| String::from("claude-sonnet-4"));
    let request_base_url = parsed
        .as_ref()
        .and_then(|value| {
            json_first_string(
                value,
                &[
                    &["env", "ANTHROPIC_BASE_URL"],
                    &["base_url"],
                    &["baseUrl"],
                    &["apiEndpoint"],
                ],
            )
        })
        .unwrap_or_default();
    let has_auth = raw.contains("ANTHROPIC_AUTH_TOKEN")
        || raw.contains("apiKey")
        || raw.contains("api_key")
        || raw.contains("auth_token");
    let mut profile = build_detected_provider(
        "claude-code",
        String::from("Claude Code · Local Settings"),
        String::from("local-settings"),
        settings_path.clone(),
        "cli-config",
        if settings_path.exists() {
            if has_auth {
                String::from("Claude settings.json（检测到认证字段，未读取明文）")
            } else {
                String::from("Claude settings.json")
            }
        } else {
            String::from("未检测到 Claude Code 配置文件")
        },
        model,
        settings_path.exists(),
        String::from(
            "读取 Claude Code 当前 live 配置；后续可用完整 payload 写回 settings.json 与 ~/.claude.json。",
        ),
        "claude-settings",
    );
    profile.homepage_url = origin_from_url(&request_base_url);
    profile.request_base_url = if request_base_url.trim().is_empty() {
        None
    } else {
        Some(request_base_url)
    };
    profile.config_payload = if raw.trim().is_empty() {
        Some(String::from("{}"))
    } else {
        Some(raw)
    };
    profile.auth_payload = global_raw;
    vec![profile]
}

fn detect_gemini_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let gemini_dir = home.join(".gemini");
    let env_path = gemini_dir.join(".env");
    let settings_path = gemini_dir.join("settings.json");
    let env_raw = read_small_text_file(&env_path).unwrap_or_default();
    let env = parse_env_text(&env_raw);
    let settings_raw = read_small_text_file(&settings_path).unwrap_or_default();
    let settings = serde_json::from_str::<Value>(&settings_raw).ok();
    let auth_type = settings
        .as_ref()
        .and_then(|value| json_path_string(value, &["security", "auth", "selectedType"]))
        .unwrap_or_else(|| {
            if env.contains_key("GEMINI_API_KEY") {
                String::from("gemini-api-key")
            } else {
                String::from("oauth-or-default")
            }
        });
    let model = env
        .get("GEMINI_MODEL")
        .or_else(|| env.get("MODEL"))
        .cloned()
        .or_else(|| {
            settings
                .as_ref()
                .and_then(|value| json_first_string(value, &[&["model"], &["selectedModel"]]))
        })
        .unwrap_or_else(|| String::from("gemini-2.5-pro"));
    let request_base_url = env
        .get("GOOGLE_GEMINI_BASE_URL")
        .or_else(|| env.get("GEMINI_BASE_URL"))
        .cloned()
        .unwrap_or_default();
    let config_path = if env_path.exists() {
        env_path
    } else {
        settings_path
    };

    let mut profile = build_detected_provider(
        "gemini-cli",
        format!("Gemini CLI · {auth_type}"),
        auth_type.clone(),
        config_path.clone(),
        "cli-config",
        if env.contains_key("GEMINI_API_KEY") {
            String::from("Gemini .env（检测到 GEMINI_API_KEY，未读取明文）")
        } else if config_path.exists() {
            String::from("Gemini settings.json / OAuth 配置")
        } else {
            String::from("未检测到 Gemini CLI 配置文件")
        },
        model,
        config_path.exists(),
        String::from(
            "读取 ~/.gemini/.env 与 settings.json；切换时应合并写入而不是覆盖整个 settings。",
        ),
        "gemini-config",
    );
    profile.homepage_url = origin_from_url(&request_base_url);
    profile.request_base_url = if request_base_url.trim().is_empty() {
        None
    } else {
        Some(request_base_url)
    };
    profile.config_payload = Some(if settings_raw.trim().is_empty() {
        String::from("{\n  \"security\": {\n    \"auth\": {\n      \"selectedType\": \"oauth-personal\"\n    }\n  }\n}")
    } else {
        settings_raw
    });
    profile.auth_payload = if env_raw.trim().is_empty() {
        None
    } else {
        Some(env_raw)
    };
    vec![profile]
}

fn detect_hermes_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let config_path = hermes_config_path(home);
    let Some(raw) = read_small_text_file(&config_path) else {
        return vec![build_detected_provider(
            "hermes",
            String::from("Hermes · Local Config"),
            String::from("default"),
            config_path,
            "cli-config",
            String::from("未检测到 Hermes config.yaml"),
            String::from("anthropic/claude-opus-4-8"),
            false,
            String::from("未发现 Hermes live config；保留路径方便后续导入整份 config.yaml 快照。"),
            "hermes-config",
        )];
    };

    let model_section = extract_yaml_section(&raw, "model");
    let active_provider = extract_yaml_string_field(&model_section, "provider")
        .unwrap_or_else(|| String::from("default"));
    let default_model = extract_yaml_string_field(&model_section, "default")
        .unwrap_or_else(|| String::from("anthropic/claude-opus-4-8"));
    let request_base_url =
        extract_yaml_string_field(&model_section, "base_url").unwrap_or_default();
    let has_auth = raw.contains("api_key:")
        || raw.contains("apiKey:")
        || raw.contains("key_env:")
        || raw.contains("Authorization:");

    let mut profile = build_detected_provider(
        "hermes",
        format!("Hermes · {active_provider}"),
        active_provider,
        config_path.clone(),
        "cli-config",
        if has_auth {
            String::from("Hermes config.yaml（检测到认证字段或环境变量引用，未读取明文）")
        } else {
            String::from("Hermes config.yaml")
        },
        default_model,
        true,
        String::from("读取 Hermes 当前 live config.yaml；后续可直接按整份 payload 写回。"),
        "hermes-config",
    );
    profile.homepage_url = origin_from_url(&request_base_url);
    profile.request_base_url = if request_base_url.trim().is_empty() {
        None
    } else {
        Some(request_base_url)
    };
    profile.config_payload = Some(raw);
    vec![profile]
}

fn opencode_config_candidates(home: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = std::env::var_os("OPENCODE_CONFIG") {
        paths.push(PathBuf::from(path));
    }
    paths.push(home.join(".config").join("opencode").join("opencode.json"));
    paths.push(home.join(".config").join("opencode").join("config.json"));
    paths
}

fn detect_opencode_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let config_path = opencode_config_candidates(home)
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| home.join(".config").join("opencode").join("opencode.json"));
    let Some(raw) = read_small_text_file(&config_path) else {
        return vec![build_detected_provider(
            "opencode",
            String::from("OpenCode · Local Config"),
            String::from("default"),
            config_path,
            "cli-config",
            String::from("未检测到 OpenCode 配置文件"),
            String::from("provider/default"),
            false,
            String::from("未发现 OpenCode 配置；保留路径方便后续创建或手动登记。"),
            "opencode-config",
        )];
    };

    let Ok(root) = serde_json::from_str::<Value>(&raw) else {
        return vec![build_detected_provider(
            "opencode",
            String::from("OpenCode · Local Config"),
            String::from("default"),
            config_path,
            "cli-config",
            String::from("OpenCode 配置存在但无法解析 JSON/JSON5"),
            String::from("provider/default"),
            false,
            String::from("OpenCode 可能使用 JSON5；当前后端只做保守 JSON 解析。"),
            "opencode-config",
        )];
    };

    let Some(providers) = root.get("provider").and_then(Value::as_object) else {
        return vec![build_detected_provider(
            "opencode",
            String::from("OpenCode · Local Config"),
            String::from("default"),
            config_path,
            "cli-config",
            String::from("OpenCode config detected"),
            String::from("provider/default"),
            true,
            String::from("读取 OpenCode 配置，但未发现 provider map。"),
            "opencode-config",
        )];
    };

    providers
        .iter()
        .map(|(provider_id, provider)| {
            let name = json_first_string(provider, &[&["name"], &["label"]])
                .unwrap_or_else(|| provider_id.clone());
            let model = json_first_string(provider, &[&["model"], &["models", "default"]])
                .unwrap_or_else(|| String::from("provider/default"));
            let has_auth = provider.to_string().contains("apiKey")
                || provider.to_string().contains("api_key")
                || provider.to_string().contains("token");
            build_detected_provider(
                "opencode",
                format!("OpenCode · {name}"),
                provider_id.clone(),
                config_path.clone(),
                "cli-config",
                if has_auth {
                    String::from("OpenCode provider map（检测到认证字段，未读取明文）")
                } else {
                    String::from("OpenCode provider map")
                },
                model,
                false,
                String::from("读取 opencode.json 的 provider map；OpenCode 天然支持多 Provider。"),
                "opencode-config",
            )
        })
        .collect()
}

fn detect_native_cli_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let mut profiles = Vec::new();
    profiles.extend(detect_codex_profiles(home));
    profiles.extend(detect_claude_profiles(home));
    profiles.extend(detect_gemini_profiles(home));
    profiles.extend(detect_opencode_profiles(home));
    profiles.extend(detect_hermes_profiles(home));
    profiles
}

fn provider_config_fingerprint(profile: &DetectedProviderProfile) -> String {
    profile
        .config_payload
        .as_deref()
        .map(canonicalize_provider_payload)
        .filter(|value| !value.is_empty())
        .map(|value| stable_text_hash(&value))
        .unwrap_or_default()
}

fn provider_match_score(stored: &DetectedProviderProfile, live: &DetectedProviderProfile) -> u8 {
    if stored.provider_kind != live.provider_kind {
        return 0;
    }

    let stored_payload = provider_payload_fingerprint(stored);
    let live_payload = provider_payload_fingerprint(live);
    if !stored_payload.is_empty() && stored_payload == live_payload {
        return 100;
    }

    let stored_config = provider_config_fingerprint(stored);
    let live_config = provider_config_fingerprint(live);
    if !stored_config.is_empty()
        && stored_config == live_config
        && (stored.auth_payload.is_none() || live.auth_payload.is_none())
    {
        return 95;
    }

    let same_profile = stored.profile_name.eq_ignore_ascii_case(&live.profile_name)
        || stored.name.eq_ignore_ascii_case(&live.profile_name)
        || stored.profile_name.eq_ignore_ascii_case(&live.name);
    let stored_url = normalized_provider_url(stored.request_base_url.as_deref());
    let live_url = normalized_provider_url(live.request_base_url.as_deref());
    let same_url = !stored_url.is_empty() && stored_url == live_url;
    let stored_model = normalized_provider_model(&stored.default_model);
    let live_model = normalized_provider_model(&live.default_model);
    let same_model = !stored_model.is_empty() && stored_model == live_model;

    if same_profile && (same_url || same_model) {
        return 90;
    }
    if same_url && same_model {
        return if stored.is_active { 85 } else { 80 };
    }
    if same_profile && stored.config_payload.is_none() {
        return 70;
    }
    0
}

fn merge_live_profile(stored: &mut DetectedProviderProfile, live: DetectedProviderProfile) {
    stored.config_path = live.config_path;
    stored.config_scope = live.config_scope;
    stored.auth_source = live.auth_source;
    stored.default_model = live.default_model;
    stored.tool_targets = live.tool_targets;
    stored.status = live.status;
    stored.is_active = live.is_active || stored.is_active;
    stored.homepage_url = live.homepage_url.or(stored.homepage_url.take());
    stored.request_base_url = live.request_base_url.or(stored.request_base_url.take());
    stored.config_payload = live.config_payload.or(stored.config_payload.take());
    stored.auth_payload = live.auth_payload.or(stored.auth_payload.take());
    stored.config_exists = live.config_exists;
    stored.detected_source = format!("{}+native-live", stored.detected_source);
}

fn resolve_unique_active_profiles(profiles: &mut [DetectedProviderProfile]) {
    let mut kinds = profiles
        .iter()
        .map(|profile| profile.provider_kind.clone())
        .collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();

    for kind in kinds {
        let live_active = profiles.iter().position(|profile| {
            profile.provider_kind == kind
                && profile.is_active
                && profile.config_exists
                && (profile.managed_by != "cc-switch"
                    || profile.detected_source.contains("+native-live"))
        });
        let selected = live_active.or_else(|| {
            profiles.iter().position(|profile| {
                profile.provider_kind == kind && profile.is_active && profile.config_exists
            })
        });

        for (index, profile) in profiles.iter_mut().enumerate() {
            if profile.provider_kind != kind {
                continue;
            }
            profile.is_active = Some(index) == selected;
            profile.status = if !profile.config_exists {
                String::from("missing")
            } else if profile.is_active {
                String::from("active")
            } else {
                String::from("available")
            };
        }
    }
}

fn merge_detected_provider_profiles(
    mut stored_profiles: Vec<DetectedProviderProfile>,
    mut live_profiles: Vec<DetectedProviderProfile>,
) -> Vec<DetectedProviderProfile> {
    for profile in &mut live_profiles {
        refresh_native_identity_key(profile);
    }

    for live in live_profiles {
        let best_score = stored_profiles
            .iter()
            .map(|stored| provider_match_score(stored, &live))
            .max()
            .unwrap_or(0);
        let matches = stored_profiles
            .iter()
            .enumerate()
            .filter_map(|(index, stored)| {
                (best_score > 0 && provider_match_score(stored, &live) == best_score)
                    .then_some(index)
            })
            .collect::<Vec<_>>();

        if matches.len() == 1 {
            merge_live_profile(&mut stored_profiles[matches[0]], live);
        } else {
            stored_profiles.push(live);
        }
    }

    let mut seen = std::collections::HashSet::new();
    stored_profiles.retain(|profile| seen.insert(profile.identity_key.clone()));
    resolve_unique_active_profiles(&mut stored_profiles);
    stored_profiles
}

fn reconciliation_match(
    existing: &ProviderReconciliationInput,
    canonical: &DetectedProviderProfile,
) -> Option<(u8, &'static str)> {
    if existing.provider_kind != canonical.provider_kind {
        return None;
    }

    if existing.identity_key.as_deref() == Some(canonical.identity_key.as_str()) {
        return Some((120, "exact-identity"));
    }

    let existing_payload = provider_payload_fingerprint_parts(
        existing.config_payload.as_deref(),
        existing.auth_payload.as_deref(),
    );
    let canonical_payload = provider_payload_fingerprint(canonical);
    if !existing_payload.is_empty() && existing_payload == canonical_payload {
        return Some((100, "payload-match"));
    }

    if !existing.profile_name.trim().is_empty()
        && existing
            .profile_name
            .eq_ignore_ascii_case(&canonical.profile_name)
    {
        return Some((90, "profile-name-match"));
    }

    let same_name =
        !existing.name.trim().is_empty() && existing.name.eq_ignore_ascii_case(&canonical.name);
    let same_url = !normalized_provider_url(existing.request_base_url.as_deref()).is_empty()
        && normalized_provider_url(existing.request_base_url.as_deref())
            == normalized_provider_url(canonical.request_base_url.as_deref());
    let same_model = !normalized_provider_model(&existing.default_model).is_empty()
        && normalized_provider_model(&existing.default_model)
            == normalized_provider_model(&canonical.default_model);

    if same_url && same_model {
        return Some((85, "endpoint-model-match"));
    }
    if same_name && (same_url || same_model) {
        return Some((80, "name-context-match"));
    }
    None
}

fn is_manual_provider_source(managed_by: &str) -> bool {
    matches!(managed_by, "manual" | "oauth" | "env" | "script")
}

fn is_definitely_synced_provider(existing: &ProviderReconciliationInput) -> bool {
    existing.managed_by == "cc-switch"
        || existing
            .identity_key
            .as_deref()
            .map(|key| key.contains(":ccs:") || key.contains(":native:"))
            .unwrap_or(false)
}

fn build_provider_reconciliation_preview(
    canonical_profiles: Vec<DetectedProviderProfile>,
    existing_profiles: Vec<ProviderReconciliationInput>,
) -> ProviderReconciliationPreview {
    let mut actions: Vec<Option<ProviderReconciliationAction>> =
        vec![None; existing_profiles.len()];
    let mut matches_by_target: BTreeMap<usize, Vec<(usize, u8, &'static str)>> = BTreeMap::new();

    for (existing_index, existing) in existing_profiles.iter().enumerate() {
        if is_manual_provider_source(&existing.managed_by) {
            actions[existing_index] = Some(ProviderReconciliationAction {
                existing_id: existing.id.clone(),
                action: String::from("preserve"),
                target_identity_key: None,
                reason_code: String::from("manual-source"),
            });
            continue;
        }

        let scored = canonical_profiles
            .iter()
            .enumerate()
            .filter_map(|(target_index, canonical)| {
                reconciliation_match(existing, canonical)
                    .map(|(score, reason)| (target_index, score, reason))
            })
            .collect::<Vec<_>>();
        let best_score = scored.iter().map(|(_, score, _)| *score).max().unwrap_or(0);
        let best = scored
            .into_iter()
            .filter(|(_, score, _)| *score == best_score)
            .collect::<Vec<_>>();

        if best.len() == 1 {
            let (target_index, score, reason) = best[0];
            matches_by_target.entry(target_index).or_default().push((
                existing_index,
                score,
                reason,
            ));
        } else if best.len() > 1 {
            actions[existing_index] = Some(ProviderReconciliationAction {
                existing_id: existing.id.clone(),
                action: String::from("review"),
                target_identity_key: None,
                reason_code: String::from("ambiguous-match"),
            });
        } else {
            actions[existing_index] = Some(ProviderReconciliationAction {
                existing_id: existing.id.clone(),
                action: if is_definitely_synced_provider(existing) {
                    String::from("remove-stale")
                } else {
                    String::from("review")
                },
                target_identity_key: None,
                reason_code: if is_definitely_synced_provider(existing) {
                    String::from("source-missing")
                } else {
                    String::from("unclassified-source")
                },
            });
        }
    }

    let mut matched_targets = std::collections::HashSet::new();
    for (target_index, mut candidates) in matches_by_target {
        candidates.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| {
                    existing_profiles[right.0]
                        .updated_at
                        .cmp(&existing_profiles[left.0].updated_at)
                })
                .then_with(|| left.0.cmp(&right.0))
        });
        let target_identity = canonical_profiles[target_index].identity_key.clone();
        matched_targets.insert(target_index);

        for (position, (existing_index, _, reason)) in candidates.into_iter().enumerate() {
            actions[existing_index] = Some(ProviderReconciliationAction {
                existing_id: existing_profiles[existing_index].id.clone(),
                action: if position == 0 {
                    String::from("update")
                } else {
                    String::from("remove-duplicate")
                },
                target_identity_key: Some(target_identity.clone()),
                reason_code: if position == 0 {
                    reason.to_string()
                } else {
                    String::from("duplicate-canonical-target")
                },
            });
        }
    }

    let actions = actions.into_iter().flatten().collect::<Vec<_>>();
    let mut summary = ProviderReconciliationSummary {
        existing_count: existing_profiles.len(),
        canonical_count: canonical_profiles.len(),
        add_count: canonical_profiles
            .len()
            .saturating_sub(matched_targets.len()),
        ..Default::default()
    };
    for action in &actions {
        match action.action.as_str() {
            "update" => summary.update_count += 1,
            "remove-duplicate" => summary.remove_duplicate_count += 1,
            "remove-stale" => summary.remove_stale_count += 1,
            "preserve" => summary.preserve_count += 1,
            _ => summary.review_count += 1,
        }
    }

    ProviderReconciliationPreview {
        summary,
        canonical_profiles,
        actions,
    }
}

fn kind_from_cc_switch_app(app: &str) -> Option<&'static str> {
    match app {
        "codex" => Some("codex"),
        "claude" | "claude-code" => Some("claude-code"),
        "gemini" => Some("gemini-cli"),
        "opencode" => Some("opencode"),
        "hermes" => Some("hermes"),
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

fn stable_text_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn canonicalize_provider_payload(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return serde_json::to_string(&value).unwrap_or_else(|_| trimmed.to_string());
    }
    if let Ok(value) = trimmed.parse::<toml::Value>() {
        return serde_json::to_string(&json_value_from_toml(&value))
            .unwrap_or_else(|_| trimmed.to_string());
    }
    let env = parse_env_text(trimmed);
    if !env.is_empty()
        && trimmed.lines().all(|line| {
            line.trim().is_empty() || line.trim().starts_with('#') || line.contains('=')
        })
    {
        let ordered = env.into_iter().collect::<BTreeMap<_, _>>();
        return serde_json::to_string(&ordered).unwrap_or_else(|_| trimmed.to_string());
    }
    if let Ok(value) = serde_yaml::from_str::<Value>(trimmed) {
        return serde_json::to_string(&value).unwrap_or_else(|_| trimmed.to_string());
    }
    trimmed.replace("\r\n", "\n")
}

fn provider_payload_fingerprint_parts(
    config_payload: Option<&str>,
    auth_payload: Option<&str>,
) -> String {
    let config = config_payload
        .map(canonicalize_provider_payload)
        .unwrap_or_default();
    let auth = auth_payload
        .map(canonicalize_provider_payload)
        .unwrap_or_default();
    if config.is_empty() {
        return String::new();
    }
    stable_text_hash(&format!("{config}\n--auth--\n{auth}"))
}

fn provider_payload_fingerprint(profile: &DetectedProviderProfile) -> String {
    provider_payload_fingerprint_parts(
        profile.config_payload.as_deref(),
        profile.auth_payload.as_deref(),
    )
}

fn normalized_provider_url(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .trim()
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn normalized_provider_model(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn refresh_native_identity_key(profile: &mut DetectedProviderProfile) {
    if profile.managed_by == "cc-switch" {
        return;
    }
    let fingerprint = provider_payload_fingerprint(profile);
    let identity_material = format!(
        "{}|{}|{}|{}",
        profile.profile_name,
        normalized_provider_url(profile.request_base_url.as_deref()),
        normalized_provider_model(&profile.default_model),
        fingerprint
    );
    profile.identity_key = format!(
        "{}:native:{}",
        profile.provider_kind,
        stable_text_hash(&identity_material)
    );
}

fn env_object_to_text(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let mut entries = object
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key, value)))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    Some(
        entries
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn build_hermes_payload_from_cc_switch(
    home: &Path,
    profile_name: &str,
    settings: &Value,
) -> Option<String> {
    if settings.get("model").is_some() && settings.get("custom_providers").is_some() {
        return serde_yaml::to_string(settings).ok();
    }

    let live_raw = read_small_text_file(&hermes_config_path(home)).unwrap_or_default();
    let mut root = if live_raw.trim().is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str::<serde_yaml::Value>(&live_raw).ok()?
    };
    let root_map = root.as_mapping_mut()?;

    let mut provider = settings.clone();
    let provider_object = provider.as_object_mut()?;
    provider_object.insert(
        String::from("name"),
        Value::String(profile_name.to_string()),
    );
    let model = provider_object
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            provider_object
                .get("models")
                .and_then(Value::as_object)
                .and_then(|models| models.keys().next().cloned())
        });
    let base_url = provider_object
        .get("base_url")
        .or_else(|| provider_object.get("baseUrl"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let providers_key = serde_yaml::Value::String(String::from("custom_providers"));
    let providers = root_map
        .entry(providers_key)
        .or_insert_with(|| serde_yaml::Value::Sequence(Vec::new()));
    let sequence = providers.as_sequence_mut()?;
    sequence.retain(|item| {
        item.get("name")
            .and_then(serde_yaml::Value::as_str)
            .map(|name| name != profile_name)
            .unwrap_or(true)
    });
    sequence.push(serde_yaml::to_value(provider).ok()?);

    let model_key = serde_yaml::Value::String(String::from("model"));
    let model_section = root_map
        .entry(model_key)
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let model_map = model_section.as_mapping_mut()?;
    model_map.insert(
        serde_yaml::Value::String(String::from("provider")),
        serde_yaml::Value::String(profile_name.to_string()),
    );
    if let Some(model) = model {
        model_map.insert(
            serde_yaml::Value::String(String::from("default")),
            serde_yaml::Value::String(model),
        );
    }
    if let Some(base_url) = base_url {
        model_map.insert(
            serde_yaml::Value::String(String::from("base_url")),
            serde_yaml::Value::String(base_url),
        );
    }

    serde_yaml::to_string(&root).ok()
}

fn cc_switch_payloads(
    home: &Path,
    app: &str,
    profile_name: &str,
    settings_config: &str,
) -> (Option<String>, Option<String>) {
    let Ok(settings) = serde_json::from_str::<Value>(settings_config) else {
        return (None, None);
    };
    match app {
        "codex" => (
            settings
                .get("config")
                .and_then(Value::as_str)
                .map(str::to_string),
            settings
                .get("auth")
                .filter(|value| !value.is_null())
                .and_then(|value| serde_json::to_string_pretty(value).ok()),
        ),
        "claude" => (serde_json::to_string_pretty(&settings).ok(), None),
        "gemini" => (
            settings
                .get("config")
                .and_then(|value| serde_json::to_string_pretty(value).ok()),
            settings.get("env").and_then(env_object_to_text),
        ),
        "hermes" => (
            build_hermes_payload_from_cc_switch(home, profile_name, &settings),
            None,
        ),
        _ => (None, None),
    }
}

fn request_base_url_from_profile_payload(
    provider_kind: &str,
    config_payload: Option<&str>,
    auth_payload: Option<&str>,
) -> Option<String> {
    match provider_kind {
        "codex" => {
            let raw = config_payload?;
            let provider = extract_toml_string_field(raw, "model_provider")?;
            let section = extract_toml_section(raw, &format!("model_providers.{provider}"));
            extract_toml_string_field(&section, "base_url")
        }
        "claude-code" => config_payload
            .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            .and_then(|value| json_path_string(&value, &["env", "ANTHROPIC_BASE_URL"])),
        "gemini-cli" => auth_payload.and_then(|raw| {
            let env = parse_env_text(raw);
            env.get("GOOGLE_GEMINI_BASE_URL")
                .or_else(|| env.get("GEMINI_BASE_URL"))
                .cloned()
        }),
        "hermes" => config_payload.and_then(|raw| {
            let section = extract_yaml_section(raw, "model");
            extract_yaml_string_field(&section, "base_url")
        }),
        _ => None,
    }
}

const HERMES_SQLITE_USAGE_EXPORT_SCRIPT: &str = r#"
import json
import re
import sqlite3
import sys
from urllib.parse import urlparse

db_path = sys.argv[1]
conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True, timeout=2)
conn.row_factory = sqlite3.Row

table = conn.execute(
    "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'sessions' LIMIT 1"
).fetchone()
if table is None:
    print("[]")
    raise SystemExit(0)

columns = {row["name"] for row in conn.execute("PRAGMA table_info(sessions)")}

def expr(name, fallback):
    return name if name in columns else fallback

rows = conn.execute(f"""
    SELECT
        CAST({expr('id', "''")} AS TEXT) AS id,
        {expr('model', "'unknown'")} AS model,
        {expr('billing_provider', 'NULL')} AS billing_provider,
        {expr('billing_base_url', 'NULL')} AS billing_base_url,
        COALESCE({expr('input_tokens', '0')}, 0) AS input_tokens,
        COALESCE({expr('output_tokens', '0')}, 0) AS output_tokens,
        COALESCE({expr('cache_read_tokens', '0')}, 0) AS cache_read_tokens,
        COALESCE({expr('cache_write_tokens', '0')}, 0) AS cache_creation_tokens,
        {expr('started_at', 'NULL')} AS started_at,
        {expr('ended_at', 'NULL')} AS ended_at,
        {expr('end_reason', 'NULL')} AS end_reason,
        COALESCE({expr('api_call_count', '0')}, 0) AS api_call_count
    FROM sessions
    WHERE COALESCE({expr('input_tokens', '0')}, 0)
        + COALESCE({expr('output_tokens', '0')}, 0)
        + COALESCE({expr('cache_read_tokens', '0')}, 0)
        + COALESCE({expr('cache_write_tokens', '0')}, 0) > 0
        OR COALESCE({expr('api_call_count', '0')}, 0) > 0
    ORDER BY {expr('started_at', 'rowid')} DESC
""").fetchall()

result = []
for row in rows:
    provider = (row["billing_provider"] or "").strip()
    base_url = (row["billing_base_url"] or "").strip()
    host = urlparse(base_url).hostname or ""
    provider_name = provider or host or "Hermes Native"
    provider_key = provider or host or "native"
    provider_key = re.sub(r"[^a-z0-9._-]+", "-", provider_key.lower()).strip("-") or "native"

    started_at = row["started_at"]
    ended_at = row["ended_at"]
    created_at = ""
    if started_at is not None:
        created_at = conn.execute(
            "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', ?, 'unixepoch')",
            (started_at,),
        ).fetchone()[0] or ""

    duration_ms = None
    if started_at is not None and ended_at is not None:
        duration_ms = max(0, round((float(ended_at) - float(started_at)) * 1000))

    reason = (row["end_reason"] or "").lower()
    if ended_at is None:
        status_code = 0
    elif any(word in reason for word in ("error", "fail", "exception")):
        status_code = 500
    else:
        status_code = 200

    result.append({
        "id": f"hermes:{row['id']}",
        "providerId": f"hermes:{provider_key}",
        "providerName": provider_name,
        "model": row["model"] or "unknown",
        "inputTokens": int(row["input_tokens"] or 0),
        "outputTokens": int(row["output_tokens"] or 0),
        "cacheReadTokens": int(row["cache_read_tokens"] or 0),
        "cacheCreationTokens": int(row["cache_creation_tokens"] or 0),
        "durationMs": duration_ms,
        "statusCode": status_code,
        "createdAt": created_at,
    })

print(json.dumps(result, ensure_ascii=False))
"#;

fn export_hermes_usage_with_python(home: &Path) -> Option<Vec<HermesUsageRow>> {
    let db_path = hermes_dir(home).join("state.db");
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
        args.push(HERMES_SQLITE_USAGE_EXPORT_SCRIPT);
        args.push(&db_path_text);
        let Ok(output) = Command::new(program)
            .args(args)
            .env("PYTHONIOENCODING", "utf-8")
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        if let Ok(rows) = serde_json::from_slice::<Vec<HermesUsageRow>>(&output.stdout) {
            return Some(rows);
        }
    }

    None
}

fn hermes_usage_row_to_log(row: HermesUsageRow) -> ManagedUsageRequestLog {
    managed_usage_log(
        row.id,
        row.provider_id,
        row.provider_name,
        String::from("hermes"),
        row.model,
        row.input_tokens,
        row.output_tokens,
        row.cache_read_tokens,
        row.cache_creation_tokens,
        None,
        row.duration_ms,
        row.status_code,
        String::from("hermes_session"),
        row.created_at,
    )
}

fn read_hermes_native_usage_logs(home: &Path) -> Vec<ManagedUsageRequestLog> {
    export_hermes_usage_with_python(home)
        .unwrap_or_default()
        .into_iter()
        .map(hermes_usage_row_to_log)
        .collect()
}

const CC_SWITCH_SQLITE_EXPORT_SCRIPT: &str = r#"
import json
import sqlite3
import sys

db_path = sys.argv[1]
request_log_limit = int(sys.argv[2]) if len(sys.argv) > 2 else 80
include_usage = len(sys.argv) > 3 and sys.argv[3] == "1"
request_limit_clause = f"LIMIT {request_log_limit}" if request_log_limit > 0 else ""
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
            "providerId": provider_id,
            "providerName": provider_id,
            "appType": row["app_type"],
            "model": row["model"] or "unknown",
            "inputTokens": integer(row["input_tokens"]),
            "outputTokens": integer(row["output_tokens"]),
            "cacheReadTokens": integer(row["cache_read_tokens"]),
            "cacheCreationTokens": integer(row["cache_creation_tokens"]),
            "costUsd": number(row["total_cost_usd"]),
            "statusCode": integer(row["status_code"]),
            "firstTokenMs": None,
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
            {request_limit_clause}
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
        WHERE app_type IN ('claude', 'codex', 'gemini', 'opencode', 'hermes')
        ORDER BY {", ".join(provider_order_parts)}
        """
    ):
        providers.append({
            "providerId": row["id"],
            "app": row["app_type"],
            "providerName": row["provider_name"],
            "settingsConfig": row["settings_config"] or "{}",
            "isCurrent": bool(row["is_current"]),
            "usageStats": usage_stats(row["app_type"], row["id"]) if include_usage else None,
        })

print(json.dumps(providers, ensure_ascii=False))
"#;

fn export_cc_switch_database_with_python(
    db_path: &Path,
    request_log_limit: Option<usize>,
    include_usage: bool,
) -> Option<Vec<CcSwitchDatabaseProvider>> {
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
        let request_log_limit = request_log_limit.unwrap_or(0).to_string();
        args.push(&request_log_limit);
        args.push(if include_usage { "1" } else { "0" });

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

fn cc_switch_filter_provider_ids(filters: &ManagedUsageQueryFilters) -> Vec<String> {
    filters
        .provider_ids
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|value| {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(
                    value
                        .split_once(":ccs:")
                        .map(|(_, id)| id)
                        .unwrap_or(value)
                        .to_string(),
                )
            }
        })
        .collect()
}

fn cc_switch_rollup_date_bounds(
    start_at: Option<&str>,
    end_at: Option<&str>,
) -> (Option<String>, Option<String>, bool) {
    let start = start_at.and_then(|value| {
        let local = chrono::DateTime::parse_from_rfc3339(value)
            .ok()?
            .with_timezone(&chrono::Local);
        let mut date = local.date_naive();
        if local.time() != chrono::NaiveTime::MIN {
            date = date.succ_opt()?;
        }
        Some(date.format("%Y-%m-%d").to_string())
    });
    let end = end_at.and_then(|value| {
        let local = chrono::DateTime::parse_from_rfc3339(value)
            .ok()?
            .with_timezone(&chrono::Local);
        let mut date = local.date_naive();
        if local.time() < chrono::NaiveTime::from_hms_opt(23, 59, 59)? {
            date = date.pred_opt()?;
        }
        Some(date.format("%Y-%m-%d").to_string())
    });
    let empty = matches!((&start, &end), (Some(start), Some(end)) if start > end);
    (start, end, empty)
}

const CC_SWITCH_USAGE_PARITY_EXPORT_SCRIPT: &str = r#"
import json
import sqlite3
import sys

(db_path, start_ts, end_ts, app_type, provider_name, provider_ids_json,
 model, data_source, rollup_start, rollup_end, rollup_empty) = sys.argv[1:12]
provider_ids = json.loads(provider_ids_json or "[]")
conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True, timeout=2)
conn.row_factory = sqlite3.Row

def has_table(name):
    return conn.execute("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (name,)).fetchone() is not None

def integer(value):
    try:
        return max(0, int(value or 0))
    except Exception:
        return 0

def number(value):
    try:
        return float(value or 0)
    except Exception:
        return 0.0

provider_names = {}
if has_table("providers"):
    columns = {row["name"] for row in conn.execute("PRAGMA table_info(providers)")}
    if {"id", "app_type"}.issubset(columns):
        name_expr = "name" if "name" in columns else "id"
        for row in conn.execute(f"SELECT id, app_type, {name_expr} AS name FROM providers"):
            provider_app = str(row["app_type"])
            provider_id = str(row["id"])
            provider_name_value = str(row["name"] or row["id"])
            provider_names[(provider_app, provider_id)] = provider_name_value
            if provider_app == "claude-desktop":
                provider_names[("claude", provider_id)] = provider_name_value

logs = []
if has_table("proxy_request_logs"):
    conditions = []
    args = []
    created_seconds = "CASE WHEN typeof(created_at) IN ('integer','real') THEN CAST(created_at AS INTEGER) ELSE unixepoch(created_at) END"
    if start_ts:
        conditions.append(f"{created_seconds} >= ?")
        args.append(int(start_ts))
    if end_ts:
        conditions.append(f"{created_seconds} <= ?")
        args.append(int(end_ts))
    if app_type:
        conditions.append("CASE WHEN app_type='claude-desktop' THEN 'claude' ELSE app_type END = ?")
        args.append(app_type)
    if provider_ids:
        conditions.append("provider_id IN (%s)" % ",".join("?" for _ in provider_ids))
        args.extend(provider_ids)
    if model:
        conditions.append("COALESCE(NULLIF(pricing_model, ''), model) = ?")
        args.append(model)
    if data_source:
        conditions.append("COALESCE(data_source, 'proxy') = ?")
        args.append(data_source)
    where = " WHERE " + " AND ".join(conditions) if conditions else ""
    query = f"""
        SELECT request_id, provider_id,
               CASE WHEN app_type='claude-desktop' THEN 'claude' ELSE app_type END AS app_type,
               model, COALESCE(NULLIF(pricing_model, ''), model) AS pricing_model,
               input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
               CAST(input_cost_usd AS REAL) AS input_cost_usd,
               CAST(output_cost_usd AS REAL) AS output_cost_usd,
               CAST(cache_read_cost_usd AS REAL) AS cache_read_cost_usd,
               CAST(cache_creation_cost_usd AS REAL) AS cache_creation_cost_usd,
               CAST(total_cost_usd AS REAL) AS total_cost_usd,
               first_token_ms, COALESCE(duration_ms, latency_ms) AS duration_ms,
               status_code, COALESCE(data_source, 'proxy') AS data_source,
               CASE WHEN typeof(created_at) IN ('integer','real')
                    THEN strftime('%Y-%m-%dT%H:%M:%SZ', created_at, 'unixepoch')
                    ELSE CAST(created_at AS TEXT) END AS created_at
        FROM proxy_request_logs{where}
        ORDER BY {created_seconds} DESC, request_id DESC
    """
    for row in conn.execute(query, args):
        raw_app = str(row["app_type"] or "")
        raw_provider = str(row["provider_id"] or "unknown")
        display_name = provider_names.get((raw_app, raw_provider), raw_provider)
        if provider_name and display_name != provider_name:
            continue
        logs.append({
            "id": str(row["request_id"]), "providerId": raw_provider,
            "providerName": display_name, "appType": raw_app,
            "model": str(row["model"] or "unknown"),
            "pricingModel": str(row["pricing_model"] or row["model"] or "unknown"),
            "inputTokens": integer(row["input_tokens"]),
            "outputTokens": integer(row["output_tokens"]),
            "cacheReadTokens": integer(row["cache_read_tokens"]),
            "cacheCreationTokens": integer(row["cache_creation_tokens"]),
            "inputCostUsd": number(row["input_cost_usd"]),
            "outputCostUsd": number(row["output_cost_usd"]),
            "cacheReadCostUsd": number(row["cache_read_cost_usd"]),
            "cacheCreationCostUsd": number(row["cache_creation_cost_usd"]),
            "totalCostUsd": number(row["total_cost_usd"]),
            "firstTokenMs": None if row["first_token_ms"] is None else integer(row["first_token_ms"]),
            "durationMs": None if row["duration_ms"] is None else integer(row["duration_ms"]),
            "statusCode": integer(row["status_code"]), "dataSource": str(row["data_source"]),
            "createdAt": str(row["created_at"] or ""),
        })

rollups = []
include_rollups = not data_source or data_source in ("rollup", "cc_switch_rollup")
if include_rollups and rollup_empty != "1" and has_table("usage_daily_rollups"):
    conditions = []
    args = []
    if rollup_start:
        conditions.append("date >= ?")
        args.append(rollup_start)
    if rollup_end:
        conditions.append("date <= ?")
        args.append(rollup_end)
    if app_type:
        conditions.append("CASE WHEN app_type='claude-desktop' THEN 'claude' ELSE app_type END = ?")
        args.append(app_type)
    if provider_ids:
        conditions.append("provider_id IN (%s)" % ",".join("?" for _ in provider_ids))
        args.extend(provider_ids)
    if model:
        conditions.append("COALESCE(NULLIF(pricing_model, ''), model) = ?")
        args.append(model)
    where = " WHERE " + " AND ".join(conditions) if conditions else ""
    query = f"""
        SELECT date, provider_id,
               CASE WHEN app_type='claude-desktop' THEN 'claude' ELSE app_type END AS app_type,
               COALESCE(NULLIF(pricing_model, ''), model) AS model,
               SUM(request_count) AS request_count, SUM(input_tokens) AS input_tokens,
               SUM(output_tokens) AS output_tokens, SUM(cache_read_tokens) AS cache_read_tokens,
               SUM(cache_creation_tokens) AS cache_creation_tokens,
               SUM(CAST(total_cost_usd AS REAL)) AS total_cost_usd
        FROM usage_daily_rollups{where}
        GROUP BY date, app_type, provider_id, COALESCE(NULLIF(pricing_model, ''), model)
        ORDER BY date ASC
    """
    for row in conn.execute(query, args):
        raw_app = str(row["app_type"] or "")
        raw_provider = str(row["provider_id"] or "unknown")
        display_name = provider_names.get((raw_app, raw_provider), raw_provider)
        if provider_name and display_name != provider_name:
            continue
        rollups.append({
            "providerId": raw_provider, "providerName": display_name, "appType": raw_app,
            "model": str(row["model"] or "unknown"), "requestCount": integer(row["request_count"]),
            "inputTokens": integer(row["input_tokens"]), "outputTokens": integer(row["output_tokens"]),
            "cacheReadTokens": integer(row["cache_read_tokens"]),
            "cacheCreationTokens": integer(row["cache_creation_tokens"]),
            "totalCostUsd": number(row["total_cost_usd"]),
            "createdAt": f"{row['date']}T00:00:00Z",
        })

print(json.dumps({"logs": logs, "rollups": rollups}, ensure_ascii=False))
"#;

fn query_cc_switch_usage_db(
    db_path: &Path,
    filters: &ManagedUsageQueryFilters,
) -> Result<CcSwitchUsageData, String> {
    if !db_path.exists() {
        return Ok(CcSwitchUsageData::default());
    }
    let provider_ids = cc_switch_filter_provider_ids(filters);
    let (rollup_start, rollup_end, rollup_empty) =
        cc_switch_rollup_date_bounds(filters.start_at.as_deref(), filters.end_at.as_deref());
    let arguments = vec![
        db_path.to_string_lossy().to_string(),
        filters
            .start_at
            .as_deref()
            .and_then(parse_iso_timestamp_seconds)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        filters
            .end_at
            .as_deref()
            .and_then(parse_iso_timestamp_seconds)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        filters.app_type.clone().unwrap_or_default(),
        filters.provider_name.clone().unwrap_or_default(),
        serde_json::to_string(&provider_ids).unwrap_or_else(|_| String::from("[]")),
        filters.model.clone().unwrap_or_default(),
        filters.data_source.clone().unwrap_or_default(),
        rollup_start.unwrap_or_default(),
        rollup_end.unwrap_or_default(),
        if rollup_empty { "1" } else { "0" }.to_string(),
    ];
    let attempts: [(&str, &[&str]); 3] = [
        ("python", &["-c"]),
        ("py", &["-3", "-c"]),
        ("python3", &["-c"]),
    ];
    for (program, prefix) in attempts {
        let output = Command::new(program)
            .args(prefix)
            .arg(CC_SWITCH_USAGE_PARITY_EXPORT_SCRIPT)
            .args(&arguments)
            .env("PYTHONIOENCODING", "utf-8")
            .output();
        let Ok(output) = output else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let Ok(export) = serde_json::from_slice::<CcSwitchUsageExport>(&output.stdout) else {
            continue;
        };
        let logs = export
            .logs
            .into_iter()
            .map(|row| {
                let provider_kind = kind_from_cc_switch_app(&row.app_type).unwrap_or(&row.app_type);
                let mut log = managed_usage_log(
                    row.id,
                    format!("{provider_kind}:ccs:{}", row.provider_id),
                    row.provider_name,
                    row.app_type,
                    row.model,
                    row.input_tokens,
                    row.output_tokens,
                    row.cache_read_tokens,
                    row.cache_creation_tokens,
                    row.first_token_ms,
                    row.duration_ms,
                    row.status_code,
                    row.data_source,
                    row.created_at,
                );
                log.pricing_model = row.pricing_model;
                log.input_cost_usd = row.input_cost_usd;
                log.output_cost_usd = row.output_cost_usd;
                log.cache_read_cost_usd = row.cache_read_cost_usd;
                log.cache_creation_cost_usd = row.cache_creation_cost_usd;
                log.total_cost_usd = row.total_cost_usd;
                log
            })
            .collect();
        let rollups = export
            .rollups
            .into_iter()
            .map(|row| ManagedUsageAggregateRow {
                provider_id: format!(
                    "{}:ccs:{}",
                    kind_from_cc_switch_app(&row.app_type).unwrap_or(&row.app_type),
                    row.provider_id
                ),
                provider_name: row.provider_name,
                app_type: row.app_type,
                model: row.model,
                request_count: row.request_count,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cache_read_tokens: row.cache_read_tokens,
                cache_creation_tokens: row.cache_creation_tokens,
                total_cost_usd: row.total_cost_usd,
                data_source: String::from("cc_switch_rollup"),
                created_at: row.created_at,
            })
            .collect();
        return Ok(CcSwitchUsageData { logs, rollups });
    }
    Err(String::from("无法执行 CC Switch Usage SQLite 只读查询。"))
}

fn read_cc_switch_database_providers(home: &Path, db_path: &Path) -> Vec<DetectedProviderProfile> {
    if !db_path.exists() {
        return Vec::new();
    }

    let Some(rows) = export_cc_switch_database_with_python(db_path, Some(80), false) else {
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
            profile.identity_key =
                format!("{}:ccs:{}", profile.provider_kind, profile.profile_name);
            let (config_payload, auth_payload) = cc_switch_payloads(
                home,
                &entry.app,
                &profile.profile_name,
                &entry.settings_config,
            );
            profile.config_payload = config_payload;
            profile.auth_payload = auth_payload;
            profile.request_base_url = request_base_url_from_profile_payload(
                &profile.provider_kind,
                profile.config_payload.as_deref(),
                profile.auth_payload.as_deref(),
            );
            profile.homepage_url = profile
                .request_base_url
                .as_deref()
                .and_then(origin_from_url);
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

fn collect_cc_switch_usage(home: &Path, filters: &ManagedUsageQueryFilters) -> CcSwitchUsageData {
    for db_path in cc_switch_database_paths(home) {
        if let Ok(data) = query_cc_switch_usage_db(&db_path, filters) {
            return data;
        }
    }
    CcSwitchUsageData::default()
}

fn collect_managed_usage_logs_uncached(
    home: &Path,
    filters: &ManagedUsageQueryFilters,
) -> (Vec<ManagedUsageRequestLog>, Vec<ManagedUsageAggregateRow>) {
    let mut logs = collect_native_usage_logs(home);
    let mut cc_switch = collect_cc_switch_usage(home, filters);
    logs.append(&mut cc_switch.logs);
    let provider_catalog = current_provider_catalog(home);
    for log in &mut logs {
        enrich_usage_provider_identity_from_catalog(&provider_catalog, log);
    }
    for rollup in &mut cc_switch.rollups {
        enrich_usage_aggregate_identity_from_catalog(&provider_catalog, rollup);
    }
    let original_logs = logs.clone();
    logs.retain(|log| should_keep_usage_log(log, &original_logs));
    (logs, cc_switch.rollups)
}

fn managed_usage_cache_key(home: &Path, filters: &ManagedUsageQueryFilters) -> String {
    let mut data_filters = filters.clone();
    data_filters.cursor = None;
    data_filters.limit = None;
    format!(
        "{}|{}",
        home.display(),
        serde_json::to_string(&data_filters).unwrap_or_default()
    )
}

fn collect_managed_usage_logs_cached(
    home: &Path,
    filters: &ManagedUsageQueryFilters,
) -> (Vec<ManagedUsageRequestLog>, Vec<ManagedUsageAggregateRow>) {
    let key = managed_usage_cache_key(home, filters);
    let Ok(mut cache) = MANAGED_USAGE_LOG_CACHE.lock() else {
        return collect_managed_usage_logs_uncached(home, filters);
    };
    if let Some(cached) = cache.as_ref() {
        if cached.key == key && cached.refreshed_at.elapsed() < Duration::from_secs(10) {
            return (cached.logs.clone(), cached.rollups.clone());
        }
    }

    let (logs, rollups) = collect_managed_usage_logs_uncached(home, filters);
    *cache = Some(ManagedUsageLogCache {
        key,
        refreshed_at: Instant::now(),
        logs: logs.clone(),
        rollups: rollups.clone(),
    });
    (logs, rollups)
}

fn build_managed_usage_query_result(
    home: &Path,
    filters: ManagedUsageQueryFilters,
) -> ManagedUsageQueryResult {
    if filters
        .provider_ids
        .as_ref()
        .is_some_and(|provider_ids| provider_ids.is_empty())
    {
        return build_managed_usage_query_result_from_data(Vec::new(), Vec::new(), filters);
    }
    ensure_model_pricing_loaded(home);
    let (logs, rollups) = collect_managed_usage_logs_cached(home, &filters);

    build_managed_usage_query_result_from_data(logs, rollups, filters)
}

fn encode_managed_usage_cursor(log: &ManagedUsageRequestLog) -> Option<String> {
    serde_json::to_string(&ManagedUsageCursor {
        created_at: log.created_at.clone(),
        id: log.id.clone(),
    })
    .ok()
}

fn managed_usage_log_is_after_cursor(
    log: &ManagedUsageRequestLog,
    cursor: &ManagedUsageCursor,
) -> bool {
    log.created_at < cursor.created_at
        || (log.created_at == cursor.created_at && log.id < cursor.id)
}

fn build_managed_usage_query_result_from_data(
    mut logs: Vec<ManagedUsageRequestLog>,
    mut rollups: Vec<ManagedUsageAggregateRow>,
    filters: ManagedUsageQueryFilters,
) -> ManagedUsageQueryResult {
    let bucket = filters.bucket.as_deref().unwrap_or("day");
    let limit = filters
        .limit
        .filter(|limit| *limit > 0)
        .unwrap_or(120)
        .min(1000);
    let app_type = filters.app_type.as_deref().map(str::trim).unwrap_or("");
    let provider_name = filters
        .provider_name
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let provider_filter_requested = filters.provider_ids.is_some();
    let provider_ids = filters
        .provider_ids
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<std::collections::HashSet<_>>();
    let model = filters.model.as_deref().map(str::trim).unwrap_or("");
    let data_source = filters.data_source.as_deref().map(str::trim).unwrap_or("");
    let start_at = filters.start_at.as_deref();
    let end_at = filters.end_at.as_deref();

    logs.retain(|log| {
        (app_type.is_empty() || log.app_type == app_type)
            && (provider_name.is_empty() || log.provider_name == provider_name)
            && (!provider_filter_requested || provider_ids.contains(log.provider_id.as_str()))
            && (model.is_empty() || log.model == model || log.pricing_model == model)
            && (data_source.is_empty() || log.data_source == data_source)
            && timestamp_in_range(&log.created_at, start_at, end_at)
    });
    rollups.retain(|row| {
        (app_type.is_empty() || row.app_type == app_type)
            && (provider_name.is_empty() || row.provider_name == provider_name)
            && (!provider_filter_requested || provider_ids.contains(row.provider_id.as_str()))
            && (model.is_empty() || row.model == model)
            && (data_source.is_empty() || data_source == "rollup" || row.data_source == data_source)
            && timestamp_in_range(&row.created_at, start_at, end_at)
    });

    logs.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.id.cmp(&left.id))
    });

    let mut summary = DetectedProviderUsageSummary {
        total_requests: 0,
        total_cost_usd: 0.0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cache_read_tokens: 0,
        total_cache_creation_tokens: 0,
        cache_hit_rate: 0.0,
    };
    let mut channels: BTreeMap<String, ManagedUsageChannelStat> = BTreeMap::new();
    let mut trends: BTreeMap<String, ManagedUsageTrendPoint> = BTreeMap::new();
    let mut provider_stats: BTreeMap<String, ManagedUsageProviderAccumulator> = BTreeMap::new();
    let mut model_stats: BTreeMap<String, ManagedUsageModelAccumulator> = BTreeMap::new();

    for log in &logs {
        summary.total_requests += 1;
        summary.total_cost_usd += log.total_cost_usd;
        summary.total_input_tokens += log.input_tokens;
        summary.total_output_tokens += log.output_tokens;
        summary.total_cache_read_tokens += log.cache_read_tokens;
        summary.total_cache_creation_tokens += log.cache_creation_tokens;

        let channel_id = format!("{}:{}", log.app_type, log.data_source);
        let channel = channels
            .entry(channel_id.clone())
            .or_insert(ManagedUsageChannelStat {
                channel_id,
                app_type: log.app_type.clone(),
                data_source: log.data_source.clone(),
                total_requests: 0,
                total_cost_usd: 0.0,
                total_input_tokens: 0,
                total_output_tokens: 0,
            });
        channel.total_requests += 1;
        channel.total_cost_usd += log.total_cost_usd;
        channel.total_input_tokens += log.input_tokens;
        channel.total_output_tokens += log.output_tokens;

        let trend_key = bucket_usage_timestamp(&log.created_at, bucket);
        let trend = trends
            .entry(trend_key.clone())
            .or_insert(ManagedUsageTrendPoint {
                timestamp: trend_key,
                request_count: 0,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.0,
            });
        trend.request_count += 1;
        trend.input_tokens += log.input_tokens;
        trend.output_tokens += log.output_tokens;
        trend.cache_read_tokens += log.cache_read_tokens;
        trend.cache_creation_tokens += log.cache_creation_tokens;
        trend.cost_usd += log.total_cost_usd;

        let provider_key = format!("{}:{}", log.app_type, log.provider_id);
        let provider =
            provider_stats
                .entry(provider_key)
                .or_insert_with(|| ManagedUsageProviderAccumulator {
                    stat: ManagedUsageProviderStat {
                        provider_id: log.provider_id.clone(),
                        provider_name: log.provider_name.clone(),
                        app_type: log.app_type.clone(),
                        total_requests: 0,
                        total_cost_usd: 0.0,
                        total_input_tokens: 0,
                        total_output_tokens: 0,
                        total_cache_read_tokens: 0,
                        total_cache_creation_tokens: 0,
                        cache_hit_rate: 0.0,
                        model_count: 0,
                        models: Vec::new(),
                        data_sources: Vec::new(),
                        last_activity_at: log.created_at.clone(),
                    },
                    models: std::collections::BTreeSet::new(),
                    data_sources: std::collections::BTreeSet::new(),
                });
        provider.stat.total_requests += 1;
        provider.stat.total_cost_usd += log.total_cost_usd;
        provider.stat.total_input_tokens += log.input_tokens;
        provider.stat.total_output_tokens += log.output_tokens;
        provider.stat.total_cache_read_tokens += log.cache_read_tokens;
        provider.stat.total_cache_creation_tokens += log.cache_creation_tokens;
        if log.created_at > provider.stat.last_activity_at {
            provider.stat.last_activity_at = log.created_at.clone();
        }
        let usage_model = if log.model.trim().is_empty() || log.model == "unknown" {
            log.pricing_model.trim()
        } else {
            log.model.trim()
        };
        if !usage_model.is_empty() && usage_model != "unknown" {
            provider.models.insert(usage_model.to_string());
        }
        if !log.data_source.trim().is_empty() {
            provider.data_sources.insert(log.data_source.clone());
        }

        let usage_model = if usage_model.is_empty() {
            "unknown"
        } else {
            usage_model
        };
        let model = model_stats
            .entry(usage_model.to_string())
            .or_insert_with(|| ManagedUsageModelAccumulator {
                stat: ManagedUsageModelStat {
                    model: usage_model.to_string(),
                    total_requests: 0,
                    total_cost_usd: 0.0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_cache_read_tokens: 0,
                    total_cache_creation_tokens: 0,
                    provider_count: 0,
                    provider_names: Vec::new(),
                    app_types: Vec::new(),
                    last_activity_at: log.created_at.clone(),
                },
                provider_names: std::collections::BTreeSet::new(),
                app_types: std::collections::BTreeSet::new(),
            });
        model.stat.total_requests += 1;
        model.stat.total_cost_usd += log.total_cost_usd;
        model.stat.total_input_tokens += log.input_tokens;
        model.stat.total_output_tokens += log.output_tokens;
        model.stat.total_cache_read_tokens += log.cache_read_tokens;
        model.stat.total_cache_creation_tokens += log.cache_creation_tokens;
        if log.created_at > model.stat.last_activity_at {
            model.stat.last_activity_at = log.created_at.clone();
        }
        if !log.provider_name.trim().is_empty() {
            model.provider_names.insert(log.provider_name.clone());
        }
        if !log.app_type.trim().is_empty() {
            model.app_types.insert(log.app_type.clone());
        }
    }

    for row in &rollups {
        summary.total_requests += row.request_count;
        summary.total_cost_usd += row.total_cost_usd;
        summary.total_input_tokens += row.input_tokens;
        summary.total_output_tokens += row.output_tokens;
        summary.total_cache_read_tokens += row.cache_read_tokens;
        summary.total_cache_creation_tokens += row.cache_creation_tokens;

        let trend_key = bucket_usage_timestamp(&row.created_at, bucket);
        let trend = trends
            .entry(trend_key.clone())
            .or_insert(ManagedUsageTrendPoint {
                timestamp: trend_key,
                request_count: 0,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.0,
            });
        trend.request_count += row.request_count;
        trend.input_tokens += row.input_tokens;
        trend.output_tokens += row.output_tokens;
        trend.cache_read_tokens += row.cache_read_tokens;
        trend.cache_creation_tokens += row.cache_creation_tokens;
        trend.cost_usd += row.total_cost_usd;

        let provider_key = format!("{}:{}", row.app_type, row.provider_id);
        let provider =
            provider_stats
                .entry(provider_key)
                .or_insert_with(|| ManagedUsageProviderAccumulator {
                    stat: ManagedUsageProviderStat {
                        provider_id: row.provider_id.clone(),
                        provider_name: row.provider_name.clone(),
                        app_type: row.app_type.clone(),
                        total_requests: 0,
                        total_cost_usd: 0.0,
                        total_input_tokens: 0,
                        total_output_tokens: 0,
                        total_cache_read_tokens: 0,
                        total_cache_creation_tokens: 0,
                        cache_hit_rate: 0.0,
                        model_count: 0,
                        models: Vec::new(),
                        data_sources: Vec::new(),
                        last_activity_at: row.created_at.clone(),
                    },
                    models: std::collections::BTreeSet::new(),
                    data_sources: std::collections::BTreeSet::new(),
                });
        provider.stat.total_requests += row.request_count;
        provider.stat.total_cost_usd += row.total_cost_usd;
        provider.stat.total_input_tokens += row.input_tokens;
        provider.stat.total_output_tokens += row.output_tokens;
        provider.stat.total_cache_read_tokens += row.cache_read_tokens;
        provider.stat.total_cache_creation_tokens += row.cache_creation_tokens;
        if row.created_at > provider.stat.last_activity_at {
            provider.stat.last_activity_at = row.created_at.clone();
        }
        if !row.model.trim().is_empty() && row.model != "unknown" {
            provider.models.insert(row.model.clone());
        }

        let usage_model = if row.model.trim().is_empty() {
            "unknown"
        } else {
            row.model.trim()
        };
        let model = model_stats
            .entry(usage_model.to_string())
            .or_insert_with(|| ManagedUsageModelAccumulator {
                stat: ManagedUsageModelStat {
                    model: usage_model.to_string(),
                    total_requests: 0,
                    total_cost_usd: 0.0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_cache_read_tokens: 0,
                    total_cache_creation_tokens: 0,
                    provider_count: 0,
                    provider_names: Vec::new(),
                    app_types: Vec::new(),
                    last_activity_at: row.created_at.clone(),
                },
                provider_names: std::collections::BTreeSet::new(),
                app_types: std::collections::BTreeSet::new(),
            });
        model.stat.total_requests += row.request_count;
        model.stat.total_cost_usd += row.total_cost_usd;
        model.stat.total_input_tokens += row.input_tokens;
        model.stat.total_output_tokens += row.output_tokens;
        model.stat.total_cache_read_tokens += row.cache_read_tokens;
        model.stat.total_cache_creation_tokens += row.cache_creation_tokens;
        if row.created_at > model.stat.last_activity_at {
            model.stat.last_activity_at = row.created_at.clone();
        }
        if !row.provider_name.trim().is_empty() {
            model.provider_names.insert(row.provider_name.clone());
        }
        if !row.app_type.trim().is_empty() {
            model.app_types.insert(row.app_type.clone());
        }
    }

    let cacheable = summary.total_input_tokens
        + summary.total_cache_read_tokens
        + summary.total_cache_creation_tokens;
    summary.cache_hit_rate = if cacheable == 0 {
        0.0
    } else {
        summary.total_cache_read_tokens as f64 / cacheable as f64
    };

    let mut provider_stats = provider_stats
        .into_values()
        .map(|mut provider| {
            let cacheable = provider.stat.total_input_tokens
                + provider.stat.total_cache_read_tokens
                + provider.stat.total_cache_creation_tokens;
            provider.stat.cache_hit_rate = if cacheable == 0 {
                0.0
            } else {
                provider.stat.total_cache_read_tokens as f64 / cacheable as f64
            };
            provider.stat.models = provider.models.into_iter().collect();
            provider.stat.model_count = provider.stat.models.len();
            provider.stat.data_sources = provider.data_sources.into_iter().collect();
            provider.stat
        })
        .collect::<Vec<_>>();
    provider_stats.sort_by(|left, right| {
        right
            .total_requests
            .cmp(&left.total_requests)
            .then_with(|| left.provider_name.cmp(&right.provider_name))
    });

    let mut model_stats = model_stats
        .into_values()
        .map(|mut model| {
            model.stat.provider_names = model.provider_names.into_iter().collect();
            model.stat.provider_count = model.stat.provider_names.len();
            model.stat.app_types = model.app_types.into_iter().collect();
            model.stat
        })
        .collect::<Vec<_>>();
    model_stats.sort_by(|left, right| {
        right
            .total_cost_usd
            .partial_cmp(&left.total_cost_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.total_requests.cmp(&left.total_requests))
            .then_with(|| left.model.cmp(&right.model))
    });

    let total = logs.len();
    let cursor = filters
        .cursor
        .as_deref()
        .and_then(|value| serde_json::from_str::<ManagedUsageCursor>(value).ok());
    let mut request_logs = logs
        .into_iter()
        .filter(|log| {
            cursor
                .as_ref()
                .map(|cursor| managed_usage_log_is_after_cursor(log, cursor))
                .unwrap_or(true)
        })
        .take(limit + 1)
        .collect::<Vec<_>>();
    let has_more = request_logs.len() > limit;
    if has_more {
        request_logs.truncate(limit);
    }
    let next_cursor = if has_more {
        request_logs.last().and_then(encode_managed_usage_cursor)
    } else {
        None
    };

    ManagedUsageQueryResult {
        summary,
        channels: channels.into_values().collect(),
        trends: trends.into_values().collect(),
        provider_stats,
        model_stats,
        request_logs,
        next_cursor,
        has_more,
        total,
        updated_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn collect_cc_switch_provider_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    let mut stored_profiles = Vec::new();
    let cc_switch_path = cc_switch_config_path(home);
    let mut found_cc_switch_database_profiles = false;

    for db_path in cc_switch_database_paths(home) {
        let mut database_profiles = read_cc_switch_database_providers(home, &db_path);
        if !database_profiles.is_empty() {
            found_cc_switch_database_profiles = true;
            stored_profiles.append(&mut database_profiles);
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
                let mut profile = build_detected_provider(
                    provider_kind,
                    entry.provider_name,
                    profile_name,
                    cc_switch_path.clone(),
                    "cc-switch",
                    String::from("CC Switch 本地配置档案"),
                    entry.default_model,
                    entry.is_current,
                    format!("从 {} 只读发现。", display_home_path(&cc_switch_path)),
                    "cc-switch",
                );
                profile.identity_key =
                    format!("{}:ccs:{}", profile.provider_kind, profile.profile_name);
                stored_profiles.push(profile);
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    stored_profiles.retain(|profile| seen.insert(profile.identity_key.clone()));
    resolve_unique_active_profiles(&mut stored_profiles);
    stored_profiles
}

fn collect_native_provider_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    merge_detected_provider_profiles(Vec::new(), detect_native_cli_profiles(home))
}

fn collect_detected_provider_profiles(home: &Path) -> Vec<DetectedProviderProfile> {
    merge_detected_provider_profiles(
        collect_cc_switch_provider_profiles(home),
        detect_native_cli_profiles(home),
    )
}

#[tauri::command]
async fn detect_local_provider_profiles() -> Result<Vec<DetectedProviderProfile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        Ok(collect_detected_provider_profiles(&home))
    })
    .await
    .map_err(|error| format!("扫描 Provider 配置任务失败：{error}"))?
}

#[tauri::command]
async fn detect_native_provider_profiles() -> Result<Vec<DetectedProviderProfile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        Ok(collect_native_provider_profiles(&home))
    })
    .await
    .map_err(|error| format!("扫描本机 Provider 配置任务失败：{error}"))?
}

#[tauri::command]
async fn detect_cc_switch_provider_profiles() -> Result<Vec<DetectedProviderProfile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        Ok(collect_cc_switch_provider_profiles(&home))
    })
    .await
    .map_err(|error| format!("扫描 CC Switch Provider 配置任务失败：{error}"))?
}

#[tauri::command]
async fn preview_provider_reconciliation(
    existing_profiles: Vec<ProviderReconciliationInput>,
) -> Result<ProviderReconciliationPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        Ok(build_provider_reconciliation_preview(
            collect_detected_provider_profiles(&home),
            existing_profiles,
        ))
    })
    .await
    .map_err(|error| format!("生成 Provider 清理预览任务失败：{error}"))?
}

#[tauri::command]
async fn read_current_codex_profile() -> Result<DetectedProviderProfile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        detect_codex_profiles(&home)
            .into_iter()
            .next()
            .ok_or_else(|| String::from("未能读取当前 Codex 配置。"))
    })
    .await
    .map_err(|error| format!("读取当前 Codex 配置任务失败：{error}"))?
}

#[tauri::command]
async fn read_current_gemini_profile() -> Result<DetectedProviderProfile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        detect_gemini_profiles(&home)
            .into_iter()
            .next()
            .ok_or_else(|| String::from("未能读取当前 Gemini 配置。"))
    })
    .await
    .map_err(|error| format!("读取当前 Gemini 配置任务失败：{error}"))?
}

#[tauri::command]
async fn read_current_claude_profile() -> Result<DetectedProviderProfile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        detect_claude_profiles(&home)
            .into_iter()
            .next()
            .ok_or_else(|| String::from("未能读取当前 Claude 配置。"))
    })
    .await
    .map_err(|error| format!("读取当前 Claude 配置任务失败：{error}"))?
}

#[tauri::command]
async fn read_current_hermes_profile() -> Result<DetectedProviderProfile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        detect_hermes_profiles(&home)
            .into_iter()
            .next()
            .ok_or_else(|| String::from("未能读取当前 Hermes 配置。"))
    })
    .await
    .map_err(|error| format!("读取当前 Hermes 配置任务失败：{error}"))?
}

#[tauri::command]
async fn read_managed_mcp_servers() -> Result<Vec<ManagedMcpServer>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let mut servers = Vec::new();
        servers.extend(read_claude_mcp_servers(&home)?);
        servers.extend(read_codex_mcp_servers(&home)?);
        servers.extend(read_gemini_mcp_servers(&home)?);
        servers.extend(read_hermes_mcp_servers(&home)?);
        Ok(servers)
    })
    .await
    .map_err(|error| format!("读取 MCP 配置任务失败：{error}"))?
}

#[tauri::command]
async fn apply_managed_mcp_servers(servers: Vec<ManagedMcpServer>) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let mut affected_paths = Vec::new();
        affected_paths.push(write_claude_mcp_servers(&home, &servers)?);
        affected_paths.push(write_codex_mcp_servers(&home, &servers)?);
        affected_paths.push(write_gemini_mcp_servers(&home, &servers)?);
        affected_paths.push(write_hermes_mcp_servers(&home, &servers)?);
        affected_paths.sort();
        affected_paths.dedup();
        Ok(affected_paths)
    })
    .await
    .map_err(|error| format!("写回 MCP 配置任务失败：{error}"))?
}

#[tauri::command]
async fn read_managed_prompt_profiles(
    tool_kinds: Vec<String>,
) -> Result<Vec<ManagedPromptProfile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let tool_kinds = if tool_kinds.is_empty() {
            vec![
                String::from("claude"),
                String::from("codex"),
                String::from("gemini"),
                String::from("hermes"),
            ]
        } else {
            tool_kinds
        };
        read_managed_prompt_profiles_for_tools(&home, &tool_kinds)
    })
    .await
    .map_err(|error| format!("读取 Prompt 配置任务失败：{error}"))?
}

#[tauri::command]
async fn apply_managed_prompt_profile(
    tool_kind: String,
    content: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        write_managed_prompt_profile(&home, tool_kind.trim(), &content)
    })
    .await
    .map_err(|error| format!("写回 Prompt 配置任务失败：{error}"))?
}

#[tauri::command]
async fn query_managed_usage(
    filters: ManagedUsageQueryFilters,
) -> Result<ManagedUsageQueryResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        Ok(build_managed_usage_query_result(&home, filters))
    })
    .await
    .map_err(|error| format!("查询 Usage 数据任务失败：{error}"))?
}

#[tauri::command]
async fn get_model_pricing() -> Result<Vec<ManagedModelPricingEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        all_model_pricing_entries(&home)
    })
    .await
    .map_err(|error| format!("读取模型价格任务失败：{error}"))?
}

#[tauri::command]
async fn update_model_pricing(
    app: AppHandle,
    model_id: String,
    display_name: String,
    vendor: Option<String>,
    input_cost_per_million: f64,
    output_cost_per_million: f64,
    cache_read_cost_per_million: f64,
    cache_creation_cost_per_million: f64,
    completion_cost_per_million: Option<f64>,
    image_cost_per_million: Option<f64>,
    video_cost_per_million: Option<f64>,
    audio_cost_per_million: Option<f64>,
) -> Result<Vec<ManagedModelPricingEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let model_id = model_id.trim().to_string();
        let mut entry = ManagedModelPricingEntry {
            display_name: display_name.trim().to_string(),
            vendor: vendor.unwrap_or_default().trim().to_string(),
            input_cost_per_million,
            output_cost_per_million,
            cache_read_cost_per_million,
            cache_creation_cost_per_million,
            completion_cost_per_million: completion_cost_per_million.unwrap_or(0.0),
            image_cost_per_million: image_cost_per_million.unwrap_or(0.0),
            video_cost_per_million: video_cost_per_million.unwrap_or(0.0),
            audio_cost_per_million: audio_cost_per_million.unwrap_or(0.0),
            source: String::from("user"),
            updated_at: chrono::Utc::now().to_rfc3339(),
            model_id,
        };
        if entry.vendor.is_empty() {
            entry.vendor = model_pricing_vendor(&entry.model_id);
        }
        validate_model_pricing(&entry)?;
        execute_model_pricing_db(
            &home,
            "upsert",
            &serde_json::to_value(&entry).map_err(|error| error.to_string())?,
        )?;
        let entries = all_model_pricing_entries(&home)?;
        let _ = std::fs::remove_file(native_usage_snapshot_path(&home));
        invalidate_managed_usage_cache();
        let _ = app.emit("model-pricing-updated", &entry.model_id);
        Ok(entries)
    })
    .await
    .map_err(|error| format!("更新模型价格任务失败：{error}"))?
}

#[tauri::command]
async fn delete_model_pricing(
    app: AppHandle,
    model_id: String,
) -> Result<Vec<ManagedModelPricingEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let model_id = model_id.trim().to_string();
        if model_id.is_empty() {
            return Err(String::from("模型 ID 不能为空。"));
        }
        execute_model_pricing_db(&home, "delete", &Value::String(model_id.clone()))?;
        let entries = all_model_pricing_entries(&home)?;
        let _ = std::fs::remove_file(native_usage_snapshot_path(&home));
        invalidate_managed_usage_cache();
        let _ = app.emit("model-pricing-updated", &model_id);
        Ok(entries)
    })
    .await
    .map_err(|error| format!("删除模型价格任务失败：{error}"))?
}

#[tauri::command]
async fn apply_provider_profile(
    provider_kind: String,
    profile_name: String,
    switch_command: Option<String>,
    config_payload: Option<String>,
    auth_payload: Option<String>,
) -> Result<AppliedProviderProfileResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let home = home_dir().ok_or_else(|| String::from("无法定位用户目录。"))?;
        let kind = provider_kind.trim().to_string();
        let profile = if profile_name.trim().is_empty() {
            String::from("default")
        } else {
            profile_name.trim().to_string()
        };
        let command = apply_provider_command_text(&kind, &profile, switch_command);
        let active_sessions = count_active_terminal_sessions();

        if command.trim().is_empty() {
            if kind == "codex" {
                let (config_path, applied_command) = if let Some(payload) =
                    config_payload.as_deref().filter(|payload| !payload.trim().is_empty())
                {
                    (
                        write_codex_payload_to_live_config(
                            &home,
                            payload,
                            auth_payload.as_deref(),
                        )?,
                        String::from("native:codex-config-payload"),
                    )
                } else {
                    (
                        write_codex_profile_to_live_config(&home, &profile)?,
                        String::from("native:codex-config"),
                    )
                };
                return Ok(AppliedProviderProfileResult {
                    provider_kind: kind,
                    profile_name: profile,
                    applied_command,
                    stdout: format!("已写入 {config_path}"),
                    stderr: String::new(),
                    requires_restart: true,
                    restart_hint: format!(
                        "Codex CLI 通常需要重启后读取新的 config.toml；当前 Chuchen 有 {active_sessions} 个终端会话，后续可接入精准识别和一键重启。"
                    ),
                    affected_session_count: active_sessions,
                });
            }
            if kind == "gemini-cli" {
                let config_path = write_gemini_payload_to_live_config(
                    &home,
                    config_payload.as_deref().unwrap_or(""),
                    auth_payload.as_deref(),
                )?;
                return Ok(AppliedProviderProfileResult {
                    provider_kind: kind,
                    profile_name: profile,
                    applied_command: String::from("native:gemini-config-payload"),
                    stdout: format!("已写入 {config_path}"),
                    stderr: String::new(),
                    requires_restart: true,
                    restart_hint: format!(
                        "Gemini CLI 通常需要重启后读取新的 .env / settings.json；当前 Chuchen 有 {active_sessions} 个终端会话，后续可接入精准识别和一键重启。"
                    ),
                    affected_session_count: active_sessions,
                });
            }
            if kind == "claude-code" {
                let config_path = write_claude_payload_to_live_config(
                    &home,
                    config_payload.as_deref().unwrap_or(""),
                    auth_payload.as_deref(),
                )?;
                return Ok(AppliedProviderProfileResult {
                    provider_kind: kind,
                    profile_name: profile,
                    applied_command: String::from("native:claude-config-payload"),
                    stdout: format!("已写入 {config_path}"),
                    stderr: String::new(),
                    requires_restart: true,
                    restart_hint: format!(
                        "Claude Code 通常需要重启后读取新的 settings.json；当前 Chuchen 有 {active_sessions} 个终端会话，后续可接入精准识别和一键重启。"
                    ),
                    affected_session_count: active_sessions,
                });
            }
            if kind == "hermes" {
                let config_path =
                    write_hermes_payload_to_live_config(&home, config_payload.as_deref().unwrap_or(""))?;
                return Ok(AppliedProviderProfileResult {
                    provider_kind: kind,
                    profile_name: profile,
                    applied_command: String::from("native:hermes-config-payload"),
                    stdout: format!("已写入 {config_path}"),
                    stderr: String::new(),
                    requires_restart: true,
                    restart_hint: format!(
                        "Hermes 通常需要重启后重新读取 config.yaml；当前 Chuchen 有 {active_sessions} 个终端会话，后续可接入精准识别和一键重启。"
                    ),
                    affected_session_count: active_sessions,
                });
            }

            return Err(format!(
                "{} 的原生配置写回尚未实现；当前只支持 Codex / Claude / Gemini / Hermes 的完整 payload 写回。",
                kind
            ));
        }

        let (stdout, stderr) = run_provider_switch_command(&command)?;
        Ok(AppliedProviderProfileResult {
            provider_kind: kind,
            profile_name: profile,
            applied_command: command,
            stdout,
            stderr,
            requires_restart: true,
            restart_hint: format!(
                "Provider 切换后相关 CLI 通常需要重启；当前 Chuchen 有 {active_sessions} 个终端会话。"
            ),
            affected_session_count: active_sessions,
        })
    })
    .await
    .map_err(|error| format!("应用 Provider 档案任务失败：{error}"))?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_provider_command() {
        assert_eq!(
            parse_simple_command("cc-switch codex use default").unwrap(),
            vec!["cc-switch", "codex", "use", "default"]
        );
    }

    #[test]
    fn parse_simple_command_rejects_shell_operators() {
        assert!(parse_simple_command("cc-switch codex use default; rm -rf tmp").is_err());
        assert!(parse_simple_command("cc-switch codex use default | more").is_err());
    }

    #[test]
    fn extracts_toml_provider_section() {
        let raw = r#"
model_provider = "right"
model = "gpt-5.4"

[model_providers.right]
name = "Niko"
base_url = "https://example.test/v1"

[model_providers.other]
name = "Other"
"#;

        let section = extract_toml_section(raw, "model_providers.right");
        assert_eq!(extract_toml_string_field(&section, "name").unwrap(), "Niko");
        assert_eq!(
            extract_toml_string_field(&section, "base_url").unwrap(),
            "https://example.test/v1"
        );
    }

    #[test]
    fn parses_env_text_without_exposing_comments() {
        let env = parse_env_text("GEMINI_API_KEY=\"secret\"\n# comment\nMODEL=gemini-2.5-pro\n");
        assert_eq!(env.get("GEMINI_API_KEY").unwrap(), "secret");
        assert_eq!(env.get("MODEL").unwrap(), "gemini-2.5-pro");
        assert!(!env.contains_key("# comment"));
    }

    #[test]
    fn extracts_yaml_model_section_fields() {
        let raw = r#"
model:
  default: "anthropic/claude-opus-4-8"
  provider: "openrouter"
  base_url: "https://openrouter.ai/api/v1"

custom_providers:
  - name: openrouter
    api_key: sk-or-demo
"#;

        let section = extract_yaml_section(raw, "model");
        assert_eq!(
            extract_yaml_string_field(&section, "default").unwrap(),
            "anthropic/claude-opus-4-8"
        );
        assert_eq!(
            extract_yaml_string_field(&section, "provider").unwrap(),
            "openrouter"
        );
        assert_eq!(
            extract_yaml_string_field(&section, "base_url").unwrap(),
            "https://openrouter.ai/api/v1"
        );
    }

    #[test]
    fn writes_codex_model_provider_switch() {
        let temp = std::env::temp_dir().join(format!("ct-codex-switch-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let config_path = codex_dir.join("config.toml");
        std::fs::write(
            &config_path,
            r#"
model_provider = "old"
model = "gpt-5.4"

[model_providers.old]
name = "Old"
base_url = "https://old.example/v1"

[model_providers.new]
name = "New"
base_url = "https://new.example/v1"
"#,
        )
        .unwrap();

        write_codex_profile_to_live_config(&temp, "new").unwrap();
        let updated = std::fs::read_to_string(&config_path).unwrap();
        assert!(updated.contains("model_provider = \"new\""));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn writes_full_codex_payload_snapshot() {
        let temp = std::env::temp_dir().join(format!("ct-codex-payload-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();

        write_codex_payload_to_live_config(
            &temp,
            "model_provider = \"right\"\nmodel = \"gpt-5.4\"\napproval_policy = \"never\"\n[mcp_servers.test]\ncommand = \"npx\"\n",
            Some("{\"OPENAI_API_KEY\":\"demo-key\"}"),
        )
        .unwrap();

        let config = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        let auth = std::fs::read_to_string(codex_dir.join("auth.json")).unwrap();
        assert!(config.contains("model_provider = \"right\""));
        assert!(config.contains("[mcp_servers.test]"));
        assert!(auth.contains("OPENAI_API_KEY"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn writes_gemini_payload_snapshot() {
        let temp = std::env::temp_dir().join(format!("ct-gemini-payload-{}", std::process::id()));
        let gemini_dir = temp.join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();

        write_gemini_payload_to_live_config(
            &temp,
            "{\n  \"security\": {\n    \"auth\": {\n      \"selectedType\": \"gemini-api-key\"\n    }\n  }\n}",
            Some("GOOGLE_GEMINI_BASE_URL=https://example.test\nGEMINI_API_KEY=demo-key"),
        )
        .unwrap();

        let settings = std::fs::read_to_string(gemini_dir.join("settings.json")).unwrap();
        let env = std::fs::read_to_string(gemini_dir.join(".env")).unwrap();
        assert!(settings.contains("\"selectedType\": \"gemini-api-key\""));
        assert!(env.contains("GOOGLE_GEMINI_BASE_URL=https://example.test"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn writes_claude_payload_snapshot() {
        let temp = std::env::temp_dir().join(format!("ct-claude-payload-{}", std::process::id()));
        let claude_dir = temp.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();

        write_claude_payload_to_live_config(
            &temp,
            "{\n  \"env\": {\n    \"ANTHROPIC_BASE_URL\": \"https://example.test/anthropic\",\n    \"ANTHROPIC_MODEL\": \"claude-sonnet-4\"\n  }\n}",
            Some("{\"mcpServers\":{\"demo\":{\"command\":\"npx\"}}}"),
        )
        .unwrap();

        let settings = std::fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        let global = std::fs::read_to_string(temp.join(".claude.json")).unwrap();
        assert!(settings.contains("ANTHROPIC_BASE_URL"));
        assert!(global.contains("mcpServers"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn writes_hermes_payload_snapshot() {
        let temp = std::env::temp_dir().join(format!("ct-hermes-payload-{}", std::process::id()));
        let hermes_home = temp.join(".hermes");
        let old_hermes_home = std::env::var_os("HERMES_HOME");
        std::env::set_var("HERMES_HOME", &hermes_home);

        write_hermes_payload_to_live_config(
            &temp,
            "model:\n  default: \"anthropic/claude-opus-4-8\"\n  provider: \"openrouter\"\n",
        )
        .unwrap();

        let config = std::fs::read_to_string(hermes_home.join("config.yaml")).unwrap();
        assert!(config.contains("provider: \"openrouter\""));

        match old_hermes_home {
            Some(value) => std::env::set_var("HERMES_HOME", value),
            None => std::env::remove_var("HERMES_HOME"),
        }
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn merges_equivalent_cc_switch_and_native_profiles() {
        let temp = std::env::temp_dir().join(format!("ct-provider-merge-{}", std::process::id()));
        let db_path = temp.join("cc-switch.db");
        let live_path = temp.join("settings.json");
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(&db_path, "db").unwrap();
        std::fs::write(&live_path, "{}").unwrap();

        let mut stored = build_detected_provider(
            "claude-code",
            String::from("Stored Name"),
            String::from("provider-1"),
            db_path,
            "cc-switch",
            String::from("cc-switch"),
            String::from("claude-sonnet-4"),
            true,
            String::from("stored note"),
            "cc-switch-db",
        );
        stored.identity_key = String::from("claude-code:ccs:provider-1");
        stored.config_payload = Some(String::from(
            r#"{"env":{"ANTHROPIC_BASE_URL":"https://example.test","ANTHROPIC_MODEL":"claude-sonnet-4"}}"#,
        ));

        let mut live = build_detected_provider(
            "claude-code",
            String::from("Claude Live"),
            String::from("local-settings"),
            live_path.clone(),
            "cli-config",
            String::from("native"),
            String::from("claude-sonnet-4"),
            true,
            String::from("live note"),
            "claude-settings",
        );
        live.config_payload = Some(String::from(
            "{\n  \"env\": {\n    \"ANTHROPIC_MODEL\": \"claude-sonnet-4\",\n    \"ANTHROPIC_BASE_URL\": \"https://example.test\"\n  }\n}",
        ));

        let merged = merge_detected_provider_profiles(vec![stored], vec![live]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].identity_key, "claude-code:ccs:provider-1");
        assert_eq!(merged[0].name, "Stored Name");
        assert_eq!(merged[0].config_path, display_home_path(&live_path));
        assert!(merged[0].is_active);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn native_live_wins_unique_active_resolution() {
        let temp = std::env::temp_dir().join(format!("ct-provider-active-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let db_path = temp.join("cc-switch.db");
        let live_path = temp.join("config.toml");
        std::fs::write(&db_path, "db").unwrap();
        std::fs::write(&live_path, "model = \"gpt-5.4\"").unwrap();

        let mut stored = build_detected_provider(
            "codex",
            String::from("Stale Current"),
            String::from("stale"),
            db_path,
            "cc-switch",
            String::from("cc-switch"),
            String::from("gpt-5.4"),
            true,
            String::new(),
            "cc-switch-db",
        );
        stored.identity_key = String::from("codex:ccs:stale");

        let live = build_detected_provider(
            "codex",
            String::from("Actual Live"),
            String::from("actual"),
            live_path,
            "cli-config",
            String::from("native"),
            String::from("gpt-5.4"),
            true,
            String::new(),
            "codex-config",
        );

        let merged = merge_detected_provider_profiles(vec![stored], vec![live]);
        assert_eq!(merged.iter().filter(|profile| profile.is_active).count(), 1);
        assert_eq!(
            merged
                .iter()
                .find(|profile| profile.is_active)
                .map(|profile| profile.name.as_str()),
            Some("Actual Live")
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn cc_switch_hermes_kind_stays_hermes() {
        assert_eq!(kind_from_cc_switch_app("hermes"), Some("hermes"));
    }

    #[test]
    fn source_specific_provider_scans_do_not_mix_native_and_cc_switch() {
        let temp = std::env::temp_dir().join(format!("ct-provider-sources-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        let cc_switch_dir = temp.join(".cc-switch");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::create_dir_all(&cc_switch_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            "model_provider = \"live\"\nmodel = \"gpt-5.4\"\n[model_providers.live]\nname = \"Live\"\nbase_url = \"https://live.example/v1\"\n",
        )
        .unwrap();
        std::fs::write(
            cc_switch_dir.join("config.json"),
            r#"{"codex":{"current":"relay","providers":{"relay":{"name":"Relay","model":"gpt-5.4"}}}}"#,
        )
        .unwrap();

        let native = collect_native_provider_profiles(&temp);
        let cc_switch = collect_cc_switch_provider_profiles(&temp);

        assert!(native
            .iter()
            .all(|profile| profile.managed_by != "cc-switch"));
        assert!(native
            .iter()
            .any(|profile| profile.identity_key.starts_with("codex:native:")));
        assert!(native.iter().all(|profile| profile.usage_stats.is_none()));
        assert_eq!(
            native
                .iter()
                .filter(|profile| profile.provider_kind == "codex" && profile.is_active)
                .count(),
            1
        );
        assert_eq!(cc_switch.len(), 1);
        assert_eq!(cc_switch[0].managed_by, "cc-switch");
        assert_eq!(cc_switch[0].identity_key, "codex:ccs:relay");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn reconciliation_preview_removes_only_confirmed_synced_duplicates_and_stale_entries() {
        let temp = std::env::temp_dir().join(format!("ct-provider-preview-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let config_path = temp.join("config.toml");
        std::fs::write(&config_path, "model = \"gpt-5.4\"").unwrap();

        let mut canonical = build_detected_provider(
            "codex",
            String::from("Relay"),
            String::from("relay"),
            config_path.clone(),
            "cc-switch",
            String::new(),
            String::from("gpt-5.4"),
            true,
            String::new(),
            "cc-switch-db+native-live",
        );
        canonical.identity_key = String::from("codex:ccs:relay");
        canonical.config_payload = Some(String::from(
            "model_provider = \"relay\"\nmodel = \"gpt-5.4\"\n",
        ));
        let mut gemini = build_detected_provider(
            "gemini-cli",
            String::from("Gemini Live"),
            String::from("live"),
            config_path,
            "cli-config",
            String::new(),
            String::from("gemini-2.5-pro"),
            true,
            String::new(),
            "gemini-settings",
        );
        refresh_native_identity_key(&mut gemini);

        let existing = vec![
            ProviderReconciliationInput {
                id: String::from("keep-exact"),
                identity_key: Some(String::from("codex:ccs:relay")),
                provider_kind: String::from("codex"),
                name: String::from("Relay"),
                profile_name: String::from("relay"),
                managed_by: String::from("cc-switch"),
                default_model: String::from("gpt-5.4"),
                request_base_url: None,
                config_payload: canonical.config_payload.clone(),
                auth_payload: None,
                updated_at: String::from("2026-07-01T00:00:00Z"),
            },
            ProviderReconciliationInput {
                id: String::from("remove-duplicate"),
                identity_key: Some(String::from("codex:ccs:old-relay")),
                provider_kind: String::from("codex"),
                name: String::from("Relay Copy"),
                profile_name: String::from("relay"),
                managed_by: String::from("cc-switch"),
                default_model: String::from("gpt-5.4"),
                request_base_url: None,
                config_payload: canonical.config_payload.clone(),
                auth_payload: None,
                updated_at: String::from("2026-07-02T00:00:00Z"),
            },
            ProviderReconciliationInput {
                id: String::from("remove-stale"),
                identity_key: Some(String::from("codex:ccs:deleted")),
                provider_kind: String::from("codex"),
                name: String::from("Deleted"),
                profile_name: String::from("deleted"),
                managed_by: String::from("cc-switch"),
                default_model: String::from("old-model"),
                request_base_url: None,
                config_payload: None,
                auth_payload: None,
                updated_at: String::new(),
            },
            ProviderReconciliationInput {
                id: String::from("preserve-manual"),
                identity_key: None,
                provider_kind: String::from("custom-cli"),
                name: String::from("Manual"),
                profile_name: String::from("manual"),
                managed_by: String::from("manual"),
                default_model: String::new(),
                request_base_url: None,
                config_payload: None,
                auth_payload: None,
                updated_at: String::new(),
            },
            ProviderReconciliationInput {
                id: String::from("review-legacy"),
                identity_key: None,
                provider_kind: String::from("codex"),
                name: String::from("Unknown Legacy"),
                profile_name: String::from("unknown"),
                managed_by: String::from("cli-config"),
                default_model: String::from("unknown"),
                request_base_url: None,
                config_payload: None,
                auth_payload: None,
                updated_at: String::new(),
            },
        ];

        let preview = build_provider_reconciliation_preview(vec![canonical, gemini], existing);
        assert_eq!(preview.summary.existing_count, 5);
        assert_eq!(preview.summary.canonical_count, 2);
        assert_eq!(preview.summary.add_count, 1);
        assert_eq!(preview.summary.update_count, 1);
        assert_eq!(preview.summary.remove_duplicate_count, 1);
        assert_eq!(preview.summary.remove_stale_count, 1);
        assert_eq!(preview.summary.preserve_count, 1);
        assert_eq!(preview.summary.review_count, 1);
        assert_eq!(
            preview
                .actions
                .iter()
                .find(|action| action.existing_id == "keep-exact")
                .map(|action| action.action.as_str()),
            Some("update")
        );
        assert_eq!(
            preview
                .actions
                .iter()
                .find(|action| action.existing_id == "review-legacy")
                .map(|action| action.action.as_str()),
            Some("review")
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn writes_codex_mcp_servers_into_top_level_table() {
        let temp = std::env::temp_dir().join(format!("ct-codex-mcp-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(codex_dir.join("config.toml"), "model = \"gpt-5.4\"\n").unwrap();

        write_codex_mcp_servers(
            &temp,
            &[ManagedMcpServer {
                id: String::from("echo"),
                name: String::from("echo"),
                app_targets: vec![String::from("codex")],
                transport_type: String::from("stdio"),
                config: serde_json::json!({
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@upstash/context7-mcp"]
                }),
                source_kind: String::from("manual"),
            }],
        )
        .unwrap();

        let config = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        assert!(config.contains("[mcp_servers.echo]"));
        assert!(config.contains("command = \"npx\""));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn normalizes_gemini_http_url_on_read() {
        let temp = std::env::temp_dir().join(format!("ct-gemini-mcp-{}", std::process::id()));
        let gemini_dir = temp.join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        std::fs::write(
            gemini_dir.join("settings.json"),
            "{\n  \"mcpServers\": {\n    \"remote\": {\n      \"httpUrl\": \"https://example.test/mcp\"\n    }\n  }\n}",
        )
        .unwrap();

        let servers = read_gemini_mcp_servers(&temp).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].transport_type, "http");
        assert_eq!(servers[0].config["url"], "https://example.test/mcp");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn reads_and_writes_codex_prompt_profile() {
        let temp = std::env::temp_dir().join(format!("ct-codex-prompt-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(codex_dir.join("AGENTS.md"), "# Old Prompt").unwrap();

        let prompts =
            read_managed_prompt_profiles_for_tools(&temp, &[String::from("codex")]).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].content, "# Old Prompt");

        write_managed_prompt_profile(&temp, "codex", "# New Prompt").unwrap();
        let updated = std::fs::read_to_string(codex_dir.join("AGENTS.md")).unwrap();
        assert_eq!(updated, "# New Prompt");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn queries_managed_usage_with_app_filter() {
        let temp = std::env::temp_dir().join(format!("ct-usage-query-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(
            codex_dir
                .join("sessions")
                .join("2026")
                .join("07")
                .join("07"),
        )
        .unwrap();
        std::fs::write(
            codex_dir
                .join("sessions")
                .join("2026")
                .join("07")
                .join("07")
                .join("demo.jsonl"),
            "{\"type\":\"session_meta\",\"payload\":{\"session_id\":\"s1\"}}\n{\"type\":\"turn_context\",\"payload\":{\"model\":\"gpt-5.4\"}}\n{\"timestamp\":\"2026-07-07T10:10:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"total_token_usage\":{\"input_tokens\":100,\"cached_input_tokens\":20,\"output_tokens\":50},\"model\":\"gpt-5.4\"}}}\n",
        )
        .unwrap();

        let result = build_managed_usage_query_result(
            &temp,
            ManagedUsageQueryFilters {
                app_type: Some(String::from("codex")),
                provider_name: None,
                provider_ids: None,
                model: None,
                data_source: None,
                start_at: None,
                end_at: None,
                bucket: Some(String::from("hour")),
                cursor: None,
                limit: Some(20),
            },
        );

        assert_eq!(result.summary.total_requests, 1);
        assert_eq!(result.request_logs.len(), 1);
        assert_eq!(result.request_logs[0].app_type, "codex");
        assert!(!result.channels.is_empty());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn native_usage_file_discovery_is_not_capped() {
        let temp =
            std::env::temp_dir().join(format!("ct-usage-unlimited-files-{}", std::process::id()));
        let sessions = temp.join(".codex").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        for index in 0..130 {
            std::fs::write(sessions.join(format!("session-{index}.jsonl")), "\n").unwrap();
        }

        assert_eq!(collect_codex_usage_files(&temp).len(), 130);
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn native_usage_snapshot_reuses_and_invalidates_by_file_fingerprint() {
        let temp = std::env::temp_dir().join(format!("ct-usage-snapshot-{}", std::process::id()));
        let sessions = temp.join(".codex").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let session = sessions.join("session.jsonl");
        std::fs::write(&session, "\n").unwrap();
        let (size, modified_nanos) = native_usage_file_marker(&session).unwrap();
        let cached_log = managed_usage_log(
            String::from("cached"),
            String::from("codex:native:test"),
            String::from("Test"),
            String::from("codex"),
            String::from("gpt-5.4"),
            1,
            1,
            0,
            0,
            None,
            None,
            200,
            String::from("codex_session"),
            String::from("2026-07-16T00:00:00Z"),
        );
        write_native_usage_snapshot(
            &native_usage_snapshot_path(&temp),
            &NativeUsageSnapshot {
                files: BTreeMap::from([(
                    session.to_string_lossy().to_string(),
                    NativeUsageFileSnapshot {
                        size,
                        modified_nanos,
                        offset: size,
                        offset_probe: native_usage_offset_probe(&session, size),
                        parser_state: None,
                        logs: vec![cached_log],
                    },
                )]),
            },
        );
        assert!(collect_native_usage_logs(&temp)
            .iter()
            .any(|log| log.id == "cached"));

        std::fs::write(&session, "\n\n").unwrap();
        assert!(collect_native_usage_logs(&temp)
            .iter()
            .all(|log| log.id != "cached"));
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn jsonl_offset_waits_for_partial_line_completion() {
        let temp =
            std::env::temp_dir().join(format!("ct-jsonl-partial-offset-{}", std::process::id()));
        std::fs::write(&temp, "{\"id\":1}\n{\"id\"").unwrap();
        let mut ids = Vec::new();
        let offset = read_jsonl_values_from_offset(&temp, 0, |value| {
            ids.push(value["id"].as_u64().unwrap())
        })
        .unwrap();
        assert_eq!(ids, vec![1]);
        assert!(offset < std::fs::metadata(&temp).unwrap().len());

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&temp)
            .unwrap();
        file.write_all(b":2}\n").unwrap();
        drop(file);
        let end = read_jsonl_values_from_offset(&temp, offset, |value| {
            ids.push(value["id"].as_u64().unwrap())
        })
        .unwrap();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(end, std::fs::metadata(&temp).unwrap().len());
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn codex_native_usage_tail_reads_append_and_resets_after_truncate() {
        let temp =
            std::env::temp_dir().join(format!("ct-codex-usage-offset-{}", std::process::id()));
        let sessions = temp.join(".codex").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let session = sessions.join("session.jsonl");
        let first = concat!(
            "{\"timestamp\":\"2026-07-16T00:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-1\",\"model_provider\":\"openai\"}}\n",
            "{\"timestamp\":\"2026-07-16T00:00:01Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"model\":\"gpt-5.4\"}}\n",
            "{\"timestamp\":\"2026-07-16T00:00:02Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"total_token_usage\":{\"input_tokens\":100,\"cached_input_tokens\":20,\"output_tokens\":10}}}}\n"
        );
        std::fs::write(&session, first).unwrap();

        let (logs, stats) = collect_native_usage_logs_with_stats(&temp);
        let logs = logs
            .into_iter()
            .filter(|log| log.app_type == "codex")
            .collect::<Vec<_>>();
        assert_eq!(stats.reset_files, 1);
        assert_eq!(logs.len(), 1);

        let second = "{\"timestamp\":\"2026-07-16T00:00:03Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"total_token_usage\":{\"input_tokens\":160,\"cached_input_tokens\":30,\"output_tokens\":25}}}}\n";
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&session)
            .unwrap();
        file.write_all(second.as_bytes()).unwrap();
        drop(file);

        let (logs, stats) = collect_native_usage_logs_with_stats(&temp);
        let logs = logs
            .into_iter()
            .filter(|log| log.app_type == "codex")
            .collect::<Vec<_>>();
        assert_eq!(stats.tail_parsed_files, 1);
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().any(|log| {
            log.input_tokens == 60 && log.cache_read_tokens == 10 && log.output_tokens == 15
        }));

        let (_, stats) = collect_native_usage_logs_with_stats(&temp);
        assert_eq!(stats.reused_files, 1);
        assert_eq!(stats.tail_parsed_files + stats.reset_files, 0);

        let truncated = concat!(
            "{\"timestamp\":\"2026-07-16T01:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-2\",\"model_provider\":\"openai\"}}\n",
            "{\"timestamp\":\"2026-07-16T01:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"last_token_usage\":{\"input_tokens\":7,\"cached_input_tokens\":2,\"output_tokens\":3},\"model\":\"gpt-5.4\"}}}\n"
        );
        std::fs::write(&session, truncated).unwrap();
        let (logs, stats) = collect_native_usage_logs_with_stats(&temp);
        let logs = logs
            .into_iter()
            .filter(|log| log.app_type == "codex")
            .collect::<Vec<_>>();
        assert_eq!(stats.reset_files, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].input_tokens, 7);
        assert!(logs[0].id.contains("session-2"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn claude_native_usage_tail_replaces_partial_message() {
        let temp =
            std::env::temp_dir().join(format!("ct-claude-usage-offset-{}", std::process::id()));
        let projects = temp.join(".claude").join("projects").join("demo");
        std::fs::create_dir_all(&projects).unwrap();
        let session = projects.join("session.jsonl");
        let partial = "{\"type\":\"assistant\",\"sessionId\":\"session-1\",\"timestamp\":\"2026-07-16T00:00:00Z\",\"message\":{\"id\":\"message-1\",\"model\":\"claude-sonnet-4-6\",\"stop_reason\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1,\"cache_read_input_tokens\":2,\"cache_creation_input_tokens\":0}}}\n";
        std::fs::write(&session, partial).unwrap();
        assert_eq!(
            collect_native_usage_logs_with_stats(&temp)
                .0
                .into_iter()
                .filter(|log| log.app_type == "claude")
                .count(),
            1
        );

        let complete = "{\"type\":\"assistant\",\"sessionId\":\"session-1\",\"timestamp\":\"2026-07-16T00:00:01Z\",\"message\":{\"id\":\"message-1\",\"model\":\"claude-sonnet-4-6\",\"stop_reason\":\"end_turn\",\"usage\":{\"input_tokens\":10,\"output_tokens\":3,\"cache_read_input_tokens\":2,\"cache_creation_input_tokens\":0}}}\n";
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&session)
            .unwrap();
        file.write_all(complete.as_bytes()).unwrap();
        drop(file);

        let (logs, stats) = collect_native_usage_logs_with_stats(&temp);
        let logs = logs
            .into_iter()
            .filter(|log| log.app_type == "claude")
            .collect::<Vec<_>>();
        assert_eq!(stats.tail_parsed_files, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].output_tokens, 3);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn gemini_native_usage_reparses_rewritten_json_document() {
        let temp =
            std::env::temp_dir().join(format!("ct-gemini-usage-cursor-{}", std::process::id()));
        let chats = temp.join(".gemini").join("tmp").join("demo").join("chats");
        std::fs::create_dir_all(&chats).unwrap();
        let session = chats.join("session-test.json");
        let document = |messages: &str| {
            format!(
                "{{\"sessionId\":\"gemini-1\",\"messages\":[{messages}],\"lastUpdated\":\"2026-07-16T00:00:10Z\"}}"
            )
        };
        let first = "{\"id\":\"m1\",\"type\":\"gemini\",\"model\":\"gemini-2.5-pro\",\"timestamp\":\"2026-07-16T00:00:00Z\",\"tokens\":{\"input\":5,\"output\":2}}";
        std::fs::write(&session, document(first)).unwrap();
        assert_eq!(
            collect_native_usage_logs_with_stats(&temp)
                .0
                .into_iter()
                .filter(|log| log.app_type == "gemini")
                .count(),
            1
        );

        let second = "{\"id\":\"m2\",\"type\":\"gemini\",\"model\":\"gemini-2.5-pro\",\"timestamp\":\"2026-07-16T00:00:05Z\",\"tokens\":{\"input\":7,\"output\":3}}";
        std::fs::write(&session, document(&format!("{first},{second}"))).unwrap();
        let (logs, stats) = collect_native_usage_logs_with_stats(&temp);
        let logs = logs
            .into_iter()
            .filter(|log| log.app_type == "gemini")
            .collect::<Vec<_>>();
        assert_eq!(stats.reset_files, 1);
        assert_eq!(stats.tail_parsed_files, 0);
        assert_eq!(logs.len(), 2);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn maps_hermes_session_usage_into_managed_usage() {
        let log = hermes_usage_row_to_log(HermesUsageRow {
            id: String::from("hermes:session-1"),
            provider_id: String::from("hermes:openrouter"),
            provider_name: String::from("openrouter"),
            model: String::from("anthropic/claude-opus-4-8"),
            input_tokens: 120,
            output_tokens: 30,
            cache_read_tokens: 40,
            cache_creation_tokens: 10,
            duration_ms: Some(1500),
            status_code: 200,
            created_at: String::from("2026-07-14T10:00:00Z"),
        });

        assert_eq!(log.app_type, "hermes");
        assert_eq!(log.data_source, "hermes_session");
        assert_eq!(log.provider_id, "hermes:openrouter");
        assert_eq!(log.input_tokens, 120);
        assert_eq!(log.cache_creation_tokens, 10);
        assert_eq!(log.duration_ms, Some(1500));
    }

    #[test]
    fn dedups_session_log_against_proxy_log_shape() {
        let session_log = managed_usage_log(
            String::from("session-1"),
            String::from("native:codex"),
            String::from("Codex Native"),
            String::from("codex"),
            String::from("gpt-5.4"),
            100,
            50,
            20,
            0,
            None,
            Some(1200),
            200,
            String::from("codex_session"),
            String::from("2026-07-07T10:10:00Z"),
        );
        let proxy_log = managed_usage_log(
            String::from("proxy-1"),
            String::from("ccs:codex:demo"),
            String::from("Demo Provider"),
            String::from("codex"),
            String::from("gpt-5.4"),
            100,
            50,
            20,
            30,
            Some(400),
            Some(1500),
            200,
            String::from("proxy"),
            String::from("2026-07-07T10:12:00Z"),
        );
        let all_logs = vec![session_log.clone(), proxy_log];

        assert!(!should_keep_usage_log(&session_log, &all_logs));
    }

    #[test]
    fn enriches_codex_usage_provider_from_live_catalog() {
        let temp = std::env::temp_dir().join(format!("ct-usage-provider-{}", std::process::id()));
        let codex_dir = temp.join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            "model_provider = \"right\"\nmodel = \"gpt-5.4\"\n[model_providers.right]\nname = \"Right Provider\"\nbase_url = \"https://example.test/v1\"\n",
        )
        .unwrap();

        let mut log = managed_usage_log(
            String::from("codex-session-1"),
            String::from("right"),
            String::from("right"),
            String::from("codex"),
            String::from("gpt-5.4"),
            100,
            50,
            20,
            0,
            Some(1200),
            Some(2000),
            200,
            String::from("codex_session"),
            String::from("2026-07-07T10:00:00Z"),
        );

        let catalog = current_provider_catalog(&temp);
        enrich_usage_provider_identity_from_catalog(&catalog, &mut log);
        assert!(log.provider_id.starts_with("codex:native:"));
        assert!(log.provider_name.contains("Right Provider"));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn maps_placeholder_and_cc_switch_usage_to_stable_provider_ids() {
        let catalog = vec![
            UsageProviderCatalogEntry {
                app_type: String::from("claude"),
                identity_key: String::from("claude-code:ccs:work"),
                profile_name: String::from("work"),
                name: String::from("Claude Work"),
                request_host: String::from("api.example.test"),
                is_active: true,
            },
            UsageProviderCatalogEntry {
                app_type: String::from("gemini"),
                identity_key: String::from("gemini-cli:native:live"),
                profile_name: String::from("live"),
                name: String::from("Gemini Live"),
                request_host: String::new(),
                is_active: true,
            },
            UsageProviderCatalogEntry {
                app_type: String::from("codex"),
                identity_key: String::from("codex:ccs:provider-1"),
                profile_name: String::from("provider-1"),
                name: String::from("Codex Relay"),
                request_host: String::new(),
                is_active: false,
            },
        ];
        let mut claude = managed_usage_log(
            String::from("claude-1"),
            String::from("_session"),
            String::from("Claude (Session)"),
            String::from("claude"),
            String::from("claude-sonnet-4-6"),
            1,
            1,
            0,
            0,
            None,
            None,
            200,
            String::from("session_log"),
            String::from("2026-07-15T10:00:00Z"),
        );
        let mut gemini = claude.clone();
        gemini.app_type = String::from("gemini");
        gemini.provider_id = String::from("_gemini_session");
        let mut codex = claude.clone();
        codex.app_type = String::from("codex");
        codex.provider_id = String::from("codex:ccs:provider-1");

        enrich_usage_provider_identity_from_catalog(&catalog, &mut claude);
        enrich_usage_provider_identity_from_catalog(&catalog, &mut gemini);
        enrich_usage_provider_identity_from_catalog(&catalog, &mut codex);

        assert_eq!(claude.provider_id, "claude-code:ccs:work");
        assert_eq!(gemini.provider_id, "gemini-cli:native:live");
        assert_eq!(codex.provider_id, "codex:ccs:provider-1");
        assert_eq!(codex.provider_name, "Codex Relay");
    }

    #[test]
    fn managed_usage_filters_before_final_limit_and_returns_rfc3339_update_time() {
        let temp = std::env::temp_dir().join(format!("ct-usage-range-{}", std::process::id()));
        let chats_dir = temp
            .join(".gemini")
            .join("tmp")
            .join("project")
            .join("chats");
        std::fs::create_dir_all(&chats_dir).unwrap();
        let messages = (0..91)
            .map(|index| {
                let timestamp = if index < 90 {
                    format!("2026-07-15T{:02}:{:02}:00Z", index / 60, index % 60)
                } else {
                    String::from("2026-06-01T00:00:00Z")
                };
                serde_json::json!({
                    "id": format!("message-{index}"),
                    "type": "gemini",
                    "model": "gemini-2.5-pro",
                    "timestamp": timestamp,
                    "tokens": {"input": 10, "output": 2, "cached": 1, "thoughts": 0}
                })
            })
            .collect::<Vec<_>>();
        std::fs::write(
            chats_dir.join("session-range.json"),
            serde_json::to_vec(&serde_json::json!({
                "sessionId": "range",
                "lastUpdated": "2026-07-15T02:00:00Z",
                "messages": messages
            }))
            .unwrap(),
        )
        .unwrap();

        let all = build_managed_usage_query_result(
            &temp,
            ManagedUsageQueryFilters {
                app_type: Some(String::from("gemini")),
                limit: Some(200),
                ..Default::default()
            },
        );
        assert_eq!(all.summary.total_requests, 91);
        assert_eq!(all.request_logs.len(), 91);

        let recent = build_managed_usage_query_result(
            &temp,
            ManagedUsageQueryFilters {
                app_type: Some(String::from("gemini")),
                start_at: Some(String::from("2026-07-09T00:00:00+08:00")),
                end_at: Some(String::from("2026-07-16T00:00:00+08:00")),
                limit: Some(200),
                ..Default::default()
            },
        );
        assert_eq!(recent.summary.total_requests, 90);
        assert_eq!(recent.request_logs.len(), 90);
        assert!(chrono::DateTime::parse_from_rfc3339(&recent.updated_at).is_ok());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn cc_switch_usage_combines_unlimited_details_and_daily_rollups() {
        let temp = std::env::temp_dir().join(format!("ct-ccs-export-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let db_path = temp.join("cc-switch.db");
        let db_path_text = db_path.to_string_lossy().to_string();
        let setup_script = r#"
import sqlite3
import sys
conn = sqlite3.connect(sys.argv[1])
conn.execute("CREATE TABLE providers (id TEXT, app_type TEXT, name TEXT, settings_config TEXT, is_current INTEGER)")
conn.execute("CREATE TABLE proxy_request_logs (request_id TEXT, provider_id TEXT, app_type TEXT, model TEXT, pricing_model TEXT, input_tokens INTEGER, output_tokens INTEGER, cache_read_tokens INTEGER, cache_creation_tokens INTEGER, input_cost_usd REAL, output_cost_usd REAL, cache_read_cost_usd REAL, cache_creation_cost_usd REAL, total_cost_usd REAL, latency_ms INTEGER, first_token_ms INTEGER, duration_ms INTEGER, status_code INTEGER, data_source TEXT, created_at INTEGER)")
conn.execute("CREATE TABLE usage_daily_rollups (date TEXT, app_type TEXT, provider_id TEXT, model TEXT, pricing_model TEXT, request_count INTEGER, success_count INTEGER, input_tokens INTEGER, output_tokens INTEGER, cache_read_tokens INTEGER, cache_creation_tokens INTEGER, total_cost_usd REAL, avg_latency_ms INTEGER)")
conn.execute("INSERT INTO providers VALUES ('relay', 'codex', 'Relay', '{}', 1)")
conn.executemany("INSERT INTO proxy_request_logs VALUES (?, 'relay', 'codex', 'gpt-5.4', 'gpt-5.4', 1, 1, 0, 0, 0.1, 0.2, 0, 0, 0.3, 10, 5, 10, 200, 'proxy', 1784080800)", [(str(i),) for i in range(81)])
conn.execute("INSERT INTO usage_daily_rollups VALUES ('2026-07-01', 'codex', 'relay', 'gpt-5.4', 'gpt-5.4', 100, 100, 1000, 500, 200, 0, 12.5, 10)")
conn.commit()
"#;
        let setup_ok = [
            ("python", vec!["-c"]),
            ("py", vec!["-3", "-c"]),
            ("python3", vec!["-c"]),
        ]
        .into_iter()
        .any(|(program, mut args)| {
            args.push(setup_script);
            args.push(&db_path_text);
            Command::new(program)
                .args(args)
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        });
        if !setup_ok {
            let _ = std::fs::remove_dir_all(&temp);
            return;
        }

        let provider_rows =
            export_cc_switch_database_with_python(&db_path, Some(80), false).unwrap();
        assert_eq!(provider_rows.len(), 1);
        assert!(provider_rows[0].usage_stats.is_none());

        let rows = export_cc_switch_database_with_python(&db_path, None, true).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].usage_stats.as_ref().unwrap().request_logs.len(), 81);

        let usage =
            query_cc_switch_usage_db(&db_path, &ManagedUsageQueryFilters::default()).unwrap();
        assert_eq!(usage.logs.len(), 81);
        assert_eq!(usage.rollups.len(), 1);
        assert!(usage.logs.iter().all(|row| row.provider_name == "Relay"));

        let result = build_managed_usage_query_result_from_data(
            usage.logs,
            usage.rollups,
            ManagedUsageQueryFilters::default(),
        );
        assert_eq!(result.summary.total_requests, 181);
        assert_eq!(result.summary.total_input_tokens, 1081);
        assert_eq!(result.request_logs.len(), 81);
        assert_eq!(result.provider_stats[0].total_requests, 181);
        assert_eq!(result.model_stats.len(), 1);
        assert_eq!(result.model_stats[0].model, "gpt-5.4");
        assert_eq!(result.model_stats[0].total_requests, 181);
        assert_eq!(result.model_stats[0].total_input_tokens, 1081);
        assert_eq!(result.model_stats[0].provider_names, vec!["Relay"]);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn managed_usage_provider_filter_and_cursor_keep_full_aggregates() {
        let make_log = |id: &str, provider_id: &str, provider_name: &str| {
            managed_usage_log(
                id.to_string(),
                provider_id.to_string(),
                provider_name.to_string(),
                String::from("codex"),
                String::from("gpt-5.4"),
                100,
                20,
                50,
                0,
                Some(100),
                Some(500),
                200,
                String::from("codex_session"),
                String::from("2026-07-15T10:00:00Z"),
            )
        };
        let logs = vec![
            make_log("e", "codex:native:one", "One"),
            make_log("d", "codex:native:one", "One"),
            make_log("c", "codex:native:one", "One"),
            make_log("b", "codex:ccs:two", "Two"),
            make_log("a", "codex:ccs:two", "Two"),
            make_log("z", "codex:ccs:excluded", "Excluded"),
        ];
        let filters = ManagedUsageQueryFilters {
            provider_ids: Some(vec![
                String::from("codex:native:one"),
                String::from("codex:ccs:two"),
            ]),
            limit: Some(2),
            ..Default::default()
        };

        let first =
            build_managed_usage_query_result_from_data(logs.clone(), Vec::new(), filters.clone());
        assert_eq!(first.summary.total_requests, 5);
        assert_eq!(first.total, 5);
        assert_eq!(first.provider_stats.len(), 2);
        assert_eq!(first.model_stats.len(), 1);
        assert_eq!(first.model_stats[0].total_requests, 5);
        assert_eq!(first.provider_stats[0].total_requests, 3);
        assert_eq!(first.request_logs.len(), 2);
        assert!(first.has_more);
        assert!(first.next_cursor.is_some());

        let second = build_managed_usage_query_result_from_data(
            logs.clone(),
            Vec::new(),
            ManagedUsageQueryFilters {
                cursor: first.next_cursor.clone(),
                ..filters.clone()
            },
        );
        assert_eq!(second.summary.total_requests, 5);
        assert_eq!(second.provider_stats.len(), 2);
        assert_eq!(second.model_stats[0].total_requests, 5);
        assert_eq!(second.request_logs.len(), 2);
        assert!(second.has_more);
        assert!(first
            .request_logs
            .iter()
            .all(|first_log| second.request_logs.iter().all(|log| log.id != first_log.id)));

        let third = build_managed_usage_query_result_from_data(
            logs,
            Vec::new(),
            ManagedUsageQueryFilters {
                cursor: second.next_cursor.clone(),
                ..filters
            },
        );
        assert_eq!(third.summary.total_requests, 5);
        assert_eq!(third.model_stats[0].total_requests, 5);
        assert_eq!(third.request_logs.len(), 1);
        assert!(!third.has_more);
        assert!(third.next_cursor.is_none());
    }

    #[test]
    fn managed_usage_empty_provider_ids_returns_empty_result() {
        let log = managed_usage_log(
            String::from("one"),
            String::from("codex:native:one"),
            String::from("One"),
            String::from("codex"),
            String::from("gpt-5.4"),
            100,
            20,
            0,
            0,
            None,
            None,
            200,
            String::from("codex_session"),
            String::from("2026-07-15T10:00:00Z"),
        );
        let result = build_managed_usage_query_result_from_data(
            vec![log],
            Vec::new(),
            ManagedUsageQueryFilters {
                provider_ids: Some(Vec::new()),
                ..Default::default()
            },
        );

        assert_eq!(result.summary.total_requests, 0);
        assert!(result.provider_stats.is_empty());
        assert!(result.model_stats.is_empty());
        assert!(result.request_logs.is_empty());
        assert_eq!(result.total, 0);
    }

    #[test]
    fn usage_week_and_month_buckets_use_real_period_boundaries() {
        assert_eq!(
            bucket_usage_timestamp("2026-07-15T10:30:00Z", "week"),
            "2026-07-13T00:00:00Z"
        );
        assert_eq!(
            bucket_usage_timestamp("2026-07-20T00:00:00Z", "week"),
            "2026-07-20T00:00:00Z"
        );
        assert_eq!(
            bucket_usage_timestamp("2026-07-15T10:30:00Z", "month"),
            "2026-07-01T00:00:00Z"
        );
        assert_eq!(bucket_usage_timestamp("2026-07-15T10:30:00Z", "all"), "all");
    }

    #[test]
    fn normalizes_model_id_for_pricing() {
        assert_eq!(
            normalize_model_id_for_pricing("openai/gpt-5.5@high"),
            "gpt-5.5-high"
        );
        assert_eq!(
            normalize_model_id_for_pricing("anthropic/claude-sonnet-4-6-20260217"),
            "claude-sonnet-4-6-20260217"
        );
    }

    #[test]
    fn builds_cc_switch_compatible_model_pricing_candidates() {
        let bedrock = model_pricing_candidates("global.anthropic.claude-opus-4-8-v1:0");
        assert!(bedrock
            .iter()
            .any(|candidate| candidate == "claude-opus-4-8"));

        let desktop = model_pricing_candidates("claude-gpt-5.5@HIGH");
        assert!(desktop.iter().any(|candidate| candidate == "gpt-5.5"));

        let dated = model_pricing_candidates("google/gemini-3-pro-preview-20260514");
        assert!(dated
            .iter()
            .any(|candidate| candidate == "gemini-3-pro-preview"));

        let dotted = model_pricing_candidates("anthropic/claude-opus-4.8");
        assert!(dotted
            .iter()
            .any(|candidate| candidate == "claude-opus-4-8"));
    }

    #[test]
    fn model_pricing_lookup_handles_wrapped_model_ids() {
        assert!(find_local_model_pricing("global.anthropic.claude-opus-4-8-v1:0").is_some());
        assert!(find_local_model_pricing("claude-gpt-5.5@HIGH").is_some());
        assert!(find_local_model_pricing("google/gemini-3-pro-preview-20260514").is_some());
        assert!(find_local_model_pricing("anthropic/claude-opus-4.8").is_some());
    }

    #[test]
    fn computes_local_usage_cost_for_known_model() {
        let cost = compute_local_usage_cost("claude", "claude-opus-4-8", 1_000_000, 100_000, 0, 0);
        assert!(cost > 7.4 && cost < 7.6);
    }

    #[test]
    fn persisted_model_pricing_overrides_cost_calculation() {
        let temp = std::env::temp_dir().join(format!("ct-pricing-{}", std::process::id()));
        let entry = ManagedModelPricingEntry {
            model_id: String::from("test-priced-model"),
            display_name: String::from("Test Priced Model"),
            vendor: String::from("other"),
            input_cost_per_million: 7.25,
            output_cost_per_million: 3.0,
            cache_read_cost_per_million: 1.0,
            cache_creation_cost_per_million: 2.0,
            completion_cost_per_million: 0.0,
            image_cost_per_million: 0.0,
            video_cost_per_million: 0.0,
            audio_cost_per_million: 0.0,
            source: String::from("user"),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        if execute_model_pricing_db(&temp, "upsert", &serde_json::to_value(&entry).unwrap())
            .is_err()
        {
            let _ = std::fs::remove_dir_all(&temp);
            return;
        }
        refresh_model_pricing_overrides(&temp).unwrap();

        let cost = compute_local_usage_cost("codex", "test-priced-model", 1_000_000, 0, 0, 0);
        assert!((cost - 7.25).abs() < f64::EPSILON);

        let _ = std::fs::remove_dir_all(&temp);
    }
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
            detect_native_provider_profiles,
            detect_cc_switch_provider_profiles,
            preview_provider_reconciliation,
            read_current_codex_profile,
            read_current_gemini_profile,
            read_current_claude_profile,
            read_current_hermes_profile,
            read_managed_mcp_servers,
            apply_managed_mcp_servers,
            read_managed_prompt_profiles,
            apply_managed_prompt_profile,
            query_managed_usage,
            get_model_pricing,
            update_model_pricing,
            delete_model_pricing,
            apply_provider_profile,
        ])
        .setup(|app| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.set_zoom(1.0);
            }
            if let Some(home) = home_dir() {
                start_usage_file_monitor(app.handle().clone(), home);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
