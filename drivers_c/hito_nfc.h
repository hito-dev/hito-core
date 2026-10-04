#ifndef __hito_nfc_h_included__
#define __hito_nfc_h_included__

#include <stdbool.h>
#include <stdint.h>

/** start nfc driver, returns true on success */
bool hito_nfc_start(const char * message);

/** stop nfc driver */
void hito_nfc_stop();

/** set nfc message */
int hito_nfc_set_message(const char * msg);

/** returns true if has new data received */
bool hito_nfc_has_data();

/** return received nfc data length */
uint32_t hito_nfc_data_len();

/** returns nfc data received */
const uint8_t * hito_nfc_data();

/** clears data in nfc buffer */
void hito_nfc_data_clear();


#endif//__hito_nfc_h_included__
