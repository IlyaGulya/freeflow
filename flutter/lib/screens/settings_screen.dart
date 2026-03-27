import 'dart:async';

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:rinf/rinf.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:wrenflow/providers/settings_provider.dart';
import 'package:wrenflow/src/bindings/signals/signals.dart';
import 'package:wrenflow/theme/wrenflow_theme.dart';
import 'package:wrenflow/widgets/green_toggle.dart';
import 'package:wrenflow/widgets/settings_card.dart';

/// Available hotkey options for the push-to-talk trigger.
const _hotkeyOptions = <String, String>{
  'fn': 'Fn',
  'rightOption': 'Right Option',
  'f5': 'F5',
};

/// Sidebar tab definition.
enum _SettingsTab {
  general(CupertinoIcons.gear, 'General'),
  about(CupertinoIcons.info, 'About');

  const _SettingsTab(this.icon, this.label);
  final IconData icon;
  final String label;
}

/// Settings screen — 720×520, sidebar + content layout.
class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  _SettingsTab _selectedTab = _SettingsTab.general;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: WrenflowStyle.bg,
      body: Row(
        children: [
          // Sidebar
          _buildSidebar(),

          // Divider
          Container(width: 0.5, color: WrenflowStyle.border),

          // Content
          Expanded(
            child: _selectedTab == _SettingsTab.general
                ? const _GeneralContent()
                : const _AboutContent(),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebar() {
    return SizedBox(
      width: 150,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Traffic light inset
          const SizedBox(height: 28),

          // App icon
          Opacity(
            opacity: 0.6,
            child: Image.asset(
              'assets/icon.png',
              width: 64,
              height: 64,
              errorBuilder: (_, __, ___) => Icon(
                CupertinoIcons.waveform,
                size: 40,
                color: WrenflowStyle.textOp60,
              ),
            ),
          ),
          const SizedBox(height: 8),

          // App name
          Text('Wrenflow', style: WrenflowStyle.body(12)),
          const SizedBox(height: 2),

          // Version
          Text(
            'v1.0.0',
            style: WrenflowStyle.mono(10).copyWith(
              color: WrenflowStyle.textTertiary,
            ),
          ),
          const SizedBox(height: 12),

          // Divider
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Container(height: 0.5, color: WrenflowStyle.border),
          ),
          const SizedBox(height: 8),

          // Tab buttons
          for (final tab in _SettingsTab.values)
            _buildTabButton(tab),

          const Spacer(),
        ],
      ),
    );
  }

  Widget _buildTabButton(_SettingsTab tab) {
    final isSelected = _selectedTab == tab;
    return GestureDetector(
      onTap: () => setState(() => _selectedTab = tab),
      child: Container(
        width: double.infinity,
        margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 1),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: BoxDecoration(
          color: isSelected ? WrenflowStyle.textOp07 : Colors.transparent,
          borderRadius: BorderRadius.circular(WrenflowStyle.radiusSmall),
        ),
        child: Row(
          children: [
            Icon(
              tab.icon,
              size: 11,
              color: isSelected ? WrenflowStyle.text : WrenflowStyle.textTertiary,
            ),
            const SizedBox(width: 6),
            Text(
              tab.label,
              style: WrenflowStyle.body(13).copyWith(
                color: isSelected
                    ? WrenflowStyle.text
                    : WrenflowStyle.textSecondary,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ── General tab content ──────────────────────────────────────

class _GeneralContent extends ConsumerStatefulWidget {
  const _GeneralContent();

  @override
  ConsumerState<_GeneralContent> createState() => _GeneralContentState();
}

class _GeneralContentState extends ConsumerState<_GeneralContent> {
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

    _deviceSubscription =
        AudioDevicesListed.rustSignalStream.listen((signal) {
      if (mounted) {
        setState(() => _audioDevices = signal.message.devices);
      }
    });

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

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Hotkey card
          SettingsCard(
            title: 'Push-to-talk key',
            child: _buildHotkeyOptions(settings),
          ),
          const SizedBox(height: 16),

          // Microphone card
          SettingsCard(
            title: 'Microphone',
            child: _buildMicrophoneDropdown(settings),
          ),
          const SizedBox(height: 16),

          // Sound effects card
          SettingsCard(
            title: 'Sound effects',
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('Play sounds', style: WrenflowStyle.body(12)),
                GreenToggle(
                  value: settings.soundEnabled,
                  onChanged: (v) =>
                      ref.read(settingsProvider.notifier).setSoundEnabled(v),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),

          // Min duration card
          SettingsCard(
            title: 'Minimum recording duration',
            child: _buildDurationSlider(settings),
          ),
          const SizedBox(height: 16),

          // Vocabulary card
          SettingsCard(
            title: 'Custom vocabulary',
            child: _buildVocabularyField(),
          ),
        ],
      ),
    );
  }

  Widget _buildHotkeyOptions(AppSettings settings) {
    return Column(
      children: _hotkeyOptions.entries.map((entry) {
        final isSelected = settings.selectedHotkey == entry.key;
        return GestureDetector(
          onTap: () =>
              ref.read(settingsProvider.notifier).setSelectedHotkey(entry.key),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 10),
            margin: const EdgeInsets.only(bottom: 2),
            decoration: BoxDecoration(
              color:
                  isSelected ? WrenflowStyle.textOp05 : Colors.transparent,
              borderRadius: BorderRadius.circular(7),
            ),
            child: Row(
              children: [
                Icon(
                  isSelected
                      ? CupertinoIcons.checkmark_circle_fill
                      : CupertinoIcons.circle,
                  size: 13,
                  color: isSelected
                      ? WrenflowStyle.text
                      : WrenflowStyle.textTertiary,
                ),
                const SizedBox(width: 8),
                Text(entry.value, style: WrenflowStyle.body(12)),
              ],
            ),
          ),
        );
      }).toList(),
    );
  }

  Widget _buildMicrophoneDropdown(AppSettings settings) {
    final items = <_DropdownItem>[
      const _DropdownItem('default', 'System Default'),
      for (final device in _audioDevices)
        _DropdownItem(device.id, device.name),
    ];

    final effectiveId =
        items.any((i) => i.value == settings.selectedMicrophoneId)
            ? settings.selectedMicrophoneId
            : 'default';

    return Column(
      children: items.map((item) {
        final isSelected = effectiveId == item.value;
        return GestureDetector(
          onTap: () => ref
              .read(settingsProvider.notifier)
              .setSelectedMicrophoneId(item.value),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 7, horizontal: 10),
            margin: const EdgeInsets.only(bottom: 2),
            decoration: BoxDecoration(
              color:
                  isSelected ? WrenflowStyle.textOp05 : Colors.transparent,
              borderRadius: BorderRadius.circular(7),
            ),
            child: Row(
              children: [
                Icon(
                  isSelected
                      ? CupertinoIcons.checkmark_circle_fill
                      : CupertinoIcons.circle,
                  size: 13,
                  color: isSelected
                      ? WrenflowStyle.text
                      : WrenflowStyle.textTertiary,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    item.label,
                    style: WrenflowStyle.body(12),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        );
      }).toList(),
    );
  }

  Widget _buildDurationSlider(AppSettings settings) {
    final durationMs = settings.minimumRecordingDurationMs;
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text('Duration', style: WrenflowStyle.body(12)),
            Text(
              '${durationMs.round()} ms',
              style: WrenflowStyle.mono(10).copyWith(
                color: WrenflowStyle.textTertiary,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        SliderTheme(
          data: SliderThemeData(
            activeTrackColor: WrenflowStyle.trackFill,
            inactiveTrackColor: WrenflowStyle.trackBg,
            thumbColor: Colors.white,
            overlayColor: WrenflowStyle.textOp10,
            trackHeight: 3,
            thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7),
          ),
          child: Slider(
            value: durationMs,
            min: 100,
            max: 1000,
            divisions: 18,
            onChanged: (value) => ref
                .read(settingsProvider.notifier)
                .setMinimumRecordingDurationMs(value),
          ),
        ),
      ],
    );
  }

  Widget _buildVocabularyField() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Words or phrases to improve recognition, one per line.',
          style: WrenflowStyle.caption(11),
        ),
        const SizedBox(height: 8),
        Container(
          height: 64,
          decoration: BoxDecoration(
            color: WrenflowStyle.bg,
            borderRadius: BorderRadius.circular(7),
            border: Border.all(color: WrenflowStyle.border, width: 1),
          ),
          child: TextField(
            controller: _vocabularyController,
            maxLines: null,
            expands: true,
            onChanged: _onVocabularyChanged,
            style: WrenflowStyle.mono(11),
            decoration: const InputDecoration(
              border: InputBorder.none,
              contentPadding: EdgeInsets.all(8),
              hintText: 'e.g.\nWrenflow\nRiverpod',
              hintStyle: TextStyle(
                fontFamily: 'Menlo',
                fontSize: 11,
                color: Color.fromRGBO(153, 153, 153, 1.0),
              ),
              isDense: true,
            ),
          ),
        ),
      ],
    );
  }
}

