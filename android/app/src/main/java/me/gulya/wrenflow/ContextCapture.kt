package me.gulya.wrenflow

import android.accessibilityservice.AccessibilityService
import android.util.Log
import android.view.accessibility.AccessibilityNodeInfo

/**
 * Captures context from the focused app via Accessibility tree traversal.
 * Used for LLM post-processing context (like ScreenCaptureKit on macOS).
 */
object ContextCapture {
    private const val TAG = "ContextCapture"

    data class AppContext(
        val appName: String?,
        val packageName: String?,
        val windowTitle: String?,
        val focusedText: String?,
    )

    /**
     * Capture context from the currently active window.
     */
    fun capture(service: AccessibilityService): AppContext {
        val rootNode = service.rootInActiveWindow

        val packageName = rootNode?.packageName?.toString()
        val windowTitle = rootNode?.findWindowTitle()
        val focusedText = rootNode?.findFocus(AccessibilityNodeInfo.FOCUS_INPUT)?.text?.toString()

        // Try to get app label from package manager
        val appName = packageName?.let { pkg ->
            try {
                val appInfo = service.packageManager.getApplicationInfo(pkg, 0)
                service.packageManager.getApplicationLabel(appInfo).toString()
            } catch (_: Exception) {
                null
            }
        }

        rootNode?.recycle()

        Log.i(TAG, "Context: app=$appName, pkg=$packageName, window=$windowTitle")
        return AppContext(
            appName = appName,
            packageName = packageName,
            windowTitle = windowTitle,
            focusedText = focusedText,
        )
    }

    /** Walk the tree to find a window title (contentDescription of root or toolbar) */
    private fun AccessibilityNodeInfo.findWindowTitle(): String? {
        contentDescription?.toString()?.let { return it }
        for (i in 0 until childCount) {
            val child = getChild(i) ?: continue
            if (child.className?.toString()?.contains("Toolbar") == true) {
                child.contentDescription?.toString()?.let {
                    child.recycle()
                    return it
                }
            }
            child.recycle()
        }
        return null
    }
}
