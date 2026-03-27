import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:wrenflow/providers/history_provider.dart';
import 'package:wrenflow/src/bindings/signals/signals.dart';

class HistoryScreen extends ConsumerStatefulWidget {
  const HistoryScreen({super.key});

  @override
  ConsumerState<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends ConsumerState<HistoryScreen> {
  @override
  void initState() {
    super.initState();
    // Request history from Rust on open
    LoadHistory().sendSignalToRust();
  }

  @override
  Widget build(BuildContext context) {
    final entries = ref.watch(historyProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('History'),
        actions: [
          if (entries.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.delete_sweep),
              tooltip: 'Clear all',
              onPressed: () => _confirmClearAll(context),
            ),
        ],
      ),
      body: entries.isEmpty
          ? const Center(
              child: Text(
                'No transcriptions yet',
                style: TextStyle(color: Colors.grey),
              ),
            )
          : ListView.builder(
              itemCount: entries.length,
              padding: const EdgeInsets.all(8),
              itemBuilder: (context, index) {
                final entry = entries[index];
                return _HistoryTile(
                  entry: entry,
                  onDelete: () => _deleteEntry(entry.id),
                );
              },
            ),
    );
  }

  void _deleteEntry(String id) {
    ref.read(historyProvider.notifier).removeEntry(id);
    DeleteHistoryEntry(id: id).sendSignalToRust();
  }

  void _confirmClearAll(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Clear History'),
        content: const Text('Delete all transcription history?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              ref.read(historyProvider.notifier).clearAll();
              ClearHistory().sendSignalToRust();
            },
            child: const Text('Clear', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }
}

class _HistoryTile extends StatelessWidget {
  final HistoryEntryData entry;
  final VoidCallback onDelete;

  const _HistoryTile({required this.entry, required this.onDelete});

  @override
  Widget build(BuildContext context) {
    final date = DateTime.fromMillisecondsSinceEpoch(
      (entry.timestamp * 1000).toInt(),
    );
    final timeStr =
        '${date.hour.toString().padLeft(2, '0')}:${date.minute.toString().padLeft(2, '0')}';
    final dateStr =
        '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';

    return Card(
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: ListTile(
        title: Text(
          entry.transcript,
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text('$dateStr $timeStr'),
        trailing: IconButton(
          icon: const Icon(Icons.delete_outline, size: 20),
          onPressed: onDelete,
        ),
        onTap: () => _showDetail(context),
      ),
    );
  }

  void _showDetail(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Transcription'),
        content: SingleChildScrollView(
          child: SelectableText(entry.transcript),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}