class _DropdownItem {
  const _DropdownItem(this.value, this.label);
  final String value;
  final String label;
}

// ── About tab content ────────────────────────────────────────

class _AboutContent extends StatelessWidget {
  const _AboutContent();

  static const _githubUrl = 'https://github.com/nichochar/wrenflow';

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Opacity(
            opacity: 0.6,
            child: Image.asset(
              'assets/icon.png',
              width: 64,
              height: 64,
              errorBuilder: (_, __, ___) => Icon(
                CupertinoIcons.waveform,
                size: 40,
                color: WrenflowStyle.textOp60,
              ),
            ),
          ),
          const SizedBox(height: 12),
          Text('Wrenflow', style: WrenflowStyle.title(16)),
          const SizedBox(height: 4),
          Text(
            'v1.0.0',
            style: WrenflowStyle.mono(10).copyWith(
              color: WrenflowStyle.textTertiary,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'Hold a key to record, release to transcribe.',
            style: WrenflowStyle.caption(12),
          ),
          const SizedBox(height: 20),
          GestureDetector(
            onTap: () async {
              final uri = Uri.parse(_githubUrl);
              if (await canLaunchUrl(uri)) await launchUrl(uri);
            },
            child: Text(
              'View on GitHub',
              style: WrenflowStyle.body(12).copyWith(
                color: WrenflowStyle.textOp50,
                decoration: TextDecoration.underline,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
