<div align="center">

# react-native-temporal

**ECMAScript Temporal API for React Native**

_Powered by [temporal_rs](https://github.com/boa-dev/temporal) — the same Rust implementation used by the V8 JavaScript engine_

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![React Native](https://img.shields.io/badge/React%20Native-0.76+-61DAFB?logo=react)](https://reactnative.dev)

</div>

---

> [!WARNING] > **This library is under active development and is not ready for production use.**
> APIs may change without notice. Use at your own risk.

---

## Overview

`react-native-temporal` brings the [TC39 Temporal proposal](https://tc39.es/proposal-temporal/docs/) to React Native, providing a modern, robust API for working with dates, times, time zones, and calendars.

### Why temporal_rs?

This library is powered by **temporal_rs**, a high-performance Rust implementation of the Temporal specification. `temporal_rs` is battle-tested and trusted by major projects across the ecosystem:

- 🚀 **V8 JavaScript Engine** — The engine powering Google Chrome, Node.js, and Deno uses `temporal_rs` for its Temporal implementation
- 🦀 **Boa Engine** — A JavaScript engine written entirely in Rust
- 📱 **React Native** — Now available for mobile through this library

By leveraging the same implementation used in V8, `react-native-temporal` ensures spec-compliant behavior and production-grade reliability.

## Features

- ✅ Native performance via Rust FFI
- ✅ Works with both iOS and Android
- ✅ New Architecture (TurboModules) ready
- ✅ Spec-compliant date/time handling
- ✅ Proper time zone and calendar support

## API Implementation Status

| API                | Description                                              | Status         |
| ------------------ | -------------------------------------------------------- | -------------- |
| **Duration**       | Represents a length of time (days, hours, minutes, etc.) | ✅ Implemented |
| **Instant**        | A fixed point in time (UTC timestamp)                    | ✅ Implemented |
| **Now**            | System time access utilities                             | ✅ Implemented |
| **PlainTime**      | Time of day without date or timezone                     | ✅ Implemented |
| **Calendar**       | Calendar system support (ISO, Buddhist, Chinese, etc.)   | 🚧 Partial     |
| **PlainDate**      | Calendar date without time or timezone                   | ✅ Implemented |
| **PlainDateTime**  | Date and time without timezone                           | ✅ Implemented |
| **PlainYearMonth** | Year and month without day                               | ✅ Implemented |
| **PlainMonthDay**  | Month and day without year                               | ✅ Implemented |
| **TimeZone**       | IANA timezone or fixed UTC offset                        | 🚧 Partial     |
| **ZonedDateTime**  | Date/time with timezone (fully aware)                    | ✅ Implemented |

### Implementation Details

- **Duration**: Full API including `from`, all component getters, `add`, `subtract`, `negated`, `abs`, `compare`, `with`
- **Instant**: `now`, `from`, `fromEpochMilliseconds`, `fromEpochNanoseconds`, `epochMilliseconds`, `epochNanoseconds`, `add`, `subtract`, `compare`, `equals`, `until`, `since`, `round`, `toZonedDateTimeISO`, `toZonedDateTime`
- **Now**: `instant`, `timeZoneId`, `plainDateTimeISO`, `plainDateISO`, `plainTimeISO`, `zonedDateTimeISO`
- **PlainTime**: Full API including `from`, all component getters, `add`, `subtract`, `with`, `compare`, `equals`, `until`, `since`, `round`
- **Calendar**: `from`, `id` getter (missing: built-in calendar constants)
- **PlainDate**: Full API including `from`, getters, `add`, `subtract`, `compare`, `equals`, `with`, `until`, `since`
- **PlainDateTime**: Full API including `from`, getters, `add`, `subtract`, `compare`, `equals`, `with`, `until`, `since`, conversions
- **PlainYearMonth**: Full API including `from`, getters, `add`, `subtract`, `compare`, `equals`, `with`, `until`, `since`, `toPlainDate`
- **PlainMonthDay**: Full API including `from`, getters, `toPlainDate`
- **TimeZone**: `from`, `id`, `getOffsetNanosecondsFor`, `getOffsetStringFor`, `getPlainDateTimeFor`, `getInstantFor`, `getNextTransition`, `getPreviousTransition` (missing: `getPossibleInstantsFor`)
- **ZonedDateTime**: Full API including `from`, `epochMilliseconds`, `epochNanoseconds`, `calendar`, `timeZone`, `offset`, `add`, `subtract`, `with`, `until`, `since`, `round`, `compare`, `equals`, `startOfDay`, `hoursInDay`, conversion methods (`toInstant`, etc.)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

- [Development workflow](CONTRIBUTING.md#development-workflow)
- [Sending a pull request](CONTRIBUTING.md#sending-a-pull-request)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## License

MIT © [Szymon Chmal](LICENSE)

---

<div align="center">
  <sub>Built with ❤️ using <a href="https://github.com/callstack/react-native-builder-bob">create-react-native-library</a></sub>
</div>
