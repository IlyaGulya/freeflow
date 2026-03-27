import 'dart:async';

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:rinf/rinf.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:wrenflow/providers/settings_provider.dart';
import 'package:wrenflow/src/bindings/signals/signals.dart';

/// Available hotkey options for the push-to-talk trigger.
const _hotkeyOptions = <String, String>{
  'fn': 'Fn',
  'rightOption': 'Right Option',
  'f5': 'F5',
};

/// Settings screen with General and About tabs.
class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  int _selectedTab = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFFF5F5F7),
      body: Column(
        children: [
          _buildTabBar(),
          Expanded(
            child: _selectedTab == 0
                ? const _GeneralTab()
                : const _AboutTab(),
          ),
        ],
      ),
    );
  }

  Widget _buildTabBar() {
    return Container(
      padding: const EdgeInsets.only(top: 12, bottom: 8),
      decoration: const BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Color(0xFFE0E0E0), width: 0.5),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          _buildTab(0, CupertinoIcons.gear, 'General'),
          const SizedBox(width: 24),
          _buildTab(1, CupertinoIcons.info, 'About'),
        ],
      ),
    );
  }

  Widget _buildTab(int index, IconData icon, String label) {
    final isSelected = _selectedTab == index;
    return GestureDetector(
      onTap: () => setState(() => _selectedTab = index),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            icon,
            size: 24,
            color: isSelected
                ? CupertinoColors.activeBlue
                : CupertinoColors.secondaryLabel,
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 11,
              fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
              color: isSelected
                  ? CupertinoColors.activeBlue
                  : CupertinoColors.secondaryLabel,
            ),
          ),
        ],
      ),
    );
  }
}

/// General settings tab with all configurable options.
class _GeneralTab extends ConsumerStatefulWidget {
  const _GeneralTab();

  @override
  ConsumerState<_GeneralTab> createState() => _GeneralTabState();
}

class _GeneralTabState extends ConsumerState<_GeneralTab> {
  late TextEditingController _vocabularyController;
  Timer? _vocabularyDebounce;
  List<AudioDeviceInfo> _audioDevices = [];
  StreamSubscription<RustSignalPack<AudioDevicesListed>>? _deviceSubscription;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(settingsProvider);
    _vocabularyController =
        TextEditingController(text: settings.customVocabulary);

    // Listen for audio device list responses from Rust.
    _deviceSubscription =
        AudioDevicesListed.rustSignalStream.listen((signal) {
      if (mounted) {
        setState(() {
          _audioDevices = signal.message.devices;
        });
      }
    });

