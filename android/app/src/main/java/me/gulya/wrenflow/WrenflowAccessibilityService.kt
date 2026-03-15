package me.gulya.wrenflow

import android.accessibilityservice.AccessibilityService
import android.view.accessibility.AccessibilityEvent

/**
 * Accessibility service for text field detection and text insertion.
 * Used for floating bubble overlay and ACTION_SET_TEXT.
 */
class WrenflowAccessibilityService : AccessibilityService() {

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // TODO: Detect focused text fields, show/hide floating bubble
    }

    override fun onInterrupt() {
        // Service interrupted
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        // TODO: Initialize floating bubble overlay
    }
}
