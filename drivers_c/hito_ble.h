#ifndef __hito_ble_h_included__
#define __hito_ble_h_included__

#include "stdint.h"
#include "stdbool.h"

#define HITO_BLE_MAX_PACKET_LEN 512

bool hito_ble_init();
void hito_ble_start();
void hito_ble_stop();

bool hito_ble_has_data();

bool hito_ble_has_error();
void hito_ble_error_clear();

bool hito_ble_has_data_package();
bool hito_ble_is_connected();

const void * hito_ble_data();
uint16_t     hito_ble_datalen();
void hito_ble_data_clear();
void hito_ble_data_package_clear();

const void * hito_ble_data_package();
uint16_t     hito_ble_data_package_len();

bool hito_ble_send(const void * data, uint32_t len);

#endif//__hito_ble_h_included__
