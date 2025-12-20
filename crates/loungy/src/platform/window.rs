use super::{AppData, ClipboardWatcher};
// pub fn get_application_data(_path: &PathBuf) -> Option<AppData> {
//     None
// }
// pub fn get_application_folders() -> Vec<std::path::PathBuf> {
//     Vec::new()
// }
// pub fn get_application_files() -> Vec<std::path::PathBuf> {
//     Vec::new()
// }
// pub fn get_frontmost_application_data() -> Option<AppData> {
//     None
// }
use crate::components::shared::{Icon, Img};
use crate::paths::paths;
use crate::window::LWindow;
use gpui::{AsyncWindowContext, WindowContext};
use std::time::Duration;
use std::{
    fs,
    path::{Path, PathBuf},
};
use windows::Win32::Foundation::{
    ERROR_NO_DATA, GlobalFree, HANDLE, HGLOBAL, HINSTANCE, LPARAM, LRESULT, POINT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{HBRUSH, UpdateWindow};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, GetClipboardData, IsClipboardFormatAvailable,
    RemoveClipboardFormatListener, SetClipboardViewer,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::GMEM_ZEROINIT;
use windows::Win32::System::Ole::{CF_HDROP, CF_TEXT, CF_UNICODETEXT};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    KEYBD_EVENT_FLAGS, MAPVK_VK_TO_VSC, MapVirtualKeyW,
};
use windows::Win32::UI::Shell::DROPFILES;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetForegroundWindow,
    GetMessageW, HCURSOR, HICON, MSG, PostQuitMessage, RegisterClassW, SW_SHOW, ShowWindow,
    TranslateMessage, WINDOW_EX_STYLE, WNDCLASS_STYLES, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
    },
    UI::WindowsAndMessaging::GetWindowTextW,
};
use windows::core::{Error, HRESULT, PCWSTR, w};

