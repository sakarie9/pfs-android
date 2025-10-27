package top.sakari.pfs

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.Settings
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountBox
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Home
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteScaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewScreenSizes
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import top.sakari.pfs.ui.theme.PFSTheme
import java.io.File

class MainActivity : ComponentActivity() {

    private var pendingUri: Uri? = null
    private var isExternalFileOpen = false

    companion object {
        private const val TAG = "MainActivity"
    }

    // ==================== Activity Result Launchers ====================

    private val filePickerLauncher = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri: Uri? ->
        Log.d(TAG, "=== 文件选择器回调触发 ===")
        Log.d(TAG, "URI: $uri")
        if (uri == null) {
            Log.d(TAG, "用户取消了文件选择")
            Toast.makeText(this, "未选择文件", Toast.LENGTH_SHORT).show()
        } else {
            Log.d(TAG, "选择了文件: $uri")
            Toast.makeText(this, "已选择文件，正在处理...", Toast.LENGTH_SHORT).show()
            handlePfsFile(uri)
        }
    }

    private val notificationPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        Log.d(TAG, "通知权限结果: $isGranted")
        val message = if (isGranted) {
            "✅ 已获得通知权限"
        } else {
            "⚠️ 未授予通知权限，将无法看到进度提示"
        }
        Toast.makeText(this, message, if (isGranted) Toast.LENGTH_SHORT else Toast.LENGTH_LONG)
            .show()
        checkAndRequestStoragePermission()
    }

    private val manageStorageLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            handleStoragePermissionResult(Environment.isExternalStorageManager())
        }
    }

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        handleStoragePermissionResult(isGranted)
    }

    // ==================== Lifecycle Methods ====================

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // 如果是通过打开 PFS 文件启动，不显示界面
        if (intent?.action == Intent.ACTION_VIEW && intent?.data != null) {
            window.setBackgroundDrawableResource(android.R.color.transparent)
            window.decorView.alpha = 0f
            isExternalFileOpen = true
            handleIntent(intent)
            return
        }

        // 正常启动，显示主界面
        setTheme(R.style.Theme_PFS)
        enableEdgeToEdge()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            checkAndRequestPermissions()
        } else {
            checkStoragePermissionAndNotify()
        }

        setContent {
            PFSTheme {
                PFSApp(onSelectFile = { selectPfsFile() })
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        if (intent.action == Intent.ACTION_VIEW && intent.data != null) {
            isExternalFileOpen = true
        }
        handleIntent(intent)
    }

    // ==================== Permission Handling ====================

    // ==================== Permission Handling ====================

    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    private fun checkAndRequestPermissions() {
        if (!hasNotificationPermission()) {
            Log.d(TAG, "请求通知权限")
            notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        } else {
            checkStoragePermissionAndNotify()
        }
    }

    private fun checkStoragePermissionAndNotify() {
        if (!hasStoragePermission()) {
            Toast.makeText(this, "首次使用需要授予存储访问权限", Toast.LENGTH_LONG).show()
        }
    }

    private fun checkAndRequestStoragePermission() {
        if (hasStoragePermission()) {
            Log.d(TAG, "已有存储权限，处理待处理的文件")
            pendingUri?.let { processPfsFile(it) }
            pendingUri = null
            if (isExternalFileOpen) {
                finish()
            }
        } else {
            Log.d(TAG, "请求存储权限")
            requestStoragePermission()
        }
    }

    private fun handleStoragePermissionResult(isGranted: Boolean) {
        if (isGranted) {
            Toast.makeText(this, "✅ 已获得存储权限", Toast.LENGTH_SHORT).show()
            Log.d(TAG, "存储权限已授予，开始处理文件")
            pendingUri?.let { processPfsFile(it) }
        } else {
            Toast.makeText(this, "❌ 未授予权限，无法解压文件", Toast.LENGTH_LONG).show()
        }
        pendingUri = null
        if (isExternalFileOpen) {
            finish()
        }
    }

    private fun requestStoragePermission() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            requestManageExternalStorage()
        } else {
            requestWriteExternalStorage()
        }
    }

    @RequiresApi(Build.VERSION_CODES.R)
    private fun requestManageExternalStorage() {
        try {
            val intent = Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION)
            intent.data = "package:$packageName".toUri()
            manageStorageLauncher.launch(intent)
        } catch (e: Exception) {
            Log.e(TAG, "无法打开权限设置", e)
            Toast.makeText(this, "❌ 无法打开权限设置", Toast.LENGTH_LONG).show()
            finish()
        }
    }

    private fun requestWriteExternalStorage() {
        requestPermissionLauncher.launch(Manifest.permission.WRITE_EXTERNAL_STORAGE)
    }

    private fun hasStoragePermission(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Environment.isExternalStorageManager()
        } else {
            ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            ) == PackageManager.PERMISSION_GRANTED
        }
    }

    private fun hasNotificationPermission(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.POST_NOTIFICATIONS
            ) == PackageManager.PERMISSION_GRANTED
        } else {
            true
        }
    }

    // ==================== File Selection ====================

    // ==================== File Selection ====================

    private fun selectPfsFile() {
        Log.d(TAG, "=== 打开文件选择器 ===")
        try {
            filePickerLauncher.launch(arrayOf("*/*"))
            Log.d(TAG, "文件选择器启动成功")
        } catch (e: Exception) {
            Log.e(TAG, "无法打开文件选择器", e)
            Toast.makeText(this, "❌ 无法打开文件选择器: ${e.message}", Toast.LENGTH_LONG).show()
        }
    }

    // ==================== File Processing ====================

    private fun handleIntent(intent: Intent?) {
        if (intent?.action == Intent.ACTION_VIEW) {
            intent.data?.let { handlePfsFile(it) }
        }
    }

    private fun handlePfsFile(uri: Uri) {
        Log.d(TAG, "handlePfsFile: $uri")
        pendingUri = uri

        // 检查通知权限
        if (!hasNotificationPermission()) {
            Log.d(TAG, "请求通知权限")
            Toast.makeText(this, "⚠️ 需要授予通知权限以显示解压进度", Toast.LENGTH_LONG).show()
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                notificationPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
            }
            return
        }

        // 检查存储权限
        if (!hasStoragePermission()) {
            Log.d(TAG, "请求存储权限")
            Toast.makeText(this, "⚠️ 必须授予完整存储访问权限才能解压文件", Toast.LENGTH_LONG)
                .show()
            requestStoragePermission()
            return
        }

        // 已有所有权限，直接处理
        Log.d(TAG, "所有权限已授予，开始处理文件")
        processPfsFile(uri)
        pendingUri = null
    }

    private fun processPfsFile(uri: Uri) {
        try {
            Log.d(TAG, "processPfsFile URI: $uri, scheme: ${uri.scheme}")

            val realPath = getRealPathFromUri(uri)
            Log.d(TAG, "Real path: $realPath")

            if (realPath == null) {
                showError("无法访问该文件，请使用文件管理器打开 PFS 文件")
                return
            }

            val file = File(realPath)
            if (!file.exists()) {
                showError("文件不存在: ${file.absolutePath}")
                return
            }

            if (file.extension.lowercase() != "pfs") {
                showError("无效的 PFS 文件（扩展名必须为 .pfs）")
                return
            }

            Log.d(TAG, "准备启动解压服务: ${file.absolutePath}")

            PfsExtractionService.startExtraction(this, file.absolutePath)
            Toast.makeText(this, "开始解压: ${file.name}", Toast.LENGTH_SHORT).show()

            if (isExternalFileOpen) {
                finish()
            }
        } catch (e: Exception) {
            Log.e(TAG, "处理文件出错", e)
            showError("处理文件出错: ${e.message}")
        }
    }

    private fun showError(message: String) {
        Toast.makeText(this, message, Toast.LENGTH_LONG).show()
        if (isExternalFileOpen) {
            finish()
        }
    }

    // ==================== URI Path Resolution ====================

    // ==================== URI Path Resolution ====================

    private fun getRealPathFromUri(uri: Uri): String? {
        Log.d(TAG, "getRealPathFromUri: scheme=${uri.scheme}, authority=${uri.authority}")

        return when (uri.scheme) {
            "file" -> {
                uri.path?.also {
                    Log.d(TAG, "文件路径 (file://): $it")
                }
            }

            "content" -> {
                tryGetPathFromContentUri(uri)
            }

            else -> {
                Log.w(TAG, "不支持的 URI scheme: ${uri.scheme}")
                null
            }
        }
    }

    private fun tryGetPathFromContentUri(uri: Uri): String? {
        // 方法1: 尝试查询 _data 列
        try {
            contentResolver.query(uri, arrayOf("_data"), null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val columnIndex = cursor.getColumnIndex("_data")
                    if (columnIndex != -1) {
                        val path = cursor.getString(columnIndex)
                        if (!path.isNullOrEmpty()) {
                            Log.d(TAG, "从 _data 列获取路径: $path")
                            return path
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "无法通过 _data 列获取路径: ${e.message}")
        }

        // 方法2: 处理特定的 content provider
        return when (uri.authority) {
            "com.android.externalstorage.documents" -> getPathFromExternalStorage(uri)
            "com.android.providers.downloads.documents" -> getPathFromDownloads(uri)
            "com.android.providers.media.documents" -> getPathFromMediaProvider(uri)
            else -> {
                Log.w(TAG, "无法获取真实路径")
                null
            }
        }
    }

    private fun getPathFromExternalStorage(uri: Uri): String? {
        val docId = DocumentsContract.getDocumentId(uri)
        Log.d(TAG, "外部存储文档 ID: $docId")
        val split = docId.split(":")
        if (split.size >= 2) {
            val type = split[0]
            val path = split[1]
            if ("primary".equals(type, ignoreCase = true)) {
                val result = "${Environment.getExternalStorageDirectory()}/$path"
                Log.d(TAG, "外部存储路径: $result")
                return result
            }
        }
        return null
    }

    private fun getPathFromDownloads(uri: Uri): String? {
        val docId = DocumentsContract.getDocumentId(uri)
        Log.d(TAG, "下载文档 ID: $docId")
        if (docId.startsWith("raw:")) {
            val result = docId.substring(4)
            Log.d(TAG, "原始路径: $result")
            return result
        }
        return null
    }

    private fun getPathFromMediaProvider(uri: Uri): String? {
        val docId = DocumentsContract.getDocumentId(uri)
        Log.d(TAG, "媒体文档 ID: $docId")
        val split = docId.split(":")
        if (split.size >= 2) {
            val type = split[0]
            val id = split[1]

            val contentUri = when (type) {
                "image" -> MediaStore.Images.Media.EXTERNAL_CONTENT_URI
                "video" -> MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                "audio" -> MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
                else -> return null
            }

            try {
                contentResolver.query(
                    contentUri,
                    arrayOf("_data"),
                    "_id=?",
                    arrayOf(id),
                    null
                )?.use { cursor ->
                    if (cursor.moveToFirst()) {
                        val columnIndex = cursor.getColumnIndex("_data")
                        if (columnIndex != -1) {
                            val result = cursor.getString(columnIndex)
                            Log.d(TAG, "媒体路径: $result")
                            return result
                        }
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "无法查询媒体文件: ${e.message}")
            }
        }
        return null
    }
}

// ==================== UI Composables ====================

@PreviewScreenSizes
@Composable
fun PFSApp(
    onSelectFile: () -> Unit = {}
) {
    var currentDestination by rememberSaveable { mutableStateOf(AppDestinations.HOME) }

    NavigationSuiteScaffold(
        navigationSuiteItems = {
            AppDestinations.entries.forEach {
                item(
                    icon = {
                        Icon(
                            it.icon,
                            contentDescription = it.label
                        )
                    },
                    label = { Text(it.label) },
                    selected = it == currentDestination,
                    onClick = { currentDestination = it }
                )
            }
        }
    ) {
        Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
            when (currentDestination) {
                AppDestinations.HOME -> HomeScreen(
                    onSelectFile = onSelectFile,
                    modifier = Modifier.padding(innerPadding)
                )

                AppDestinations.FAVORITES -> PlaceholderScreen(
                    title = "收藏",
                    modifier = Modifier.padding(innerPadding)
                )

                AppDestinations.PROFILE -> PlaceholderScreen(
                    title = "设置",
                    modifier = Modifier.padding(innerPadding)
                )
            }
        }
    }
}

@Composable
fun HomeScreen(
    onSelectFile: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
        ) {
            Column(
                modifier = Modifier.padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Icon(
                    imageVector = Icons.Default.Home,
                    contentDescription = "选择文件",
                    modifier = Modifier.size(64.dp),
                    tint = MaterialTheme.colorScheme.primary
                )

                Spacer(modifier = Modifier.height(16.dp))

                Text(
                    text = "PFS 文件解压工具",
                    style = MaterialTheme.typography.headlineSmall,
                    color = MaterialTheme.colorScheme.onSurface
                )

                Spacer(modifier = Modifier.height(8.dp))

                Text(
                    text = "选择 PFS 文件并解压到同一目录",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                Spacer(modifier = Modifier.height(24.dp))

                Button(
                    onClick = onSelectFile,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = null,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.size(8.dp))
                    Text("选择 PFS 文件")
                }

                Spacer(modifier = Modifier.height(16.dp))

                Text(
                    text = "提示：也可以直接通过其他文件管理器打开 PFS 文件进行解压",
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun PlaceholderScreen(
    title: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.headlineMedium
        )
    }
}

enum class AppDestinations(
    val label: String,
    val icon: ImageVector,
) {
    HOME("主页", Icons.Default.Home),
    FAVORITES("收藏", Icons.Default.Favorite),
    PROFILE("设置", Icons.Default.AccountBox),
}

@Preview(showBackground = true)
@Composable
fun HomeScreenPreview() {
    PFSTheme {
        HomeScreen(onSelectFile = {})
    }
}

