use jni::JNIEnv;
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::sys::{jboolean, jlong, jstring};
use std::sync::{Arc, Mutex};

/// 辅助函数：将 Java 字符串转换为 Rust String
fn jstring_to_string(env: &mut JNIEnv, jstr: &JString) -> Result<String, String> {
    env.get_string(jstr)
        .map(|s| s.into())
        .map_err(|e| format!("Failed to convert JString: {}", e))
}

/// 辅助函数：创建 Java 异常
fn throw_exception(env: &mut JNIEnv, message: &str) {
    let _ = env.throw_new("java/lang/RuntimeException", message);
}

/// 从目录创建 PF8 归档文件
///
/// # 参数
/// * `input_dir` - 输入目录路径
/// * `output_path` - 输出归档文件路径
///
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的JNI参数
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_createArchive(
    mut env: JNIEnv,
    _class: JClass,
    input_dir: JString,
    output_path: JString,
) -> jboolean {
    let input_dir_str = match jstring_to_string(&mut env, &input_dir) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let output_path_str = match jstring_to_string(&mut env, &output_path) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    match pf8::create_from_dir(&input_dir_str, &output_path_str) {
        Ok(_) => 1,
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to create archive: {}", e));
            0
        }
    }
}

/// 提取 PF8 归档文件
///
/// # 参数
/// * `archive_path` - 归档文件路径
/// * `output_dir` - 输出目录路径
///
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的JNI参数
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_extractArchive(
    mut env: JNIEnv,
    _class: JClass,
    archive_path: JString,
    output_dir: JString,
) -> jboolean {
    let archive_path_str = match jstring_to_string(&mut env, &archive_path) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let output_dir_str = match jstring_to_string(&mut env, &output_dir) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    match pf8::extract(&archive_path_str, &output_dir_str) {
        Ok(_) => 1,
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to extract archive: {}", e));
            0
        }
    }
}