pub fn get_application_data(path: &Path) -> Option<AppData> {
    let cache_dir = paths().cache.join("apps");
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir.clone()).unwrap();
    }

    // 检查文件扩展名
    let extension = match path.extension() {
        Some(ext) => ext.to_str().unwrap().to_lowercase(),
        None => return None,
    };

    let tag = match extension.as_str() {
        "exe" => "Application",
        "lnk" => "Shortcut",
        "url" => "Web Shortcut",
        _ => return None,
    };

    // 从Windows快捷方式或可执行文件获取信息
    let file_name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let id = format!("{:x}", md5::compute(path.to_string_lossy().as_bytes()));
    let icon_path = cache_dir.join(format!("{}.ico", id));

    // TODO: 实际需要从.exe或.lnk文件中提取图标
    // 这里简化为生成默认图标

    Some(AppData {
        id,
        name: file_name,
        icon: Img::default().icon(Icon::DraftingCompass), // 需要设置Windows图标加载逻辑
        icon_path,
        keywords: vec![],
        tag: tag.to_string(),
    })
}
pub fn get_application_folders() -> Vec<PathBuf> {
    use windows::{
        Win32::{
            System::Com::{
                COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
            },
            UI::Shell::{
                FOLDERID_Desktop, FOLDERID_LocalAppData, FOLDERID_ProgramData,
                FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, FOLDERID_RoamingAppData,
                KF_FLAG_DEFAULT, SHGetKnownFolderPath,
            },
        },
        core::PWSTR,
    };
    let mut folders = Vec::new();

    // 初始化COM库
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    // 获取常见应用程序目录
    let known_folders = [
        (FOLDERID_ProgramFiles, vec!["Common Files"]),
        (FOLDERID_ProgramFilesX86, vec!["Common Files"]),
        (FOLDERID_Desktop, vec![]),
        (
            FOLDERID_ProgramData,
            vec!["Microsoft\\Windows\\Start Menu\\Programs"],
        ),
        (FOLDERID_RoamingAppData, vec![]),
        (FOLDERID_LocalAppData, vec![]),
    ];

    unsafe {
        for (folder_id, subfolders) in known_folders.iter() {
            // 获取已知文件夹路径
            let path_ptr: PWSTR = PWSTR::null();
            let result = SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, None);
            if result.is_ok() && !path_ptr.0.is_null() {
                // 转换PWSTR为字符串
                if let Ok(path_str) = path_ptr.to_string() {
                    let path_buf = PathBuf::from(&path_str);
                    folders.push(path_buf.clone());

                    // 添加子目录
                    for subfolder in subfolders {
                        if !subfolder.is_empty() {
                            folders.push(path_buf.join(subfolder));
                        }
                    }
                }

                // 释放SHGetKnownFolderPath分配的内存
                CoTaskMemFree(Some(path_ptr.0 as *mut _));
            }
        }
    }

    // 用户特定的应用程序目录
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let appdata_path = PathBuf::from(appdata);
        folders.extend(vec![
            appdata_path.join("Microsoft\\Windows\\Start Menu\\Programs"),
            appdata_path.join("Microsoft\\Windows\\Start Menu"),
        ]);
    }

    // 系统目录和公共目录
    folders.extend(vec![
        PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"),
        PathBuf::from("C:\\Windows\\System32"),
        PathBuf::from("C:\\Windows"),
        PathBuf::from("C:\\Program Files\\WindowsApps"),
        PathBuf::from("C:\\Program Files (x86)\\WindowsApps"),
    ]);

    // 添加一些可能的安装目录
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        folders.push(PathBuf::from(program_files));
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        folders.push(PathBuf::from(program_files_x86));
    }

    // 去重
    folders.sort();
    folders.dedup();

    unsafe {
        CoUninitialize();
    }

    folders
}
pub fn get_application_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let valid_extensions = ["exe", "lnk", "url", "appref-ms"];

    for folder in get_application_folders() {
        if let Ok(entries) = fs::read_dir(&folder) {
            for entry in entries.flatten() {
                let path = entry.path();

                // 如果是目录，递归搜索
                if path.is_dir() {
                    // 限制递归深度，避免系统目录
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // 跳过已知的大系统目录以提高性能
                    if !dir_name.contains("Windows") && !dir_name.contains("$") {
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.is_file() {
                                    if let Some(ext) = sub_path.extension() {
                                        if valid_extensions.contains(&ext.to_str().unwrap_or("")) {
                                            files.push(sub_path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if valid_extensions.contains(&ext.to_str().unwrap_or("")) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    files
}

pub fn get_frontmost_application_data() -> Option<AppData> {
    unsafe {
        // 获取当前前台窗口
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // 获取窗口标题作为应用程序名
        let mut buffer = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buffer);
        let window_title = String::from_utf16_lossy(&buffer[..len as usize]);

        // 简化实现：使用窗口标题作为应用名
        // 实际需要获取进程的可执行文件路径
        // 如果窗口标题为空，使用默认名称
        let _app_name = if window_title.is_empty() {
            "Unknown Application".to_string()
        } else {
            window_title.clone()
        };

        let id = format!("foreground_{:x}", hwnd.0 as usize);
        let cache_dir = paths().cache.join("apps");
        let icon_path = cache_dir.join(format!("{}.ico", id));

        Some(AppData {
            id,
            name: window_title.clone(),
            icon: Img::default(),
            icon_path,
            keywords: vec![],
            tag: "Foreground".to_string(),
        })
    }
}

// Windows剪贴板操作函数
pub fn paste_text(value: &str, use_unicode: bool) -> Result<(), windows::core::Error> {
    unsafe {
        // 打开剪贴板
        OpenClipboard(None)?;

        // 清空剪贴板
        EmptyClipboard()?;

        if use_unicode {
            // Unicode文本（UTF-16）
            let wide_str: Vec<u16> = value.encode_utf16().collect();
            let size = (wide_str.len() + 1) * std::mem::size_of::<u16>();
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, size)?;

            // 检查分配是否成功
            if h_mem.0.is_null() {
                CloseClipboard()?;
                return Err(windows::core::Error::from_win32());
            }

            let ptr = GlobalLock(h_mem) as *mut u16;
            if ptr.is_null() {
                GlobalFree(Some(h_mem))?;
                CloseClipboard()?;
                return Err(windows::core::Error::from_win32());
            }

            std::ptr::copy_nonoverlapping(wide_str.as_ptr(), ptr, wide_str.len());
            *ptr.add(wide_str.len()) = 0; // Null terminator

            GlobalUnlock(h_mem)?;
            SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(h_mem.0)))?;
        } else {
            // ANSI文本
            let mut ansi_bytes = Vec::new();
            for c in value.chars() {
                if c as u32 <= 0xFF {
                    ansi_bytes.push(c as u8);
                } else {
                    // 替换非ANSI字符
                    ansi_bytes.push(b'?');
                }
            }

            let size = ansi_bytes.len() + 1; // +1 for null terminator
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, size)?;

            let ptr = GlobalLock(h_mem) as *mut u8;
            if ptr.is_null() {
                GlobalFree(Some(h_mem))?;
                CloseClipboard()?; // 先关闭剪贴板
                return Err(windows::core::Error::from_win32());
            };

            std::ptr::copy_nonoverlapping(ansi_bytes.as_ptr(), ptr, ansi_bytes.len());
            *ptr.add(ansi_bytes.len()) = 0; // Null terminator

            GlobalUnlock(h_mem)?;
            SetClipboardData(CF_TEXT.0 as u32, Some(HANDLE(h_mem.0)))?;
        };

        CloseClipboard()?;
        Ok(())
    }
}

pub fn copy_file_to_clipboard(path: &Path) -> Result<(), windows::core::Error> {
    unsafe {
        OpenClipboard(None)?;
        EmptyClipboard()?;

        // 获取完整路径
        let full_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let path_str = full_path.to_string_lossy();
        let mut wide_path: Vec<u16> = path_str.encode_utf16().collect();

        // 确保以双空字符结尾（Windows文件列表要求）
        wide_path.push(0); // 单个路径后的空字符
        wide_path.push(0); // 整个列表结束的双空字符

        // 计算所需内存大小
        let dropfiles_size = std::mem::size_of::<DROPFILES>();
        let path_size = wide_path.len() * std::mem::size_of::<u16>();
        let total_size = dropfiles_size + path_size;

        // 分配全局内存
        let h_mem = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size)?;

        let ptr = GlobalLock(h_mem) as *mut u8;
        if ptr.is_null() {
            GlobalFree(Some(h_mem))?;
            CloseClipboard()?;
            return Err(windows::core::Error::from_win32());
        }

        // 填充 DROPFILES 结构
        let dropfiles_ptr = ptr as *mut DROPFILES;
        (*dropfiles_ptr).pFiles = dropfiles_size as u32; // 文件列表在结构体之后的偏移量
        (*dropfiles_ptr).pt = POINT { x: 0, y: 0 }; // 鼠标位置（可设为0）
        (*dropfiles_ptr).fNC = false.into(); // 非客户区标志
        (*dropfiles_ptr).fWide = true.into(); // 使用Unicode

        // 复制文件路径（Unicode）
        let path_ptr = ptr.add(dropfiles_size) as *mut u16;
        std::ptr::copy_nonoverlapping(wide_path.as_ptr(), path_ptr, wide_path.len());

        GlobalUnlock(h_mem)?;

        // 设置剪贴板数据
        SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(h_mem.0)))?;

        CloseClipboard()?;
        Ok(())
    }
}
pub fn copy_files_to_clipboard(paths: &[&Path]) -> Result<(), windows::core::Error> {
    unsafe {
        OpenClipboard(None)?;
        EmptyClipboard()?;

        // 构建所有文件路径的字符串
        let mut all_paths = Vec::new();

        for path in paths {
            let full_path = match path.canonicalize() {
                Ok(p) => p,
                Err(_) => path.to_path_buf(),
            };

            let path_str = full_path.to_string_lossy();
            let mut wide_path: Vec<u16> = path_str.encode_utf16().collect();
            wide_path.push(0); // 每个路径以空字符分隔

            all_paths.extend(wide_path);
        }

        // 添加双空字符结束整个列表
        all_paths.push(0);
        all_paths.push(0);

        // 计算所需内存大小
        let dropfiles_size = std::mem::size_of::<DROPFILES>();
        let path_size = all_paths.len() * std::mem::size_of::<u16>();
        let total_size = dropfiles_size + path_size;

        // 分配全局内存
        let h_mem = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size)?;

        let ptr = GlobalLock(h_mem) as *mut u8;
        if ptr.is_null() {
            GlobalFree(Some(h_mem))?;
            CloseClipboard()?;
            return Err(windows::core::Error::from_win32());
        }

        // 填充 DROPFILES 结构
        let dropfiles_ptr = ptr as *mut DROPFILES;
        (*dropfiles_ptr).pFiles = dropfiles_size as u32;
        (*dropfiles_ptr).pt = POINT { x: 0, y: 0 };
        (*dropfiles_ptr).fNC = false.into();
        (*dropfiles_ptr).fWide = true.into();

        // 复制所有文件路径
        let path_ptr = ptr.add(dropfiles_size) as *mut u16;
        std::ptr::copy_nonoverlapping(all_paths.as_ptr(), path_ptr, all_paths.len());

        GlobalUnlock(h_mem)?;

        // 设置剪贴板数据
        SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(h_mem.0)))?;

        CloseClipboard()?;
        Ok(())
    }
}
pub fn close_and_paste(value: &str, formatting: bool, cx: &mut WindowContext) {
    LWindow::close(cx);
    let value = value.to_string();
    cx.spawn(move |mut cx| async move {
        LWindow::wait_for_close(&mut cx).await;
        ClipboardWatcher::disabled(&mut cx);

        // Windows实现：根据formatting决定使用ANSI还是Unicode
        let use_unicode = formatting; // 简化：formatting为true时用Unicode
        let _ = paste_text(&value, use_unicode);
    })
    .detach();
}