    // Request the device list from Rust.
    const ListAudioDevices().sendSignalToRust();
  }

  @override
  void dispose() {
    _vocabularyController.dispose();
    _vocabularyDebounce?.cancel();
    _deviceSubscription?.cancel();
    super.dispose();
  }

  void _onVocabularyChanged(String value) {
    _vocabularyDebounce?.cancel();
    _vocabularyDebounce = Timer(const Duration(milliseconds: 500), () {
      ref.read(settingsProvider.notifier).setCustomVocabulary(value);
    });
  }

  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(settingsProvider);

    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
      children: [
        _buildSectionHeader('Recording'),
        const SizedBox(height: 8),
        _buildCard([
          _buildHotkeyRow(settings),
          _buildDivider(),
          _buildMicrophoneRow(settings),
          _buildDivider(),
          _buildSoundEffectsRow(settings),
          _buildDivider(),
          _buildMinDurationRow(settings),
        ]),
        const SizedBox(height: 24),
        _buildSectionHeader('Transcription'),
        const SizedBox(height: 8),
        _buildCard([
          _buildVocabularyRow(),
        ]),
      ],
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(left: 4),
      child: Text(
        title,
        style: const TextStyle(
          fontSize: 13,
          fontWeight: FontWeight.w600,
          color: Color(0xFF6E6E73),
        ),
      ),
    );
  }

  Widget _buildCard(List<Widget> children) {
    return Container(
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(10),
        boxShadow: const [
          BoxShadow(
            color: Color(0x0F000000),
            blurRadius: 1,
            offset: Offset(0, 1),
          ),
        ],
      ),
      child: Column(children: children),
    );
  }

  Widget _buildDivider() {
    return const Padding(
      padding: EdgeInsets.only(left: 16),
      child: Divider(height: 0.5, thickness: 0.5, color: Color(0xFFE8E8E8)),
    );
  }

  Widget _buildHotkeyRow(AppSettings settings) {
    return _buildRow(
      label: 'Push-to-talk key',
      trailing: _buildDropdown<String>(
        value: settings.selectedHotkey,
        items: _hotkeyOptions.entries
            .map(
                (e) => DropdownMenuItem(value: e.key, child: Text(e.value)))
            .toList(),
        onChanged: (value) {
          if (value != null) {
            ref.read(settingsProvider.notifier).setSelectedHotkey(value);
          }
        },
      ),
    );
  }

  Widget _buildMicrophoneRow(AppSettings settings) {
    // Build the device list. Always include a "System Default" option.
    final items = <DropdownMenuItem<String>>[
      const DropdownMenuItem(
          value: 'default', child: Text('System Default')),
      for (final device in _audioDevices)
        DropdownMenuItem(value: device.id, child: Text(device.name)),
    ];

    // Ensure the currently selected ID exists in the list; fall back to
    // 'default' if the device was removed (e.g. unplugged).
    final effectiveId =
        items.any((i) => i.value == settings.selectedMicrophoneId)
            ? settings.selectedMicrophoneId
            : 'default';

    return _buildRow(
      label: 'Microphone',
      trailing: _buildDropdown<String>(
        value: effectiveId,
        items: items,
        onChanged: (value) {
          if (value != null) {
            ref
                .read(settingsProvider.notifier)
                .setSelectedMicrophoneId(value);
          }
        },
      ),
    );
  }

  Widget _buildSoundEffectsRow(AppSettings settings) {
    return _buildRow(
      label: 'Sound effects',
      trailing: CupertinoSwitch(
        value: settings.soundEnabled,
        onChanged: (value) {
          ref.read(settingsProvider.notifier).setSoundEnabled(value);
        },
      ),
    );
  }

  Widget _buildMinDurationRow(AppSettings settings) {
    final durationMs = settings.minimumRecordingDurationMs;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Minimum recording duration',
                style: TextStyle(fontSize: 13),
              ),
              Text(
                '${durationMs.round()} ms',
                style: const TextStyle(
                  fontSize: 13,
                  color: Color(0xFF8E8E93),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SliderTheme(
            data: SliderThemeData(
              activeTrackColor: CupertinoColors.activeBlue,
              inactiveTrackColor: const Color(0xFFE0E0E0),
              thumbColor: Colors.white,
              overlayColor:
                  CupertinoColors.activeBlue.withValues(alpha: 0.12),
              trackHeight: 4,
              thumbShape:
                  const RoundSliderThumbShape(enabledThumbRadius: 8),
            ),
            child: Slider(
              value: durationMs,
              min: 100,
              max: 1000,
              divisions: 18, // 50ms steps
              onChanged: (value) {
                ref
                    .read(settingsProvider.notifier)
                    .setMinimumRecordingDurationMs(value);
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildVocabularyRow() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Custom vocabulary',
            style: TextStyle(fontSize: 13),
          ),
          const SizedBox(height: 4),
          const Text(
            'Words or phrases to improve recognition accuracy, '
            'one per line.',
            style: TextStyle(fontSize: 11, color: Color(0xFF8E8E93)),
          ),
          const SizedBox(height: 8),
          CupertinoTextField(
            controller: _vocabularyController,
            maxLines: 5,
            minLines: 3,
            placeholder: 'e.g.\nWrenflow\nRiverpod',
            onChanged: _onVocabularyChanged,
            padding: const EdgeInsets.all(10),
            style: const TextStyle(fontSize: 13),
            decoration: BoxDecoration(
              color: const Color(0xFFF5F5F7),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFFD1D1D6)),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRow({required String label, required Widget trailing}) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(fontSize: 13)),
          trailing,
        ],
      ),
    );
  }

  Widget _buildDropdown<T>({
    required T value,
    required List<DropdownMenuItem<T>> items,
    required ValueChanged<T?> onChanged,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: const Color(0xFFF5F5F7),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFFD1D1D6)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<T>(
          value: value,
          items: items,
          onChanged: onChanged,
          isDense: true,
          style: const TextStyle(fontSize: 13, color: Colors.black87),
          icon: const Icon(CupertinoIcons.chevron_down, size: 12),
        ),
      ),
    );
  }
}

/// About tab showing app version and project link.
class _AboutTab extends StatelessWidget {
  const _AboutTab();

  static const _githubUrl = 'https://github.com/nichochar/wrenflow';

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // App icon placeholder
            Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: const Color(0xFF5856D6),
                borderRadius: BorderRadius.circular(14),
              ),
              child: const Icon(
                CupertinoIcons.waveform,
                color: Colors.white,
                size: 32,
              ),
            ),
            const SizedBox(height: 16),
            const Text(
              'Wrenflow',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 4),
            const Text(
              'Version 1.0.0',
              style: TextStyle(
                fontSize: 13,
                color: Color(0xFF8E8E93),
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Hold a key to record, release to transcribe.',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 13,
                color: Color(0xFF8E8E93),
              ),
            ),
            const SizedBox(height: 24),
            CupertinoButton(
              padding: EdgeInsets.zero,
              onPressed: () => _openGitHub(),
              child: const Text(
                'View on GitHub',
                style: TextStyle(fontSize: 13),
              ),
            ),
          ],
        ),
      ),
    );
  }

  static Future<void> _openGitHub() async {
    final uri = Uri.parse(_githubUrl);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri);
    }
  }
}
