import { ScrollView, View, Text, Switch, StyleSheet } from "react-native";
import { useState } from "react";

/** General settings — matches macOS SettingsView GeneralSettingsView */
export default function GeneralSettings() {
  const [postProcessingEnabled, setPostProcessingEnabled] = useState(false);
  const [transcriptionProvider, setTranscriptionProvider] = useState<
    "local" | "groq"
  >("local");

  return (
    <ScrollView style={styles.container}>
      <Text style={styles.header}>Wrenflow</Text>
      <Text style={styles.version}>v0.1.0</Text>

      <SettingsCard title="Transcription">
        <View style={styles.row}>
          <Text>Provider</Text>
          <Text style={styles.value}>
            {transcriptionProvider === "local"
              ? "Local (Parakeet)"
              : "Groq (Whisper)"}
          </Text>
        </View>
        <Text style={styles.caption}>
          {transcriptionProvider === "local"
            ? "On-device, no internet needed"
            : "Cloud-based, requires API key"}
        </Text>
      </SettingsCard>

      <SettingsCard title="Post-Processing">
        <View style={styles.row}>
          <Text>Enable LLM post-processing</Text>
          <Switch
            value={postProcessingEnabled}
            onValueChange={setPostProcessingEnabled}
          />
        </View>
        <Text style={styles.caption}>
          When enabled, an LLM cleans up transcriptions using screen context.
        </Text>
      </SettingsCard>

      <SettingsCard title="API Key">
        <Text style={styles.caption}>
          Used for Groq transcription and post-processing.
        </Text>
      </SettingsCard>

      <SettingsCard title="Push-to-Talk Key">
        <Text style={styles.caption}>
          Hold to record, release to transcribe.
        </Text>
      </SettingsCard>

      <SettingsCard title="Microphone">
        <Text style={styles.caption}>System Default</Text>
      </SettingsCard>

      <SettingsCard title="Permissions">
        <Text style={styles.caption}>
          Microphone, Accessibility, Screen Recording
        </Text>
      </SettingsCard>
    </ScrollView>
  );
}

function SettingsCard({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <View style={styles.card}>
      <Text style={styles.cardTitle}>{title}</Text>
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  header: { fontSize: 20, fontWeight: "bold", textAlign: "center" },
  version: {
    fontSize: 12,
    color: "#888",
    textAlign: "center",
    marginBottom: 16,
  },
  card: {
    backgroundColor: "#f5f5f5",
    borderRadius: 10,
    padding: 16,
    marginBottom: 12,
  },
  cardTitle: { fontSize: 16, fontWeight: "600", marginBottom: 8 },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 4,
  },
  value: { color: "#007AFF" },
  caption: { fontSize: 12, color: "#888", marginTop: 4 },
});