pub fn close_and_paste_file(path: &Path, cx: &mut WindowContext) {
    LWindow::close(cx);
    let path = path.to_path_buf();
    cx.spawn(move |mut cx| async move {
        LWindow::wait_for_close(&mut cx).await;
        ClipboardWatcher::disabled(&mut cx);

        let _ = copy_file_to_clipboard(&path);
    })
    .detach();
}

// Windows的自动填充实现
pub fn autofill(value: &str, _password: bool, prev: &str) -> Option<String> {
    // Windows实现通常使用UI Automation或SendInput
    // 这里提供简化实现
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
    };

    // 简化：模拟键盘输入
    unsafe {
        let mut inputs = Vec::new();

        // 删除之前的文本（发送Backspace）
        for _ in 0..prev.len() {
            let mut input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: std::mem::zeroed(),
            };
            input.Anonymous.ki = KEYBDINPUT {
                wVk: VIRTUAL_KEY(0x08), // VK_BACK
                wScan: MapVirtualKeyW(0x08, MAPVK_VK_TO_VSC) as u16,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            inputs.push(input);

            // Key up
            let mut input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: std::mem::zeroed(),
            };
            input_up.Anonymous.ki = KEYBDINPUT {
                wVk: VIRTUAL_KEY(0x08),
                wScan: MapVirtualKeyW(0x08, MAPVK_VK_TO_VSC) as u16,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            inputs.push(input_up);
        }

        // 输入新值
        for ch in value.chars() {
            // 简化：只处理ASCII字符
            if ch.is_ascii() {
                let vk = ch.to_ascii_uppercase() as u32;
                let mut input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: std::mem::zeroed(),
                };
                input.Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                };
                inputs.push(input);

                let mut input_up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: std::mem::zeroed(),
                };
                input_up.Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };
                inputs.push(input_up);
            }
        }

        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == inputs.len() as u32 {
            Some(value.to_string())
        } else {
            None
        }
    }
}

