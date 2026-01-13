import { useEffect, useState } from 'react';
import { Text, View, StyleSheet, Button } from 'react-native';
import { multiply, Instant } from 'react-native-temporal';

const multiplyResult = multiply(3, 7);

export default function App() {
  const [currentInstant, setCurrentInstant] = useState<string>('');

  const updateInstant = () => {
    const instant = Instant.now();
    setCurrentInstant(instant);
  };

  useEffect(() => {
    updateInstant();
  }, []);

  return (
    <View style={styles.container}>
      <Text style={styles.label}>Multiply Result:</Text>
      <Text style={styles.value}>{multiplyResult}</Text>

      <Text style={styles.label}>Current Instant (ISO 8601):</Text>
      <Text style={styles.value}>{currentInstant}</Text>

      <View style={styles.buttonContainer}>
        <Button title="Refresh Instant" onPress={updateInstant} />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
  },
  label: {
    fontSize: 16,
    fontWeight: 'bold',
    marginTop: 20,
  },
  value: {
    fontSize: 14,
    marginTop: 5,
    fontFamily: 'monospace',
  },
  buttonContainer: {
    marginTop: 30,
  },
});
