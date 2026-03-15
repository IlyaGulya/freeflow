package me.gulya.wrenflow

import android.Manifest
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.util.Log
import androidx.core.content.ContextCompat
import java.io.*
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Captures audio from the microphone at 16kHz mono,
 * writes to a WAV file, and pushes samples to Rust core.
 */
class AudioCapture {
    companion object {
        private const val TAG = "AudioCapture"
        private const val SAMPLE_RATE = 16000
        private const val CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_MONO
        private const val AUDIO_FORMAT = AudioFormat.ENCODING_PCM_FLOAT
        private const val BUFFER_SIZE_FACTOR = 4
    }

    private var audioRecord: AudioRecord? = null
    private var isRecording = false
    private var recordingThread: Thread? = null
    private var outputFile: File? = null
    private val samples = mutableListOf<Float>()

    val isActive: Boolean get() = isRecording

    /**
     * Start recording audio to a temp WAV file.
     * Returns the file path where audio will be saved.
     */
    fun startRecording(cacheDir: File): String {
        if (isRecording) return outputFile?.absolutePath ?: ""

        val bufferSize = AudioRecord.getMinBufferSize(
            SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT
        ) * BUFFER_SIZE_FACTOR

        audioRecord = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            SAMPLE_RATE,
            CHANNEL_CONFIG,
            AUDIO_FORMAT,
            bufferSize
        )

        val file = File(cacheDir, "wrenflow_recording_${System.currentTimeMillis()}.wav")
        outputFile = file
        samples.clear()

        audioRecord?.startRecording()
        isRecording = true

        recordingThread = Thread {
            val buffer = FloatArray(bufferSize / 4) // float = 4 bytes
            while (isRecording) {
                val read = audioRecord?.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING) ?: 0
                if (read > 0) {
                    synchronized(samples) {
                        for (i in 0 until read) {
                            samples.add(buffer[i])
                        }
                    }
                }
            }
        }.also { it.start() }

        Log.i(TAG, "Recording started, output: ${file.absolutePath}")
        return file.absolutePath
    }

    /**
     * Stop recording and write collected samples to WAV file.
     * Returns recording duration in milliseconds.
     */
    fun stopRecording(): Double {
        if (!isRecording) return 0.0

        isRecording = false
        recordingThread?.join(2000)
        recordingThread = null

        audioRecord?.stop()
        audioRecord?.release()
        audioRecord = null

        val collectedSamples: FloatArray
        synchronized(samples) {
            collectedSamples = samples.toFloatArray()
            samples.clear()
        }

        val durationMs = collectedSamples.size.toDouble() / SAMPLE_RATE * 1000.0
        Log.i(TAG, "Recording stopped: ${collectedSamples.size} samples, ${durationMs}ms")

        // Write WAV file
        outputFile?.let { file ->
            writeWav(file, collectedSamples, SAMPLE_RATE)
        }

        return durationMs
    }

    /** Get the path to the last recorded file */
    fun getOutputPath(): String? = outputFile?.absolutePath

    /** Clean up the temp recording file */
    fun cleanup() {
        outputFile?.delete()
        outputFile = null
    }

    private fun writeWav(file: File, samples: FloatArray, sampleRate: Int) {
        val bitsPerSample = 16
        val channels = 1
        val byteRate = sampleRate * channels * bitsPerSample / 8
        val blockAlign = channels * bitsPerSample / 8
        val dataSize = samples.size * 2 // int16 = 2 bytes

        DataOutputStream(BufferedOutputStream(FileOutputStream(file))).use { out ->
            // RIFF header
            out.writeBytes("RIFF")
            out.writeIntLE(36 + dataSize)
            out.writeBytes("WAVE")

            // fmt chunk
            out.writeBytes("fmt ")
            out.writeIntLE(16)
            out.writeShortLE(1) // PCM
            out.writeShortLE(channels)
            out.writeIntLE(sampleRate)
            out.writeIntLE(byteRate)
            out.writeShortLE(blockAlign)
            out.writeShortLE(bitsPerSample)

            // data chunk
            out.writeBytes("data")
            out.writeIntLE(dataSize)

            for (sample in samples) {
                val clamped = sample.coerceIn(-1.0f, 1.0f)
                val int16 = (clamped * Short.MAX_VALUE).toInt().toShort()
                out.writeShortLE(int16.toInt())
            }
        }
        Log.i(TAG, "WAV written: ${file.length()} bytes")
    }

    // Little-endian write helpers
    private fun DataOutputStream.writeIntLE(value: Int) {
        write(value and 0xFF)
        write((value shr 8) and 0xFF)
        write((value shr 16) and 0xFF)
        write((value shr 24) and 0xFF)
    }

    private fun DataOutputStream.writeShortLE(value: Int) {
        write(value and 0xFF)
        write((value shr 8) and 0xFF)
    }
}
