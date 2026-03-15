import { ScrollView, View, Text, TextInput, StyleSheet } from "react-native";
import { useState } from "react";

/** Prompts settings — system prompt and context prompt editors */
export default function PromptsSettings() {
  const [systemPrompt, setSystemPrompt] = useState("");
  const [contextPrompt, setContextPrompt] = useState("");

  return (
    <ScrollView style={styles.container}>
      <View style={styles.card}>
        <Text style={styles.cardTitle}>System Prompt</Text>
        <Text style={styles.caption}>
          Controls how raw transcriptions are cleaned up.
        </Text>
        <TextInput
          style={styles.editor}
          multiline
          value={systemPrompt}
          onChangeText={setSystemPrompt}
          placeholder="Using default system prompt..."
          placeholderTextColor="#aaa"
        />
      </View>

      <View style={styles.card}>
        <Text style={styles.cardTitle}>Context Prompt</Text>
        <Text style={styles.caption}>
          Controls how Wrenflow infers your current activity.
        </Text>
        <TextInput
          style={styles.editor}
          multiline
          value={contextPrompt}
          onChangeText={setContextPrompt}
          placeholder="Using default context prompt..."
          placeholderTextColor="#aaa"
        />
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  card: {
    backgroundColor: "#f5f5f5",
    borderRadius: 10,
    padding: 16,
    marginBottom: 12,
  },
  cardTitle: { fontSize: 16, fontWeight: "600", marginBottom: 8 },
  caption: { fontSize: 12, color: "#888", marginBottom: 8 },
  editor: {
    backgroundColor: "#fff",
    borderRadius: 6,
    borderWidth: 1,
    borderColor: "#ddd",
    padding: 12,
    minHeight: 120,
    fontFamily: "monospace",
    fontSize: 14,
    textAlignVertical: "top",
  },
});