pub fn ocr(_path: &Path) {
    // Windows OCR通常使用Windows.Media.Ocr命名空间（UWP）
    // 或者第三方库如Tesseract
    // 这里留空实现
    println!("OCR功能在Windows上需要额外实现");
}

pub async fn clipboard(
    mut on_change: impl FnMut(&mut AsyncWindowContext),
    mut cx: AsyncWindowContext,
) {
    use windows::Win32::{
        Foundation::{GetLastError, HINSTANCE},
        UI::WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, RegisterClassExW, WNDCLASSEXW, WS_OVERLAPPED,
        },
    };

    unsafe {
        // 创建隐藏窗口用于接收剪贴板消息
        let instance = HINSTANCE(std::ptr::null_mut());
        let class_name = windows::core::w!("ClipboardMonitorClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(clipboard_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            eprintln!("Failed to register window class: {}", GetLastError().0);
            return;
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            windows::core::w!("ClipboardMonitor"),
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance),
            None,
        )
        .unwrap();

        if hwnd.0.is_null() {
            eprintln!("Failed to create window: {}", GetLastError().0);
            return;
        }

        // 设置为剪贴板查看器
        let _ = SetClipboardViewer(hwnd);

        // 简化的消息循环（实际需要更完整的实现）
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;

            // 这里应该处理Windows消息，但简化实现中我们依赖计时器
            on_change(&mut cx);
        }

        // 清理
        // let _ = ChangeClipboardChain(hwnd, HWND_MESSAGE);
        // DestroyWindow(hwnd);
    }
}

