package me.gulya.wrenflow

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.PixelFormat
import android.os.Build
import android.util.Log
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.widget.FrameLayout
import android.widget.ImageView

/**
 * Floating bubble overlay for push-to-talk activation.
 * Shows a draggable mic button over all apps.
 *
 * Touch and hold to record, release to transcribe.
 */
class FloatingBubble(private val context: Context) {
    companion object {
        private const val TAG = "FloatingBubble"
        private const val BUBBLE_SIZE_DP = 56
    }

    private val windowManager = context.getSystemService(Context.WINDOW_SERVICE) as WindowManager
    private var bubbleView: View? = null
    private var isShowing = false

    var onRecordStart: (() -> Unit)? = null
    var onRecordStop: (() -> Unit)? = null

    @SuppressLint("ClickableViewAccessibility")
    fun show() {
        if (isShowing) return

        val density = context.resources.displayMetrics.density
        val sizePx = (BUBBLE_SIZE_DP * density).toInt()

        val imageView = ImageView(context).apply {
            setImageResource(android.R.drawable.ic_btn_speak_now)
            scaleType = ImageView.ScaleType.CENTER_INSIDE
            setBackgroundResource(android.R.drawable.dialog_holo_dark_frame)
        }

        val container = FrameLayout(context).apply {
            addView(imageView, FrameLayout.LayoutParams(sizePx, sizePx))
        }

        val params = WindowManager.LayoutParams(
            sizePx,
            sizePx,
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O)
                WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
            else
                @Suppress("DEPRECATION")
                WindowManager.LayoutParams.TYPE_PHONE,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = (100 * density).toInt()
            y = (300 * density).toInt()
        }

        // Drag + push-to-talk gesture
        var initialX = 0
        var initialY = 0
        var initialTouchX = 0f
        var initialTouchY = 0f
        var isDragging = false

        container.setOnTouchListener { _, event ->
            when (event.action) {
                MotionEvent.ACTION_DOWN -> {
                    initialX = params.x
                    initialY = params.y
                    initialTouchX = event.rawX
                    initialTouchY = event.rawY
                    isDragging = false

                    // Start recording on press
                    onRecordStart?.invoke()
                    Log.i(TAG, "Touch down — recording started")
                    true
                }
                MotionEvent.ACTION_MOVE -> {
                    val dx = event.rawX - initialTouchX
                    val dy = event.rawY - initialTouchY
                    if (dx * dx + dy * dy > 25 * density * density) {
                        isDragging = true
                    }
                    if (isDragging) {
                        params.x = initialX + dx.toInt()
                        params.y = initialY + dy.toInt()
                        windowManager.updateViewLayout(container, params)
                    }
                    true
                }
                MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                    // Stop recording on release
                    onRecordStop?.invoke()
                    Log.i(TAG, "Touch up — recording stopped")
                    true
                }
                else -> false
            }
        }

        windowManager.addView(container, params)
        bubbleView = container
        isShowing = true
        Log.i(TAG, "Bubble shown")
    }

    fun dismiss() {
        bubbleView?.let {
            windowManager.removeView(it)
            bubbleView = null
        }
        isShowing = false
        Log.i(TAG, "Bubble dismissed")
    }

    fun updateState(recording: Boolean) {
        // TODO: Change bubble appearance based on recording state
        // - Recording: red pulsing
        // - Idle: default mic icon
        // - Transcribing: spinning indicator
    }
}
