import { Tabs } from "expo-router";

export default function Layout() {
  return (
    <Tabs
      screenOptions={{
        headerShown: true,
        tabBarActiveTintColor: "#007AFF",
      }}
    >
      <Tabs.Screen
        name="index"
        options={{ title: "General", tabBarIcon: () => null }}
      />
      <Tabs.Screen
        name="settings/prompts"
        options={{ title: "Prompts", tabBarIcon: () => null }}
      />
      <Tabs.Screen
        name="settings/run-log"
        options={{ title: "Run Log", tabBarIcon: () => null }}
      />
    </Tabs>
  );
}