// 剪贴板窗口过程
#[allow(unreachable_patterns, non_snake_case)]
extern "system" fn clipboard_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            _WM_CREATE => {
                // 注册剪贴板更新通知
                let _ = AddClipboardFormatListener(hwnd);
                LRESULT(0)
            }

            _WM_DESTROY => {
                // 移除剪贴板更新通知
                let _ = RemoveClipboardFormatListener(hwnd);
                PostQuitMessage(0);
                LRESULT(0)
            }

            _WM_CLIPBOARDUPDATE => {
                // 剪贴板内容更新
                handle_clipboard_update(hwnd);
                LRESULT(0)
            }

            // 其他剪贴板相关消息
            _WM_DRAWCLIPBOARD => {
                // 旧的剪贴板链消息
                LRESULT(0)
            }

            _WM_CHANGECBCHAIN => {
                // 剪贴板链变化
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn handle_clipboard_update(_hwnd: HWND) {
    unsafe {
        // 检查剪贴板是否有内容
        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_ok() {
            if let Ok(text) = get_clipboard_text() {
                // 在这里处理剪贴板文本
                // 例如：发送到主程序、记录日志等
                println!("剪贴板文本更新: {}", text);
            }
        }

        // 检查是否有文件
        if IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() {
            if let Ok(files) = get_clipboard_files() {
                println!("剪贴板文件更新: {:?}", files);
            }
        }
    }
}
fn get_clipboard_text() -> Result<String, windows::core::Error> {
    unsafe {
        OpenClipboard(None)?;

        // 优先尝试Unicode格式
        let text = if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_ok() {
            get_clipboard_unicode_text()
        } else if IsClipboardFormatAvailable(CF_TEXT.0 as u32).is_ok() {
            get_clipboard_ansi_text()
        } else {
            CloseClipboard()?;
            return Err(Error::new(
                HRESULT::from_win32(ERROR_NO_DATA.0),
                "No text data in clipboard",
            ));
        };

        CloseClipboard()?;
        text
    }
}

fn get_clipboard_unicode_text() -> Result<String, windows::core::Error> {
    unsafe {
        let h_data = GetClipboardData(CF_UNICODETEXT.0 as u32)?;
        let h_global = HGLOBAL(h_data.0);
        let ptr = GlobalLock(h_global) as *const u16;

        if !ptr.is_null() {
            // 计算长度
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }

            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16_lossy(slice);

            let _ = GlobalUnlock(h_global);
            return Ok(text);
        }

        Err(windows::core::Error::from_win32())
    }
}

