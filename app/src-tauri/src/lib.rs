use once_cell::sync::Lazy;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
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
        ])
        .setup(|app| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.set_zoom(1.0);
            }
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
