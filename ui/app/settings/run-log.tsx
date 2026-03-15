import {
  ScrollView,
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
} from "react-native";
import { useState } from "react";

/** Run log — pipeline history viewer */
export default function RunLogSettings() {
  const [entries] = useState<RunLogEntry[]>([]);

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.headerTitle}>Run Log</Text>
        <Text style={styles.headerCaption}>
          Stored locally. Only the 20 most recent runs are kept.
        </Text>
      </View>

      {entries.length === 0 ? (
        <View style={styles.empty}>
          <Text style={styles.emptyText}>
            No runs yet. Use dictation to populate history.
          </Text>
        </View>
      ) : (
        entries.map((entry) => (
          <RunLogEntryCard key={entry.id} entry={entry} />
        ))
      )}
    </ScrollView>
  );
}

interface RunLogEntry {
  id: string;
  timestamp: number;
  rawTranscript: string;
  postProcessedTranscript: string;
  totalMs?: number;
}

function RunLogEntryCard({ entry }: { entry: RunLogEntry }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <TouchableOpacity
      style={styles.card}
      onPress={() => setExpanded(!expanded)}
    >
      <View style={styles.row}>
        <Text style={styles.timestamp}>
          {new Date(entry.timestamp * 1000).toLocaleString()}
        </Text>
        {entry.totalMs && (
          <Text style={styles.badge}>
            {entry.totalMs >= 1000
              ? `${(entry.totalMs / 1000).toFixed(1)}s`
              : `${Math.round(entry.totalMs)}ms`}
          </Text>
        )}
      </View>
      <Text style={styles.transcript} numberOfLines={expanded ? undefined : 1}>
        {entry.postProcessedTranscript || "(no transcript)"}
      </Text>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  header: { marginBottom: 16 },
  headerTitle: { fontSize: 18, fontWeight: "bold" },
  headerCaption: { fontSize: 12, color: "#888" },
  empty: { flex: 1, alignItems: "center", paddingTop: 60 },
  emptyText: { color: "#888" },
  card: {
    backgroundColor: "#f5f5f5",
    borderRadius: 10,
    padding: 12,
    marginBottom: 8,
  },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 4,
  },
  timestamp: { fontSize: 13, fontWeight: "600" },
  badge: {
    fontSize: 11,
    color: "#888",
    backgroundColor: "#e8e8e8",
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  transcript: { fontSize: 13, color: "#666" },
});
