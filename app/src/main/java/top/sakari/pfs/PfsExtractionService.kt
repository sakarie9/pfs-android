package top.sakari.pfs

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/**
 * 后台服务，用于解压 PFS 文件
 */
class PfsExtractionService : Service() {

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private lateinit var notificationManager: NotificationManager
    private var cancellationToken: CancellationToken? = null

    override fun onCreate() {
        super.onCreate()
        notificationManager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        createNotificationChannel()
    }

    override fun onBind(intent: Intent?) = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // 处理取消操作
        if (intent?.action == ACTION_CANCEL) {
            cancelExtraction()
            stopSelf(startId)
            return START_NOT_STICKY
        }

        val pfsFilePath = intent?.getStringExtra(EXTRA_PFS_FILE_PATH)

        if (pfsFilePath.isNullOrEmpty()) {
            stopSelf(startId)
            return START_NOT_STICKY
        }

        // 启动前台服务（显示初始进度）
        val notification = createProgressNotification(
            fileName = File(pfsFilePath).name,
            progress = 0,
            currentFile = "准备中...",
            canCancel = true
        )
        startForeground(NOTIFICATION_ID, notification)

        serviceScope.launch {
            try {
                extractPfsFile(pfsFilePath)
            } finally {
                stopSelf(startId)
            }
        }

