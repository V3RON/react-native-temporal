<div align="center">

# react-native-temporal

**ECMAScript Temporal API for React Native**

*Powered by [temporal_rs](https://github.com/boa-dev/temporal) — the same Rust implementation used by the V8 JavaScript engine*

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![React Native](https://img.shields.io/badge/React%20Native-0.76+-61DAFB?logo=react)](https://reactnative.dev)

</div>

---

> [!WARNING]
> **This library is under active development and is not ready for production use.**
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

- ✅ Full Temporal API support
- ✅ Native performance via Rust FFI
- ✅ Works with both iOS and Android
- ✅ New Architecture (TurboModules) ready
- ✅ Spec-compliant date/time handling
- ✅ Proper time zone and calendar support

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
