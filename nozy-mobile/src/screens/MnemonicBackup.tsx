import * as Clipboard from "expo-clipboard";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "MnemonicBackup">;

export function MnemonicBackupScreen({ navigation, route }: Props) {
  const { mnemonic } = route.params;
  const [copied, setCopied] = useState(false);
  const words = mnemonic.trim().split(/\s+/);

  async function copyMnemonic() {
    await Clipboard.setStringAsync(mnemonic);
    setCopied(true);
  }

  return (
    <SafeAreaView style={styles.safe}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.title}>Back up your seed phrase</Text>
        <Text style={styles.warning}>
          Write these words down and store them safely. Anyone with this phrase
          can access your funds.
        </Text>

        <View style={styles.grid}>
          {words.map((word, index) => (
            <View key={`${word}-${index}`} style={styles.wordChip}>
              <Text style={styles.wordIndex}>{index + 1}</Text>
              <Text style={styles.word}>{word}</Text>
            </View>
          ))}
        </View>

        <Button
          label={copied ? "Copied!" : "Copy to clipboard"}
          variant="secondary"
          onPress={() => void copyMnemonic()}
        />
        <Button
          label="I saved my seed phrase"
          onPress={() => navigation.replace("Dashboard")}
        />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  container: {
    padding: spacing.lg,
    gap: spacing.md,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: "800",
  },
  warning: {
    color: colors.error,
    fontSize: fontSize.md,
    lineHeight: 22,
  },
  grid: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: spacing.sm,
  },
  wordChip: {
    width: "48%",
    backgroundColor: colors.surface,
    borderRadius: 10,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    flexDirection: "row",
    gap: spacing.sm,
    alignItems: "center",
  },
  wordIndex: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    width: 20,
  },
  word: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: "600",
  },
});
