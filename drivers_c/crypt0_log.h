#ifndef __crypt0_logger_h_included__
#define __crypt0_logger_h_included__

#ifdef __ZEPHYR__

#include <logging/log.h>

#else

enum {
  LOG_LEVEL_INF = 1,
  LOG_LEVEL_DBG = 2,
  LOG_LEVEL_ERR = 3,
};

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#undef LOG_MODULE_REGISTER
#undef LOG_MODULE_CHEKAMOR
#undef LOG_HEXDUMP_DBG
#undef LOG_DBG
#undef LOG_ERR

#define LOG_MODULE_REGISTER(module, level) static const char * crypt0_log_module_name = #module; static int crypt0_log_level = level;

void print_hex(const void* data, int size);

#define LOG_HEXDUMP_DBG(val, len, header) { \
  fprintf(stderr, "<dbg> %s.%s.%s:\n", crypt0_log_module_name, __func__, header);\
  print_hex(val, len); \
}

#define LOG_DBG(...) { fprintf(stderr, "<dbg> %s.%s: ", crypt0_log_module_name, __func__); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); }
#define LOG_ERR(...) { fprintf(stderr, "<err> %s: ", __func__); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); }

#endif//__ZEPHYR__
#endif//__crypt0_logger_h_included__
