import { useEffect, useState } from 'react';
import {
  Text,
  View,
  StyleSheet,
  Button,
  ScrollView,
  TextInput,
  TouchableOpacity,
} from 'react-native';
import {
  multiply,
  Instant,
  Duration,
  PlainDateTime,
} from 'react-native-temporal';

const multiplyResult = multiply(3, 7);

const formatComponents = (
  duration: ReturnType<typeof Duration.from>
): string => {
  return `
Years: ${duration.years}
Months: ${duration.months}
Weeks: ${duration.weeks}
Days: ${duration.days}
Hours: ${duration.hours}
Minutes: ${duration.minutes}
Seconds: ${duration.seconds}
Milliseconds: ${duration.milliseconds}
Microseconds: ${duration.microseconds}
Nanoseconds: ${duration.nanoseconds}
  `.trim();
};

export default function App() {
  const [currentInstant, setCurrentInstant] = useState<string>('');

  // Duration test state
  const [durationInput1, setDurationInput1] =
    useState<string>('P1Y2M3DT4H5M6S');
  const [durationInput2, setDurationInput2] = useState<string>('P1M2DT3H');
  const [parsedDuration, setParsedDuration] = useState<string>('');
  const [durationComponents, setDurationComponents] = useState<string>('');
  const [addResult, setAddResult] = useState<string>('');
  const [subtractResult, setSubtractResult] = useState<string>('');
  const [negatedResult, setNegatedResult] = useState<string>('');
  const [absResult, setAbsResult] = useState<string>('');
  const [signResult, setSignResult] = useState<string>('');
  const [isZeroResult, setIsZeroResult] = useState<string>('');
  const [error, setError] = useState<string>('');

  // PlainDateTime test state
  const [dateTimeInput, setDateTimeInput] = useState<string>(
    '2024-03-14T15:30:00'
  );
  const [parsedDateTime, setParsedDateTime] = useState<string>('');
  const [dateTimeComponents, setDateTimeComponents] = useState<string>('');
  const [dateTimeAddResult, setDateTimeAddResult] = useState<string>('');

  const updateInstant = () => {
    const instant = Instant.now();
    setCurrentInstant(instant.toString());
  };

  const testDateTimeParsing = () => {
    try {
      setError('');
      const dt = PlainDateTime.from(dateTimeInput);
      setParsedDateTime(dt.toString());
      setDateTimeComponents(
        `Y:${dt.year} M:${dt.month} D:${dt.day} H:${dt.hour} m:${dt.minute} s:${dt.second}`
      );

      // Test add
      const added = dt.add('P1D');
      setDateTimeAddResult(`${dt.toString()} + P1D = ${added.toString()}`);
    } catch (e) {
      setError(`DateTime error: ${e}`);
      setParsedDateTime('');
    }
  };

  const testDurationParsing = () => {
    try {
      setError('');
      const duration = Duration.from(durationInput1);
      setParsedDuration(duration.toString());
      setDurationComponents(formatComponents(duration));
    } catch (e) {
      setError(`Parse error: ${e}`);
      setParsedDuration('');
      setDurationComponents('');
    }
  };

  const testDurationArithmetic = () => {
    try {
      setError('');
      const d1 = Duration.from(durationInput1);
      const d2 = Duration.from(durationInput2);

      // Add
      const sum = d1.add(d2);
      setAddResult(`${d1.toString()} + ${d2.toString()} = ${sum.toString()}`);

      // Subtract
      const diff = d1.subtract(d2);
      setSubtractResult(
        `${d1.toString()} - ${d2.toString()} = ${diff.toString()}`
      );

      // Negated
      const neg = d1.negated();
      setNegatedResult(`-${d1.toString()} = ${neg.toString()}`);

      // Abs
      const abs = neg.abs();
      setAbsResult(`abs(${neg.toString()}) = ${abs.toString()}`);

      // Sign
      const sign = d1.sign;
      setSignResult(`sign(${d1.toString()}) = ${sign}`);

      // Blank (TC39 property name for zero check)
      const isBlank = d1.blank;
      setIsZeroResult(`blank(${d1.toString()}) = ${isBlank}`);
    } catch (e) {
      setError(`Arithmetic error: ${e}`);
    }
  };

  const testObjectCreation = () => {
    try {
      setError('');
      const duration = Duration.from({
        years: 1,
        months: 2,
        days: 3,
        hours: 4,
        minutes: 30,
        seconds: 15,
      });
      setDurationInput1(duration.toString());
      setParsedDuration(duration.toString());
    } catch (e) {
      setError(`Object creation error: ${e}`);
    }
  };

  const testZeroDuration = () => {
    try {
      setError('');
      const zero = Duration.from('PT0S');
      setDurationInput1(zero.toString());
      setParsedDuration(zero.toString());
      setIsZeroResult(`blank(${zero.toString()}) = ${zero.blank}`);
    } catch (e) {
      setError(`Zero duration error: ${e}`);
    }
  };

  useEffect(() => {
    updateInstant();
    // Initial parse on mount
    try {
      const duration = Duration.from(durationInput1);
      setParsedDuration(duration.toString());
      setDurationComponents(formatComponents(duration));
    } catch (e) {
      setError(`Parse error: ${e}`);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <ScrollView style={styles.container}>
      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Stage 1: Duration API</Text>

        <Text style={styles.label}>Multiply Test (legacy):</Text>
        <Text style={styles.value}>{multiplyResult}</Text>

        <Text style={styles.label}>Current Instant:</Text>
        <Text style={styles.value}>{currentInstant}</Text>
        <Button title="Refresh Instant" onPress={updateInstant} />
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Duration Parsing</Text>

        <Text style={styles.inputLabel}>Duration 1:</Text>
        <TextInput
          style={styles.input}
          value={durationInput1}
          onChangeText={setDurationInput1}
          placeholder="P1Y2M3DT4H5M6S"
        />

        <Text style={styles.inputLabel}>Duration 2:</Text>
        <TextInput
          style={styles.input}
          value={durationInput2}
          onChangeText={setDurationInput2}
          placeholder="P1M2DT3H"
        />

        <View style={styles.buttonRow}>
          <TouchableOpacity
            style={styles.smallButton}
            onPress={testDurationParsing}
          >
            <Text style={styles.buttonText}>Parse D1</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={styles.smallButton}
            onPress={testObjectCreation}
          >
            <Text style={styles.buttonText}>From Object</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={styles.smallButton}
            onPress={testZeroDuration}
          >
            <Text style={styles.buttonText}>Zero</Text>
          </TouchableOpacity>
        </View>

        {error ? <Text style={styles.error}>{error}</Text> : null}

        {parsedDuration ? (
          <>
            <Text style={styles.label}>Parsed Duration:</Text>
            <Text style={styles.value}>{parsedDuration}</Text>

            <Text style={styles.label}>Components:</Text>
            <Text style={styles.value}>{durationComponents}</Text>
          </>
        ) : null}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Duration Arithmetic</Text>

        <Button title="Test All Operations" onPress={testDurationArithmetic} />

        {addResult ? (
          <>
            <Text style={styles.label}>Addition:</Text>
            <Text style={styles.value}>{addResult}</Text>
          </>
        ) : null}

        {subtractResult ? (
          <>
            <Text style={styles.label}>Subtraction:</Text>
            <Text style={styles.value}>{subtractResult}</Text>
          </>
        ) : null}

        {negatedResult ? (
          <>
            <Text style={styles.label}>Negation:</Text>
            <Text style={styles.value}>{negatedResult}</Text>
          </>
        ) : null}

        {absResult ? (
          <>
            <Text style={styles.label}>Absolute Value:</Text>
            <Text style={styles.value}>{absResult}</Text>
          </>
        ) : null}

        {signResult ? (
          <>
            <Text style={styles.label}>Sign:</Text>
            <Text style={styles.value}>{signResult}</Text>
          </>
        ) : null}

        {isZeroResult ? (
          <>
            <Text style={styles.label}>Blank:</Text>
            <Text style={styles.value}>{isZeroResult}</Text>
          </>
        ) : null}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>PlainDateTime</Text>
        <TextInput
          style={styles.input}
          value={dateTimeInput}
          onChangeText={setDateTimeInput}
          placeholder="ISO DateTime"
        />
        <Button title="Test DateTime" onPress={testDateTimeParsing} />

        {parsedDateTime ? (
          <>
            <Text style={styles.label}>Parsed:</Text>
            <Text style={styles.value}>{parsedDateTime}</Text>
            <Text style={styles.label}>Components:</Text>
            <Text style={styles.value}>{dateTimeComponents}</Text>
            <Text style={styles.label}>Add P1D:</Text>
            <Text style={styles.value}>{dateTimeAddResult}</Text>
          </>
        ) : null}
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 20,
    backgroundColor: '#f5f5f5',
  },
  section: {
    backgroundColor: 'white',
    padding: 15,
    marginBottom: 20,
    borderRadius: 10,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  sectionTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    marginBottom: 15,
    color: '#333',
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    marginTop: 12,
    marginBottom: 4,
    color: '#555',
  },
  inputLabel: {
    fontSize: 14,
    fontWeight: '600',
    marginTop: 8,
    marginBottom: 4,
    color: '#555',
  },
  value: {
    fontSize: 13,
    fontFamily: 'monospace',
    backgroundColor: '#f8f8f8',
    padding: 8,
    borderRadius: 4,
    color: '#333',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 6,
    padding: 10,
    fontSize: 13,
    fontFamily: 'monospace',
    backgroundColor: '#fff',
    marginBottom: 8,
  },
  buttonRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 10,
    marginBottom: 10,
  },
  smallButton: {
    backgroundColor: '#007AFF',
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderRadius: 6,
    flex: 1,
    marginHorizontal: 4,
  },
  buttonText: {
    color: 'white',
    fontSize: 12,
    fontWeight: '600',
    textAlign: 'center',
  },
  error: {
    color: '#ff3b30',
    fontSize: 13,
    marginTop: 8,
    padding: 8,
    backgroundColor: '#ffebee',
    borderRadius: 4,
  },
});