/// 获取归档文件中的文件列表（JSON 格式）
///
/// # 参数
/// * `archive_path` - 归档文件路径
///
/// # 返回
/// * 成功返回 JSON 字符串，失败返回 null（并抛出异常）
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的JNI参数
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_listArchive(
    mut env: JNIEnv,
    _class: JClass,
    archive_path: JString,
) -> jstring {
    let archive_path_str = match jstring_to_string(&mut env, &archive_path) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return std::ptr::null_mut();
        }
    };

    // 读取归档文件并获取条目信息
    match pf8::Pf8Reader::open(&archive_path_str) {
        Ok(reader) => {
            let entries: Vec<String> = reader
                .entries()
                .map(|entry| {
                    let path_str = entry.path().display().to_string();
                    format!("{{\"name\":\"{}\",\"size\":{}}}", path_str, entry.size())
                })
                .collect();

            let json = format!("[{}]", entries.join(","));

            match env.new_string(&json) {
                Ok(jstr) => jstr.into_raw(),
                Err(e) => {
                    throw_exception(&mut env, &format!("Failed to create Java string: {}", e));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to read archive: {}", e));
            std::ptr::null_mut()
        }
    }
}

/// 验证 PF8 归档文件格式
///
/// # 参数
/// * `archive_path` - 归档文件路径
///
/// # 返回
/// * 有效返回 true，无效返回 false
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的JNI参数
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_validateArchive(
    mut env: JNIEnv,
    _class: JClass,
    archive_path: JString,
) -> jboolean {
    let archive_path_str = match jstring_to_string(&mut env, &archive_path) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // 简化验证：尝试打开文件
    match pf8::Pf8Reader::open(&archive_path_str) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// Java 回调包装器，用于将进度信息传递回 Java
/// 实现新的 ArchiveHandler trait
struct JavaArchiveHandler {
    jvm: Arc<jni::JavaVM>,
    callback_obj: GlobalRef,
    cancelled: Arc<Mutex<bool>>,
}

impl JavaArchiveHandler {
    fn new(env: &mut JNIEnv, callback_obj: JObject) -> Result<Self, String> {
        let jvm = env
            .get_java_vm()
            .map_err(|e| format!("Failed to get JavaVM: {}", e))?;

        let callback_global = env
            .new_global_ref(callback_obj)
            .map_err(|e| format!("Failed to create global ref: {}", e))?;

        Ok(Self {
            jvm: Arc::new(jvm),
            callback_obj: callback_global,
            cancelled: Arc::new(Mutex::new(false)),
        })
    }
}

impl pf8::ArchiveHandler for JavaArchiveHandler {
    fn on_started(&mut self, op_type: pf8::OperationType) -> pf8::ControlAction {
        // 检查是否已取消
        if matches!(self.cancelled.lock().as_deref(), Ok(true)) {
            return pf8::ControlAction::Abort;
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let op_str = match op_type {
            pf8::OperationType::Pack => "Pack",
            pf8::OperationType::Unpack => "Unpack",
        };

        let jstr = env
            .new_string(op_str)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        let _ = env.call_method(
            &self.callback_obj,
            "onStarted",
            "(Ljava/lang/String;)V",
            &[(&jstr).into()],
        );

        pf8::ControlAction::Continue
    }

    fn on_entry_started(&mut self, name: &str) -> pf8::ControlAction {
        // 检查是否已取消
        if matches!(self.cancelled.lock().as_deref(), Ok(true)) {
            return pf8::ControlAction::Abort;
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let jstr = env
            .new_string(name)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        let _ = env.call_method(
            &self.callback_obj,
            "onEntryStarted",
            "(Ljava/lang/String;)V",
            &[(&jstr).into()],
        );

        pf8::ControlAction::Continue
    }

    fn on_progress(&mut self, info: &pf8::ProgressInfo) -> pf8::ControlAction {
        // 检查是否已取消
        if matches!(self.cancelled.lock().as_deref(), Ok(true)) {
            return pf8::ControlAction::Abort;
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let file_name = env
            .new_string(&info.current_file)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        // 调用 Java 回调方法
        // onProgress(String currentFile, long processedBytes, long totalBytes,
        //            int processedFiles, int totalFiles)
        let total_bytes = info.total_bytes.unwrap_or(0);
        let total_files = info.total_files.unwrap_or(0);

        let _ = env.call_method(
            &self.callback_obj,
            "onProgress",
            "(Ljava/lang/String;JJII)V",
            &[
                (&file_name).into(),
                (info.processed_bytes as i64).into(),
                (total_bytes as i64).into(),
                (info.processed_files as i32).into(),
                (total_files as i32).into(),
            ],
        );

        pf8::ControlAction::Continue
    }

    fn on_entry_finished(&mut self, name: &str) -> pf8::ControlAction {
        // 检查是否已取消
        if matches!(self.cancelled.lock().as_deref(), Ok(true)) {
            return pf8::ControlAction::Abort;
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let jstr = env
            .new_string(name)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        let _ = env.call_method(
            &self.callback_obj,
            "onEntryFinished",
            "(Ljava/lang/String;)V",
            &[(&jstr).into()],
        );

        pf8::ControlAction::Continue
    }

    fn on_warning(&mut self, message: &str) -> pf8::ControlAction {
        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let jstr = env
            .new_string(message)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        let _ = env.call_method(
            &self.callback_obj,
            "onWarning",
            "(Ljava/lang/String;)V",
            &[(&jstr).into()],
        );

        pf8::ControlAction::Continue
    }

    fn on_finished(&mut self) -> pf8::ControlAction {
        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return pf8::ControlAction::Continue,
        };

        let _ = env.call_method(&self.callback_obj, "onFinished", "()V", &[]);

        pf8::ControlAction::Continue
    }
}

/// 创建取消令牌
///
/// # 返回
/// * 返回取消令牌的句柄（指针）
///
/// # Safety
/// 此函数通过JNI从Java调用
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_createCancellationToken(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let token = Box::new(Arc::new(Mutex::new(false)));
    Box::into_raw(token) as jlong
}

/// 取消操作
///
/// # 参数
/// * `token_handle` - 取消令牌句柄
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的token_handle
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_cancelOperation(
    _env: JNIEnv,
    _class: JClass,
    token_handle: jlong,
) {
    if token_handle == 0 {
        return;
    }

    unsafe {
        let token = &*(token_handle as *const Arc<Mutex<bool>>);
        if let Ok(mut cancelled) = token.lock() {
            *cancelled = true;
        }
    }
}

/// 释放取消令牌
///
/// # 参数
/// * `token_handle` - 取消令牌句柄
///
/// # Safety
/// 此函数通过JNI从Java调用，需要有效的token_handle
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_freeCancellationToken(
    _env: JNIEnv,
    _class: JClass,
    token_handle: jlong,
) {
    if token_handle == 0 {
        return;
    }

    unsafe {
        let _ = Box::from_raw(token_handle as *mut Arc<Mutex<bool>>);
    }
}

/// 带回调和取消支持的解压函数
///
/// # 参数
/// * `archive_path` - 归档文件路径
/// * `output_dir` - 输出目录路径
/// * `callback` - 进度回调对象
/// * `token_handle` - 取消令牌句柄（可选，0表示不使用）
///
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
///
/// # Safety
/// 此函数通过JNI从Java调用
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_extractArchiveWithCallback(
    mut env: JNIEnv,
    _class: JClass,
    archive_path: JString,
    output_dir: JString,
    callback: JObject,
    token_handle: jlong,
) -> jboolean {
    let archive_path_str = match jstring_to_string(&mut env, &archive_path) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let output_dir_str = match jstring_to_string(&mut env, &output_dir) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    // 创建 Java 回调包装器
    let mut java_handler = match JavaArchiveHandler::new(&mut env, callback) {
        Ok(cb) => cb,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    // 如果提供了取消令牌，将其连接到回调
    if token_handle != 0 {
        unsafe {
            let token = &*(token_handle as *const Arc<Mutex<bool>>);
            java_handler.cancelled = Arc::clone(token);
        }
    }

    // 打开归档并进行解压
    match pf8::Pf8Reader::open(&archive_path_str) {
        Ok(mut reader) => {
            match reader.extract_all_with_progress(&output_dir_str, &mut java_handler) {
                Ok(_) => 1,
                Err(e) => {
                    let error_msg = match e {
                        pf8::Error::Cancelled => "Operation cancelled".to_string(),
                        _ => format!("Failed to extract archive: {}", e),
                    };
                    throw_exception(&mut env, &error_msg);
                    0
                }
            }
        }
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to open archive: {}", e));
            0
        }
    }
}

/// 带回调和取消支持的创建归档函数
///
/// # 参数
/// * `input_dir` - 输入目录路径
/// * `output_path` - 输出归档文件路径
/// * `callback` - 进度回调对象
/// * `token_handle` - 取消令牌句柄（可选，0表示不使用）
///
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
///
/// # Safety
/// 此函数通过JNI从Java调用
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_createArchiveWithCallback(
    mut env: JNIEnv,
    _class: JClass,
    input_dir: JString,
    output_path: JString,
    callback: JObject,
    token_handle: jlong,
) -> jboolean {
    let input_dir_str = match jstring_to_string(&mut env, &input_dir) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let output_path_str = match jstring_to_string(&mut env, &output_path) {
        Ok(s) => s,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    // 创建 Java 回调包装器
    let mut java_handler = match JavaArchiveHandler::new(&mut env, callback) {
        Ok(cb) => cb,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    // 如果提供了取消令牌，将其连接到回调
    if token_handle != 0 {
        unsafe {
            let token = &*(token_handle as *const Arc<Mutex<bool>>);
            java_handler.cancelled = Arc::clone(token);
        }
    }

    // 创建归档
    match pf8::create_from_dir_with_progress(&input_dir_str, &output_path_str, &mut java_handler) {
        Ok(_) => 1,
        Err(e) => {
            let error_msg = match e {
                pf8::Error::Cancelled => "Operation cancelled".to_string(),
                _ => format!("Failed to create archive: {}", e),
            };
            throw_exception(&mut env, &error_msg);
            0
        }
    }
}
