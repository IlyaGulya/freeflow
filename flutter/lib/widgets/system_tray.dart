import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:window_manager/window_manager.dart';

import 'package:wrenflow/providers/pipeline_state_provider.dart';
import 'package:wrenflow/src/bindings/signals/signals.dart';

/// Manages the macOS system tray (menu bar) icon and context menu.
///
/// Listens to pipeline state changes via Riverpod and updates the tray icon
/// and status text accordingly.
class SystemTrayManager {
  SystemTrayManager(this._ref);

  final ProviderContainer _ref;
  final _trayManager = TrayManager.instance;

  String? _idleIconPath;
  String? _recordingIconPath;
  String? _transcribingIconPath;

  /// Initialize the system tray with icon and context menu.
  Future<void> init() async {
    // Extract tray icon assets to temp files so tray_manager can use file paths.
    _idleIconPath = await _extractAsset('assets/tray_icons/tray_idle.png');
    _recordingIconPath =
        await _extractAsset('assets/tray_icons/tray_recording.png');
    _transcribingIconPath =
        await _extractAsset('assets/tray_icons/tray_transcribing.png');

    // Set the initial idle icon.
    if (_idleIconPath != null) {
      await _trayManager.setIcon(_idleIconPath!);
    }

    // Build the initial context menu.
    await _updateContextMenu(const PipelineStateIdle());

    // Listen to pipeline state changes and update tray accordingly.
    _ref.listen<AsyncValue<PipelineState>>(
      pipelineStateProvider,
      (previous, next) {
        final state = next.value;
        if (state != null) {
          _onPipelineStateChanged(state);
        }
      },
    );
  }

  /// Extract a Flutter asset to a temporary file and return its path.
  Future<String?> _extractAsset(String assetPath) async {
    try {
      final data = await rootBundle.load(assetPath);
      final bytes = data.buffer.asUint8List();
      final fileName = assetPath.split('/').last;
      final tempDir = Directory.systemTemp;
      final file = File('${tempDir.path}/wrenflow_$fileName');
      await file.writeAsBytes(bytes);
      return file.path;
    } catch (e) {
      // Asset not found or write failed; icon will be unavailable.
      return null;
    }
  }

  void _onPipelineStateChanged(PipelineState state) {
    _updateIcon(state);
    _updateContextMenu(state);
  }

  Future<void> _updateIcon(PipelineState state) async {
    final String? iconPath;
    if (state is PipelineStateRecording) {
      iconPath = _recordingIconPath;
    } else if (state is PipelineStateTranscribing) {
      iconPath = _transcribingIconPath;
    } else {
      iconPath = _idleIconPath;
    }

    if (iconPath != null) {
      await _trayManager.setIcon(iconPath);
    }
  }

  Future<void> _updateContextMenu(PipelineState state) async {
    final statusText = _statusText(state);

    final menu = Menu(
      items: [
        MenuItem(label: statusText, disabled: true),
        MenuItem.separator(),
        MenuItem(
          label: 'Settings...',
          onClick: (_) => _showSettings(),
        ),
        MenuItem(
          label: 'History',
          onClick: (_) => _showHistory(),
        ),
        MenuItem.separator(),
        MenuItem(
          label: 'Quit Wrenflow',
          onClick: (_) => _quit(),
        ),
      ],
    );

    await _trayManager.setContextMenu(menu);
  }

  String _statusText(PipelineState state) {
    if (state is PipelineStateIdle) return 'Ready';
    if (state is PipelineStateStarting) return 'Starting...';
    if (state is PipelineStateInitializing) return 'Initializing...';
    if (state is PipelineStateRecording) return 'Recording...';
    if (state is PipelineStateTranscribing) return 'Transcribing...';
    if (state is PipelineStatePasting) return 'Pasting...';
    if (state is PipelineStateError) return 'Error';
    return 'Ready';
  }

  void _showSettings() {
    // Show the main window for settings.
    windowManager.show();
    windowManager.focus();
  }

  void _showHistory() {
    // Show the main window for history.
    windowManager.show();
    windowManager.focus();
  }

  Future<void> _quit() async {
    await _trayManager.destroy();
    exit(0);
  }

  /// Clean up tray resources.
  Future<void> dispose() async {
    await _trayManager.destroy();
  }
}
