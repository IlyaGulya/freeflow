package me.gulya.wrenflow

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                WrenflowApp()
            }
        }
    }
}

@Composable
fun WrenflowApp() {
    var statusText by remember { mutableStateOf("Loading...") }

    LaunchedEffect(Unit) {
        try {
            val config = RustBridge.defaultConfig()
            val status = RustBridge.pipelineStatusText("idle")
            statusText = "Rust core loaded!\nStatus: $status\nConfig: ${config.take(100)}..."
        } catch (e: Exception) {
            statusText = "Failed to load Rust core: ${e.message}"
        }
    }

    Surface(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier.padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            Text("Wrenflow", style = MaterialTheme.typography.headlineLarge)
            Spacer(modifier = Modifier.height(16.dp))
            Text(statusText, style = MaterialTheme.typography.bodyMedium)
        }
    }
}
