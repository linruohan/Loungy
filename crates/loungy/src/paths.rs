use std::{collections::HashSet, path::PathBuf, sync::OnceLock};

pub struct Paths {
    pub path_env: String,
    pub cache: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
}

pub static NAME: &str = "loungy";

impl Paths {
    pub fn new() -> Self {
        let username = whoami::username();
        #[cfg(target_os = "macos")]
        let user_dir = PathBuf::from("/Users").join(username.clone());
        #[cfg(target_os = "linux")]
        let user_dir = PathBuf::from("/home").join(username.clone());
        #[cfg(target_os = "windows")]
        let user_dir = PathBuf::from("C:\\Users").join(username.clone());
        let user_dir_str = user_dir.to_string_lossy().to_string();
        // 获取各平台的标准路径
        // 构建 PATH 环境变量
        Self {
            #[cfg(target_os = "macos")]
            path_env: format!(
                "/opt/homebrew/bin:/usr/local/bin:/Users/{}/.nix-profile/bin",
                username
            ),
            #[cfg(target_os = "linux")]
            path_env: format!(
                "/opt/homebrew/bin:/usr/local/bin:/home/{}/.nix-profile/bin",
                username
            ),
            #[cfg(target_os = "windows")]
            path_env: format!(
                "C:\\Windows\\System32;C:\\Windows;{}\\.cargo\\bin;{}\\.local\\bin",
                user_dir_str, user_dir_str
            ),
            #[cfg(target_os = "macos")]
            cache: user_dir.clone().join("Library/Caches").join(NAME),
            #[cfg(target_os = "linux")]
            cache: user_dir.clone().join(".cache").join(NAME),
            #[cfg(target_os = "windows")]
            cache: user_dir.clone().join(".cache").join(NAME),
            config: user_dir.clone().join(".config").join(NAME),
            #[cfg(target_os = "macos")]
            data: user_dir
                .clone()
                .join("Library/Application Support")
                .join(NAME),
            #[cfg(target_os = "linux")]
            data: user_dir.clone().join(".local/share").join(NAME),
            #[cfg(target_os = "windows")]
            data: user_dir
                .clone()
                .join("Library/Application Support")
                .join(NAME),
        }
    }

    // 创建目录（如果不存在）
    pub fn create_dirs(&self) -> std::io::Result<()> {
        let dirs = [&self.cache, &self.config, &self.data];
        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }
        Ok(())
    }
}

pub fn paths() -> &'static Paths {
    static PATHS: OnceLock<Paths> = OnceLock::new();
    PATHS.get_or_init(Paths::new)
}

fn get_user_home_dir(username: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows 代码...
        PathBuf::from("C:\\Users").join(username)
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: /Users/用户名
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from("/Users").join(username)
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: /home/用户名
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from("/home").join(username)
        }
    }

    #[cfg(target_os = "android")]
    {
        // Android: /data/data/包名 或 /storage/emulated/0
        PathBuf::from("/data/data").join(username)
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    {
        // 其他 Unix-like 系统
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from("/home").join(username)
        }
    }
}
fn get_standard_paths(
    user_dir: &PathBuf,
    app_name: &str,
) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    #[cfg(target_os = "macos")]
    {
        (
            user_dir.join("Library/Caches").join(app_name),
            user_dir.join(".config").join(app_name),
            user_dir.join("Library/Application Support").join(app_name),
            user_dir.join("Library/Logs").join(app_name),
            std::env::temp_dir().join(app_name),
        )
    }

    #[cfg(target_os = "linux")]
    {
        use std::env;

        // 遵循 XDG 规范
        let cache_home = env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join(".cache"));

        let config_home = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join(".config"));

        let data_home = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join(".local/share"));

        let state_home = env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join(".local/state"));

        (
            cache_home.join(app_name),
            config_home.join(app_name),
            data_home.join(app_name),
            state_home.join(app_name).join("log"),
            env::var("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
                .join(app_name),
        )
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let local_app_data = env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join("AppData").join("Local"));

        let app_data = env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| user_dir.join("AppData").join("Roaming"));

        (
            local_app_data.join(app_name).join("Cache"),
            app_data.join(app_name).join("Config"),
            app_data.join(app_name).join("Data"),
            local_app_data.join(app_name).join("Logs"),
            env::temp_dir().join(app_name),
        )
    }
}

fn build_path_env(username: &str, user_dir: &PathBuf) -> String {
    let mut paths = Vec::new();
    // 使用 HashSet 去重，同时维护顺序
    let mut seen = HashSet::new();
    // 辅助函数：添加路径（去重）
    let mut add_path = |path: String| {
        if seen.insert(path.clone()) {
            paths.push(path);
        }
    };

    // 添加系统路径
    #[cfg(target_os = "macos")]
    {
        paths.extend([
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
            &format!("/Users/{}/.nix-profile/bin", username),
            &format!("/Users/{}/.cargo/bin", username),
            &format!("/Users/{}/.local/bin", username),
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        paths.extend([
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
            &format!("/home/{}/.nix-profile/bin", username),
            &format!("/home/{}/.cargo/bin", username),
            &format!("/home/{}/.local/bin", username),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        // 1. 添加系统路径
        for path in [
            r"C:\Windows\System32",
            r"C:\Windows",
            r"C:\Windows\System32\Wbem",
        ] {
            add_path(path.to_string());
        }

        // 2. 添加用户目录
        add_path(user_dir.to_string_lossy().into_owned());

        // 3. 添加现有环境变量路径
        if let Ok(existing_path) = std::env::var("PATH") {
            // 创建临时的 String 存储，确保生命周期
            for path_part in existing_path.split(";") {
                let trimmed = path_part.trim();
                if !trimmed.is_empty() {
                    add_path(trimmed.to_string());
                }
            }
        }
    }

    paths.join(if cfg!(windows) { ";" } else { ":" })
}
