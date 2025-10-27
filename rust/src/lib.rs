use jni::JNIEnv;
use jni::objects::{JClass, JString, JObjectArray, JObject, GlobalRef};
use jni::sys::{jstring, jboolean, jlong};
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

/// 从目录创建 PF8 归档文件（带模式匹配）
/// 
/// # 参数
/// * `input_dir` - 输入目录路径
/// * `output_path` - 输出归档文件路径
/// * `patterns` - 文件模式数组（可选）
/// 
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_createArchiveWithPatterns(
    mut env: JNIEnv,
    _class: JClass,
    input_dir: JString,
    output_path: JString,
    patterns: JObjectArray,
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

    // 转换模式数组
    let patterns_vec: Result<Vec<String>, String> = (|| {
        let len = env.get_array_length(&patterns)
            .map_err(|e| format!("Failed to get array length: {}", e))? as usize;
        
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let pattern_obj = env.get_object_array_element(&patterns, i as i32)
                .map_err(|e| format!("Failed to get array element: {}", e))?;
            let pattern_jstr = JString::from(pattern_obj);
            let pattern = jstring_to_string(&mut env, &pattern_jstr)?;
            result.push(pattern);
        }
        Ok(result)
    })();

    let patterns_vec = match patterns_vec {
        Ok(v) => v,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let patterns_refs: Vec<&str> = patterns_vec.iter().map(|s| s.as_str()).collect();

    match pf8::create_from_dir_with_patterns(&input_dir_str, &output_path_str, &patterns_refs) {
        Ok(_) => 1,
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to create archive with patterns: {}", e));
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

/// 提取 PF8 归档文件（带模式匹配）
/// 
/// # 参数
/// * `archive_path` - 归档文件路径
/// * `output_dir` - 输出目录路径
/// * `patterns` - 文件模式数组（可选）
/// 
/// # 返回
/// * 成功返回 true，失败返回 false（并抛出异常）
#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "system" fn Java_top_sakari_pfs_Pf8Native_extractArchiveWithPatterns(
    mut env: JNIEnv,
    _class: JClass,
    archive_path: JString,
    output_dir: JString,
    patterns: JObjectArray,
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

    // 转换模式数组
    let patterns_vec: Result<Vec<String>, String> = (|| {
        let len = env.get_array_length(&patterns)
            .map_err(|e| format!("Failed to get array length: {}", e))? as usize;
        
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let pattern_obj = env.get_object_array_element(&patterns, i as i32)
                .map_err(|e| format!("Failed to get array element: {}", e))?;
            let pattern_jstr = JString::from(pattern_obj);
            let pattern = jstring_to_string(&mut env, &pattern_jstr)?;
            result.push(pattern);
        }
        Ok(result)
    })();

    let patterns_vec = match patterns_vec {
        Ok(v) => v,
        Err(e) => {
            throw_exception(&mut env, &e);
            return 0;
        }
    };

    let patterns_refs: Vec<&str> = patterns_vec.iter().map(|s| s.as_str()).collect();

    match pf8::extract_with_patterns(&archive_path_str, &output_dir_str, &patterns_refs) {
        Ok(_) => 1,
        Err(e) => {
            throw_exception(&mut env, &format!("Failed to extract archive with patterns: {}", e));
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
            let entries: Vec<String> = reader.entries()
                .map(|entry| {
                    let path_str = entry.path().display().to_string();
                    format!(
                        "{{\"name\":\"{}\",\"size\":{}}}",
                        path_str,
                        entry.size()
                    )
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
struct JavaProgressCallback {
    jvm: Arc<jni::JavaVM>,
    callback_obj: GlobalRef,
    cancelled: Arc<Mutex<bool>>,
}

impl JavaProgressCallback {
    fn new(env: &mut JNIEnv, callback_obj: JObject) -> Result<Self, String> {
        let jvm = env.get_java_vm()
            .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
        
        let callback_global = env.new_global_ref(callback_obj)
            .map_err(|e| format!("Failed to create global ref: {}", e))?;
        
        Ok(Self {
            jvm: Arc::new(jvm),
            callback_obj: callback_global,
            cancelled: Arc::new(Mutex::new(false)),
        })
    }
}

impl pf8::ProgressCallback for JavaProgressCallback {
    fn on_progress(&mut self, progress: &pf8::ProgressInfo) -> pf8::Result<()> {
        // 检查是否已取消
        if let Ok(cancelled) = self.cancelled.lock() {
            if *cancelled {
                return Err(pf8::Error::Cancelled);
            }
        }

        // 获取 JNI 环境
        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return Ok(()), // 如果无法附加线程，静默继续
        };

        // 创建字符串对象
        let file_name = env.new_string(&progress.current_file)
            .unwrap_or_else(|_| env.new_string("").unwrap());

        // 调用 Java 回调方法
        let _ = env.call_method(
            &self.callback_obj,
            "onProgress",
            "(Ljava/lang/String;IIJJJJ)V",
            &[
                (&file_name).into(),
                (progress.current_file_index as i32).into(),
                (progress.total_files as i32).into(),
                (progress.current_file_bytes as i64).into(),
                (progress.current_file_total as i64).into(),
                (progress.total_bytes_processed as i64).into(),
                (progress.total_bytes as i64).into(),
            ],
        );

        Ok(())
    }

    fn on_file_start(&mut self, path: &std::path::Path, file_index: usize, total_files: usize) -> pf8::Result<()> {
        // 检查是否已取消
        if let Ok(cancelled) = self.cancelled.lock() {
            if *cancelled {
                return Err(pf8::Error::Cancelled);
            }
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return Ok(()),
        };

        let path_str = path.display().to_string();
        let jstr = env.new_string(&path_str)
            .unwrap_or_else(|_| env.new_string("").unwrap());
        
        let _ = env.call_method(
            &self.callback_obj,
            "onFileStart",
            "(Ljava/lang/String;II)V",
            &[
                (&jstr).into(),
                (file_index as i32).into(),
                (total_files as i32).into(),
            ],
        );

        Ok(())
    }

    fn on_file_complete(&mut self, path: &std::path::Path, file_index: usize) -> pf8::Result<()> {
        // 检查是否已取消
        if let Ok(cancelled) = self.cancelled.lock() {
            if *cancelled {
                return Err(pf8::Error::Cancelled);
            }
        }

        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return Ok(()),
        };

        let path_str = path.display().to_string();
        let jstr = env.new_string(&path_str)
            .unwrap_or_else(|_| env.new_string("").unwrap());
        
        let _ = env.call_method(
            &self.callback_obj,
            "onFileComplete",
            "(Ljava/lang/String;I)V",
            &[
                (&jstr).into(),
                (file_index as i32).into(),
            ],
        );

        Ok(())
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
    let mut java_callback = match JavaProgressCallback::new(&mut env, callback) {
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
            java_callback.cancelled = Arc::clone(token);
        }
    }

    // 打开归档并进行解压
    match pf8::Pf8Reader::open(&archive_path_str) {
        Ok(mut reader) => {
            match reader.extract_all_with_progress(&output_dir_str, &mut java_callback) {
                Ok(_) => 1,
                Err(e) => {
                    let error_msg = match e {
                        pf8::Error::Cancelled => "Operation cancelled",
                        _ => &format!("Failed to extract archive: {}", e),
                    };
                    throw_exception(&mut env, error_msg);
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
