/* temporal-rn C bindings */
#ifndef TEMPORAL_RN_H
#define TEMPORAL_RN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Returns the current instant as an ISO 8601 string (e.g., "2024-01-15T10:30:45.123Z").
 * The caller is responsible for freeing the returned string using `temporal_free_string`.
 *
 * Returns NULL on error.
 */
char *temporal_instant_now(void);

/**
 * Frees a string allocated by temporal functions.
 */
void temporal_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* TEMPORAL_RN_H */