fn get_clipboard_ansi_text() -> Result<String, windows::core::Error> {
    unsafe {
        let h_data = GetClipboardData(CF_TEXT.0 as u32)?;
        let h_global = HGLOBAL(h_data.0);
        let ptr = GlobalLock(h_global) as *const u8;

        if !ptr.is_null() {
            // 计算长度
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }

            let slice = std::slice::from_raw_parts(ptr, len);
            // ANSI转换为UTF-8，假设是本地编码（如CP936等）
            // 更好的做法是使用MultiByteToWideChar进行正确的编码转换
            let text = String::from_utf8_lossy(slice).into_owned();

            let _ = GlobalUnlock(h_global);
            return Ok(text);
        }

        Err(windows::core::Error::from_win32())
    }
}

fn get_clipboard_files() -> Result<Vec<String>, windows::core::Error> {
    unsafe {
        OpenClipboard(None)?;

        let h_data = GetClipboardData(CF_HDROP.0 as u32)?;
        if h_data.0.is_null() {
            CloseClipboard()?;
            return Err(windows::core::Error::from_win32());
        }

        let h_mem = HGLOBAL(h_data.0);
        let drop_ptr = GlobalLock(h_mem) as *const DROPFILES;
        if drop_ptr.is_null() {
            CloseClipboard()?;
            return Err(windows::core::Error::from_win32());
        }

        let dropfiles = &*drop_ptr;
        let mut files = Vec::new();

        // 获取第一个文件的指针
        let mut file_ptr = (drop_ptr as *const u8).add(dropfiles.pFiles as usize) as *const u16;

        // 遍历所有文件
        while *file_ptr != 0 {
            // 计算文件名长度
            let mut len = 0;
            while *file_ptr.add(len) != 0 {
                len += 1;
            }

            // 读取文件名
            let slice = std::slice::from_raw_parts(file_ptr, len);
            if let Ok(file_path) = String::from_utf16(slice) {
                files.push(file_path);
            }

            // 移动到下一个文件
            file_ptr = file_ptr.add(len + 1);
        }

        GlobalUnlock(h_mem)?;
        CloseClipboard()?;

        Ok(files)
    }
}

// 创建剪贴板监听窗口的函数
pub fn create_clipboard_listener() -> Result<HWND, windows::core::Error> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("ClipboardListener");

        // 注册窗口类
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(clipboard_wnd_proc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        // 创建消息窗口（不可见）
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Clipboard Listener"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            Some(std::ptr::null_mut()),
        )?;

        if hwnd.0.is_null() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(hwnd)
        }
    }
}

// 消息循环
pub fn run_clipboard_listener() -> Result<(), windows::core::Error> {
    unsafe {
        let hwnd = create_clipboard_listener()?;

        // 显示窗口（可选）
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        // 消息循环
        let mut msg = MSG::default();
        loop {
            // GetMessageW 返回 BOOL，-1 表示错误，0 表示退出，正数表示有消息
            let result = GetMessageW(&mut msg, None, 0, 0);

            if result.0 == -1 {
                // 错误情况
                return Err(Error::from_win32());
            } else if result.0 == 0 {
                // WM_QUIT 消息，正常退出
                break;
            }

            // 处理消息
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
