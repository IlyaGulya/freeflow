package me.gulya.wrenflow

import android.accessibilityservice.AccessibilityService
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.util.Log
import android.view.accessibility.AccessibilityNodeInfo

/**
 * Inserts transcribed text into the focused text field via Accessibility API.
 * Falls back to clipboard if direct insertion fails.
 */
object TextInserter {
    private const val TAG = "TextInserter"

    /**
     * Insert text into the currently focused text field.
     * Uses ACTION_SET_TEXT if available, falls back to clipboard.
     */
    fun insertText(service: AccessibilityService, text: String): Boolean {
        val focusedNode = service.rootInActiveWindow
            ?.findFocus(AccessibilityNodeInfo.FOCUS_INPUT)

        if (focusedNode != null && focusedNode.isEditable) {
            // Read existing text and append
            val existing = focusedNode.text?.toString() ?: ""
            val newText = existing + text

            val args = Bundle()
            args.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, newText)
            val success = focusedNode.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
            focusedNode.recycle()

            if (success) {
                Log.i(TAG, "Text inserted via ACTION_SET_TEXT")
                return true
            }
        }

        // Fallback: copy to clipboard
        val clipboard = service.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("Wrenflow", text))
        Log.i(TAG, "Text copied to clipboard (fallback)")
        return false
    }
}