        return START_NOT_STICKY
    }

    private suspend fun extractPfsFile(pfsFilePath: String) {
        try {
            val pfsFile = File(pfsFilePath)
            if (!pfsFile.exists()) {
                showErrorNotification("文件不存在: ${pfsFile.name}")
                return
            }

            // 解压到 PFS 文件的同目录
            val parentDir = pfsFile.parentFile
            if (parentDir == null) {
                showErrorNotification("无法获取文件所在目录")
                return
            }

            val extractDir = File(parentDir, pfsFile.nameWithoutExtension)

            // 创建解压目录
            if (!extractDir.exists() && !extractDir.mkdirs()) {
                showErrorNotification("无法创建解压目录，请检查存储权限")
                return
            }

            android.util.Log.d(TAG, "PFS 文件: ${pfsFile.absolutePath}")
            android.util.Log.d(TAG, "输出目录: ${extractDir.absolutePath}")

            // 创建取消令牌
            cancellationToken = CancellationToken()

            // 创建进度回调（使用新的 ArchiveProgressCallback 接口）
            val callback = object : ArchiveProgressCallback {
                private var lastUpdateTime = 0L

                override fun onStarted(operationType: String) {
                    android.util.Log.d(TAG, "开始操作: $operationType")
                }

                override fun onEntryStarted(entryName: String) {
                    // 条目开始时的处理
                }

                override fun onProgress(
                    currentFile: String,
                    processedBytes: Long,
                    totalBytes: Long,
                    processedFiles: Int,
                    totalFiles: Int
                ) {
                    val now = System.currentTimeMillis()
                    // 限制通知更新频率（每1000ms更新一次）
                    if (now - lastUpdateTime < 1000) return
                    lastUpdateTime = now

                    val progress = if (totalBytes > 0) {
                        (processedBytes * 100.0 / totalBytes).toInt()
                    } else 0

                    val fileName = currentFile.substringAfterLast('/')
                    val sizeText =
                        "${formatBytes(processedBytes)} / ${formatBytes(totalBytes)}"

                    // 更新通知
                    val notification = createProgressNotification(
                        fileName = pfsFile.name,
                        progress = progress,
                        currentFile = "$fileName ($sizeText)",
                        canCancel = true
                    )
                    notificationManager.notify(NOTIFICATION_ID, notification)

//                    android.util.Log.d(TAG, "进度: $progress% - $fileName ($processedFiles/$totalFiles)")
                }

                override fun onEntryFinished(entryName: String) {
                    android.util.Log.d(TAG, "完成解压: $entryName")
                }

                override fun onWarning(message: String) {
                    android.util.Log.w(TAG, "警告: $message")
                }

                override fun onFinished() {
                    android.util.Log.d(TAG, "解压完成")
                }
            }

            // 执行解压（使用 Rust JNI）
            val success = withContext(Dispatchers.IO) {
                try {
                    Pf8Native.extractArchiveWithCallback(
                        archivePath = pfsFile.absolutePath,
                        outputDir = extractDir.absolutePath,
                        callback = callback,
                        tokenHandle = cancellationToken!!.getHandle()
                    )
                    true
                } catch (e: Exception) {
                    android.util.Log.e(TAG, "解压失败", e)
                    false
                }
            }

            if (success) {
                showSuccessNotification(pfsFile.name, extractDir)
            } else {
                showErrorNotification("解压失败: ${pfsFile.name}")
            }

        } catch (e: Exception) {
            android.util.Log.e(TAG, "解压过程出错", e)
            when {
                e.message?.contains("cancelled", ignoreCase = true) == true -> {
                    showCancelledNotification(File(pfsFilePath).name)
                }

                else -> {
                    showErrorNotification("解压出错: ${e.message}")
                }
            }
        } finally {
            // 释放取消令牌资源
            cancellationToken?.close()
            cancellationToken = null
        }
    }

    private fun cancelExtraction() {
        android.util.Log.d(TAG, "取消解压请求")
        cancellationToken?.cancel()
    }

    private fun formatBytes(bytes: Long): String {
        return when {
            bytes < 1024 -> "$bytes B"
            bytes < 1024 * 1024 -> "%.1f KB".format(bytes / 1024.0)
            bytes < 1024 * 1024 * 1024 -> "%.1f MB".format(bytes / (1024.0 * 1024))
            else -> "%.2f GB".format(bytes / (1024.0 * 1024 * 1024))
        }
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "PFS 文件解压",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "显示 PFS 文件解压进度"
        }
        notificationManager.createNotificationChannel(channel)
    }

    private fun createProgressNotification(
        fileName: String,
        progress: Int,
        currentFile: String,
        canCancel: Boolean
    ): android.app.Notification {
        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_sys_download)
            .setContentTitle("正在解压: $fileName")
            .setContentText(currentFile)
            .setProgress(100, progress, false)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)

        if (canCancel) {
            val cancelIntent = Intent(this, PfsExtractionService::class.java).apply {
                action = ACTION_CANCEL
            }
            val cancelPendingIntent = PendingIntent.getService(
                this,
                0,
                cancelIntent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
            )
            builder.addAction(
                android.R.drawable.ic_menu_close_clear_cancel,
                "取消",
                cancelPendingIntent
            )
        }

        return builder.build()
    }

    private fun showSuccessNotification(fileName: String, outputDir: File) {

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("解压完成")
            .setContentText("$fileName → ${outputDir.absolutePath}")
            .setStyle(
                NotificationCompat.BigTextStyle()
                    .bigText("文件已解压到:\n${outputDir.absolutePath}")
            )
            .setSmallIcon(android.R.drawable.stat_sys_download_done)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .build()

        notificationManager.notify(NOTIFICATION_ID + 1, notification)
    }

    private fun showErrorNotification(message: String) {
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("❌ 解压失败")
            .setContentText(message)
            .setSmallIcon(android.R.drawable.stat_notify_error)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .build()

        notificationManager.notify(NOTIFICATION_ID + 2, notification)
    }

    private fun showCancelledNotification(fileName: String) {
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("🚫 解压已取消")
            .setContentText(fileName)
            .setSmallIcon(android.R.drawable.ic_menu_close_clear_cancel)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()

        notificationManager.notify(NOTIFICATION_ID + 3, notification)
    }

    override fun onDestroy() {
        super.onDestroy()
        // 取消前台服务
        stopForeground(STOP_FOREGROUND_REMOVE)
        // 取消所有协程
        cancellationToken?.close()
    }

    companion object {
        private const val TAG = "PfsExtractionService"
        private const val CHANNEL_ID = "pfs_extraction_channel"
        private const val NOTIFICATION_ID = 1001
        const val EXTRA_PFS_FILE_PATH = "extra_pfs_file_path"
        const val ACTION_CANCEL = "top.sakari.pfs.ACTION_CANCEL"

        /**
         * 启动解压服务
         */
        fun startExtraction(context: Context, pfsFilePath: String) {
            val intent = Intent(context, PfsExtractionService::class.java).apply {
                putExtra(EXTRA_PFS_FILE_PATH, pfsFilePath)
            }
            context.startForegroundService(intent)
        }
    }
}

