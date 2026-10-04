#ifndef __hito_ble_h_included__
#define __hito_ble_h_included__

#include "stdint.h"
#include "stdbool.h"

#define HITO_BLE_MAX_PACKET_LEN 512

bool hito_ble_init(uint8_t * payload_buffer, uint32_t payload_buffer_size);
void hito_ble_start();
void hito_ble_stop();

bool hito_ble_is_active();
bool hito_ble_is_connected();

// single BLE packet (<= 512 bytes, raw transport unit)
bool hito_ble_has_packet();
const void * hito_ble_packet();
uint16_t     hito_ble_packet_len();
void hito_ble_packet_clear();

bool hito_ble_has_error();
void hito_ble_error_clear();

// reconstructed payload (all packets assembled)
bool hito_ble_has_payload();
void hito_ble_payload_clear();

const void * hito_ble_payload();
uint16_t     hito_ble_payload_len();

bool hito_ble_send(const void * data, uint32_t len);

#endif//__hito_ble_h_included__
