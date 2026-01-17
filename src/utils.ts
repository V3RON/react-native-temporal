/**
 * Wraps a native call and ensures proper TC39 Temporal error types are thrown.
 * Native exceptions come through as generic Errors, so we re-throw
 * them as RangeError or TypeError to match the Temporal specification.
 */
export const wrapNativeCall = <T>(fn: () => T, context: string): T => {
  try {
    return fn();
  } catch (error) {
    if (error instanceof Error) {
      let message = error.message || '';

      // Strip React Native's "Exception in HostFunction: " prefix
      message = message.replace(/^Exception in HostFunction:\s*/i, '');

      // Check for error type markers
      const isTypeError =
        error.name === 'TypeError' ||
        message.startsWith('[TypeError]') ||
        message.toLowerCase().includes('cannot be null') ||
        message.toLowerCase().includes('type error');

      // Clean up any [ErrorType] prefix from the message
      message = message.replace(/^\[(RangeError|TypeError)\]\s*/i, '');

      if (isTypeError) {
        throw new TypeError(message || context);
      }
      throw new RangeError(message || context);
    }
    throw new RangeError(context);
  }
};
