// Minimal string.h for RAIL bindings generation
// Provides only the declarations needed by RAIL without macOS-specific code

#ifndef _STRING_H_
#define _STRING_H_

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Memory manipulation functions used by RAIL
void *memcpy(void *dest, const void *src, size_t n);
void *memset(void *s, int c, size_t n);
void *memmove(void *dest, const void *src, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);

#ifdef __cplusplus
}
#endif

#endif // _STRING_H_
