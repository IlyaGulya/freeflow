package me.gulya.wrenflow

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log

/**
 * Foreground service for microphone access.
 * Required on Android 14+ for background audio recording.
 */
class RecordingService : Service() {
    companion object {
        private const val TAG = "RecordingService"
        private const val CHANNEL_ID = "wrenflow_recording"
        private const val NOTIFICATION_ID = 1
    }

    private val audioCapture = AudioCapture()

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "START" -> startRecording()
            "STOP" -> stopRecording()
        }
        return START_NOT_STICKY
    }

    private fun startRecording() {
        val notification = Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("Wrenflow")
            .setContentText("Recording...")
            .setSmallIcon(android.R.drawable.ic_btn_speak_now)
            .setOngoing(true)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        audioCapture.startRecording(cacheDir)
        Log.i(TAG, "Recording started via foreground service")
    }

    private fun stopRecording() {
        val durationMs = audioCapture.stopRecording()
        Log.i(TAG, "Recording stopped: ${durationMs}ms")

        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()

        // TODO: Send result back to activity/accessibility service via broadcast
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Recording",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "Shows while Wrenflow is recording audio"
        }
        getSystemService(NotificationManager::class.java)?.createNotificationChannel(channel)
    }
}
