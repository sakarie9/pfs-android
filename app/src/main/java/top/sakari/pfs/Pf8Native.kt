package top.sakari.pfs

/**
 * JNI 绑定类，用于调用 Rust 实现的 PF8 归档功能
 */
object Pf8Native {
    init {
        System.loadLibrary("pf8")
    }

    /**
     * 从目录创建 PF8 归档文件
     *
     * @param inputDir 输入目录路径
     * @param outputPath 输出归档文件路径
     * @return 成功返回 true，失败返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun createArchive(inputDir: String, outputPath: String): Boolean

    /**
     * 从目录创建 PF8 归档文件（带模式匹配）
     *
     * @param inputDir 输入目录路径
     * @param outputPath 输出归档文件路径
     * @param patterns 文件模式数组
     * @return 成功返回 true，失败返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun createArchiveWithPatterns(
        inputDir: String,
        outputPath: String,
        patterns: Array<String>
    ): Boolean

    /**
     * 提取 PF8 归档文件
     *
     * @param archivePath 归档文件路径
     * @param outputDir 输出目录路径
     * @return 成功返回 true，失败返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun extractArchive(archivePath: String, outputDir: String): Boolean

    /**
     * 提取 PF8 归档文件（带模式匹配）
     *
     * @param archivePath 归档文件路径
     * @param outputDir 输出目录路径
     * @param patterns 文件模式数组
     * @return 成功返回 true，失败返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun extractArchiveWithPatterns(
        archivePath: String,
        outputDir: String,
        patterns: Array<String>
    ): Boolean

    /**
     * 获取归档文件中的文件列表
     *
     * @param archivePath 归档文件路径
     * @return JSON 格式的文件列表字符串
     * @throws RuntimeException 如果操作失败
     */
    external fun listArchive(archivePath: String): String

    /**
     * 验证 PF8 归档文件
     *
     * @param archivePath 归档文件路径
     * @return 有效返回 true，无效返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun validateArchive(archivePath: String): Boolean

    /**
     * 创建取消令牌
     *
     * @return 取消令牌句柄
     */
    external fun createCancellationToken(): Long

    /**
     * 取消正在进行的操作
     *
     * @param tokenHandle 取消令牌句柄
     */
    external fun cancelOperation(tokenHandle: Long)

    /**
     * 释放取消令牌资源
     *
     * @param tokenHandle 取消令牌句柄
     */
    external fun freeCancellationToken(tokenHandle: Long)

    /**
     * 带回调的解压函数
     *
     * @param archivePath 归档文件路径
     * @param outputDir 输出目录路径
     * @param callback 进度回调
     * @param tokenHandle 取消令牌句柄（0表示不使用）
     * @return 成功返回 true，失败返回 false
     * @throws RuntimeException 如果操作失败
     */
    external fun extractArchiveWithCallback(
        archivePath: String,
        outputDir: String,
        callback: ExtractionProgressCallback,
        tokenHandle: Long = 0
    ): Boolean
}

/**
 * 解压进度回调接口
 */
interface ExtractionProgressCallback {
    /**
     * 进度更新回调
     *
     * @param currentFile 当前处理的文件名
     * @param currentFileIndex 当前文件索引（从0开始）
     * @param totalFiles 总文件数
     * @param currentFileBytes 当前文件已处理字节数
     * @param currentFileTotal 当前文件总字节数
     * @param totalBytesProcessed 总已处理字节数
     * @param totalBytes 总字节数
     */
    fun onProgress(
        currentFile: String,
        currentFileIndex: Int,
        totalFiles: Int,
        currentFileBytes: Long,
        currentFileTotal: Long,
        totalBytesProcessed: Long,
        totalBytes: Long
    )

    /**
     * 文件开始解压回调
     *
     * @param path 文件路径
     * @param fileIndex 文件索引
     * @param totalFiles 总文件数
     */
    fun onFileStart(path: String, fileIndex: Int, totalFiles: Int)

    /**
     * 文件解压完成回调
     *
     * @param path 文件路径
     * @param fileIndex 文件索引
     */
    fun onFileComplete(path: String, fileIndex: Int)
}

/**
 * 取消令牌类，用于管理解压操作的取消
 */
class CancellationToken {
    private val handle: Long = Pf8Native.createCancellationToken()

    /**
     * 取消操作
     */
    fun cancel() {
        Pf8Native.cancelOperation(handle)
    }

    /**
     * 获取令牌句柄
     */
    fun getHandle(): Long = handle

    /**
     * 释放资源
     */
    fun close() {
        Pf8Native.freeCancellationToken(handle)
    }
}
