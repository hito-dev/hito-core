#include <zephyr/types.h>
#include <random/rand32.h>
#include <stddef.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>
#include <sys/printk.h>
#include <sys/byteorder.h>
#include <zephyr.h>
#include <drivers/gpio.h>
#include <nrf.h>
#if defined(NRF5340_XXAA_APPLICATION) || defined(NRF5340_XXAA_NETWORK)
    #include <hal/nrf_reset.h>
    // Use nrf_reset_* functions
#else
    #include <hal/nrf_power.h>
    // Use nrf_power_resetreas_* functions
#endif
#include <sys/byteorder.h>

#include <bluetooth/bluetooth.h>
#include <bluetooth/hci.h>
#include <bluetooth/conn.h>
#include <bluetooth/uuid.h>
#include <bluetooth/gatt.h>

#include <logging/log.h>
LOG_MODULE_REGISTER(boot_ble);

#include "hito_ble.h"

/* Button value. */
static uint16_t but_val;

/* Prototype */
static ssize_t hito_ble_recv(struct bt_conn *conn,
		    const struct bt_gatt_attr *attr, const void *buf,
		    uint16_t len, uint16_t offset, uint8_t flags);

/* ST Custom Service  */
static struct bt_uuid_128 st_service_uuid = BT_UUID_INIT_128(

	0x02, 0x00, 0x12, 0xac, 0x42, 0x02, 0x39, 0xb9, // hito
	0xed, 0x11, 0x0a, 0x07, 0x16, 0x4b, 0xc4, 0x5c); // hito
	//0x02, 0x00, 0x12, 0xac, 0x42, 0x02, 0x39, 0xb9, 
	//0xed, 0x11, 0x0a, 0x07, 0x16, 0x4b, 0xc4, 0x5c);
	//6E400001-B5A3-F393-E0A9-E50E24DCCA9E - nordic uart
	//0x9e, 0xca, 0xdc, 0x24, 0x03, 0xe5, 0xa9, 0xe0, 
	//0x93, 0xf3, 0xa3, 0xb5, 0x01, 0x00, 0x40, 0x6e);
	

//5cc44b16-070a-11ed-b939-0242ac120002

/* ST LED service */
static struct bt_uuid_128 led_char_uuid = BT_UUID_INIT_128(
	//0x19, 0xed, 0x82, 0xae, 0xed, 0x21, 0x4c, 0x9d,
	//0x41, 0x45, 0x22, 0x8e, 0x41, 0xfe, 0x00, 0x00);
	
	0x02, 0x00, 0x12, 0xac, 0x42, 0x02, 0x39, 0xb9,   // - hito
	0xed, 0x11, 0x0a, 0x07, 0x17, 0x4b, 0xc4, 0x5c); // - hito
	//0x02, 0x00, 0x12, 0xac, 0x42, 0x02, 0x39, 0xb9, 
	//0xed, 0x11, 0x0a, 0x07, 0x17, 0x4b, 0xc4, 0x5c);
	//0x9e, 0xca, 0xdc, 0x24, 0x03, 0xe5, 0xa9, 0xe0, 
	//0x93, 0xf3, 0xa3, 0xb5, 0x02, 0x00, 0x40, 0x6e);
	
// 0000FFFD-0000-1000-8000-00805F9B34FB - fido service
static struct bt_uuid_128 fido_service_uuid = BT_UUID_INIT_128(
	0xFB, 0x34, 0x9B, 0x5F, 0x80, 0x00, 0x00, 0x80,
	0x00, 0x10, 0x00, 0x00, 0xFD, 0xFF, 0x00, 0x00);

// Fido control point
// UUID: 0000F1D0-0000-1000-8000-00805F9B34FB
// Properties:
// - Write
// - Write Without Response   (important: support this)
// Permissions:
// - Write
static struct bt_uuid_128 fido_control_point_char_uuid = BT_UUID_INIT_128(
	0xFB, 0x34, 0x9B, 0x5F, 0x80, 0x00, 0x00, 0x80,
	0x00, 0x10, 0x00, 0x00, 0xD0, 0xF1, 0x00, 0x00);

// Fido Status
// UUID: 0000F1D1-0000-1000-8000-00805F9B34FB
// Properties:
// - Notify
// Permissions:
// - Read (optional)
// - Notify (mandatory)
static struct bt_uuid_128 fido_status_char_uuid = BT_UUID_INIT_128(
	0xFB, 0x34, 0x9B, 0x5F, 0x80, 0x00, 0x00, 0x80,
	0x00, 0x10, 0x00, 0x00, 0xD1, 0xF1, 0x00, 0x00);

#define DEVICE_NAME CONFIG_BT_DEVICE_NAME
#define DEVICE_NAME_LEN (sizeof(DEVICE_NAME) - 1)
#define ADV_LEN 2

static struct bt_gatt_notify_params gatt_notify_params;

static uint8_t m_hito_ble_data[HITO_BLE_MAX_PACKET_LEN];
uint16_t m_hito_ble_data_len;

uint32_t m_hito_ble_data_package_len;
uint32_t m_hito_ble_data_package_progress;
uint8_t * m_hito_ble_data_package = NULL;

bool notify_in_progress = false;

void bt_notify_callback(struct bt_conn * conn, void * user_data) {
	//LOG_INF("Notify callback");
	notify_in_progress = false;
}

/* Advertising data */
static uint8_t manuf_data[ADV_LEN] = {
	0x22 /*SKD version */,
	0x0e /* STM32WB - P2P Server 1 */,
	//0x00 /* GROUP A Feature  */,
	//0x00 /* GROUP A Feature */,
	//0x00 /* GROUP B Feature */,
	//0x00 /* GROUP B Feature */,
	//0x00, /* BLE MAC start -MSB */
	//0x00,
	//0x00,
	//0x00,
	//0x00,
	//0x00, /* BLE MAC stop */
};

static const struct bt_data ad[] = {
	BT_DATA_BYTES(BT_DATA_FLAGS, (BT_LE_AD_GENERAL | BT_LE_AD_NO_BREDR)),
	//BT_DATA(BT_DATA_NAME_COMPLETE, DEVICE_NAME, DEVICE_NAME_LEN),
	//BT_DATA(BT_DATA_MANUFACTURER_DATA, manuf_data, ADV_LEN),
};

static const struct bt_data sd[] = {
	BT_DATA(BT_DATA_NAME_COMPLETE, DEVICE_NAME, DEVICE_NAME_LEN),
	BT_DATA(BT_DATA_MANUFACTURER_DATA, manuf_data, ADV_LEN),
};


/* BLE connection */
struct bt_conn *conn;
/* Notification state */
volatile bool notify_enable;

static void mpu_ccc_cfg_changed(const struct bt_gatt_attr *attr, uint16_t value)
{
	ARG_UNUSED(attr);
	notify_enable = (value == BT_GATT_CCC_NOTIFY);
	LOG_INF("Notification %s", notify_enable ? "enabled" : "disabled");
}

/* The embedded board is acting as GATT server.
 * The ST BLE Android app is the BLE GATT client.
 */

/* ST BLE Sensor GATT services and characteristic */

BT_GATT_SERVICE_DEFINE(stsensor_svc,
BT_GATT_PRIMARY_SERVICE(&st_service_uuid),
BT_GATT_CHARACTERISTIC(&led_char_uuid.uuid,
		       BT_GATT_CHRC_READ | BT_GATT_CHRC_WRITE_WITHOUT_RESP | BT_GATT_CHRC_NOTIFY,
  	       BT_GATT_PERM_WRITE, NULL, hito_ble_recv, (void *)1),
BT_GATT_CCC(mpu_ccc_cfg_changed, BT_GATT_PERM_READ | BT_GATT_PERM_WRITE),
);

bool m_hito_ble_has_error = false;

//-----------------------------------------------------------------------------
//
static ssize_t hito_ble_recv(struct bt_conn *conn,
		    const struct bt_gatt_attr *attr, const void *buf,
		    uint16_t len, uint16_t offset, uint8_t flags)
{
	//LOG_INF("recv %d %d", len, offset);
	//LOG_HEXDUMP_INF(buf, 5, "data");
	//printk(".");
	//led_update();
	if (len > HITO_BLE_MAX_PACKET_LEN) {
		LOG_ERR("data received length is too big - %d, max: %d", 
				len, HITO_BLE_MAX_PACKET_LEN);
		return 0;
	}

	if (m_hito_ble_data_len > 0) {
		LOG_ERR("data buffer is not clear");
		return 0;
	}

	memcpy(m_hito_ble_data, (const char *)buf + offset, len);
	m_hito_ble_data_len = len;

	// Data package process
	if (m_hito_ble_data[0] == 'i') {

		uint32_t data_size;
		memcpy(&data_size, &m_hito_ble_data[1], 4);
		data_size = sys_le32_to_cpu(data_size);
		LOG_DBG("received data package header, size: %u", data_size);

		if (m_hito_ble_data_package != NULL) {
			LOG_ERR("data package buffer is not empty");
			m_hito_ble_has_error = true;
			hito_ble_send("err", 3);
			return 0;
		}

		m_hito_ble_data_package = malloc(data_size);
		if (m_hito_ble_data_package == NULL) {
			LOG_ERR("Memory allocation error");
			m_hito_ble_has_error = true;
			hito_ble_send("err", 3);
			return 0;
		}

		m_hito_ble_data_package_len = data_size;
		m_hito_ble_data_package_progress = 0;
		hito_ble_data_clear();
		hito_ble_send("ok", 2);
		
	} else if (m_hito_ble_data[0] == 'd') {

		uint32_t data_size = len - 1;
		LOG_DBG("received data chunk, size: %u, progress %u/%u", data_size,
				m_hito_ble_data_package_progress, m_hito_ble_data_package_len);

		if (m_hito_ble_data_package == NULL) {
			LOG_ERR("Data package info has not provided yet");
			m_hito_ble_has_error = true;
			hito_ble_send("err", 3);
			return 0;
		}

		if (data_size + m_hito_ble_data_package_progress > m_hito_ble_data_package_len) {
			LOG_ERR("Data chunk size is bigger than expected");
			m_hito_ble_has_error = true;
			hito_ble_send("err", 3);
			return 0;
		}

		memcpy(&m_hito_ble_data_package[m_hito_ble_data_package_progress], &m_hito_ble_data[1], data_size);

    m_hito_ble_data_package_progress += data_size;
    
		hito_ble_data_clear();
		hito_ble_send("ok", 2);

	}

	return 0;
}


const void * hito_ble_data() 
{
	return m_hito_ble_data;
}

const void * hito_ble_data_package() 
{
	return m_hito_ble_data_package;
}

uint16_t hito_ble_datalen()
{
	return m_hito_ble_data_len;
}

uint16_t hito_ble_data_package_len()
{
	return m_hito_ble_data_package_len;
}

//-----------------------------------------------------------------------------
// check if bluetooth has data received
bool hito_ble_has_data() 
{
	return m_hito_ble_data_len != 0 ? true : false;
}

bool hito_ble_has_data_package() 
{
	return m_hito_ble_data_package_len != 0 && 
		m_hito_ble_data_package_len == m_hito_ble_data_package_progress
		? true : false;
}

void hito_ble_data_clear() 
{
	m_hito_ble_data_len = 0;
}

void hito_ble_data_package_clear() 
{
	m_hito_ble_data_package_len = 0;
	m_hito_ble_data_package_progress = 0;
	if (m_hito_ble_data_package != NULL) {
	  free(m_hito_ble_data_package);
	  m_hito_ble_data_package = NULL;
	}
}

//-----------------------------------------------------------------------------
// returns true if peer connected
bool hito_ble_is_connected()
{
	return conn != NULL ? true : false;
}

bool hito_ble_has_error()
{
	return m_hito_ble_has_error;
}

void hito_ble_error_clear()
{
	m_hito_ble_has_error = false;
}


//err = bt_conn_le_phy_update(default_conn, phy);
//-----------------------------------------------------------------------------
// send data to a connected peer
bool hito_ble_send(const void * data, uint32_t len)
{
	if (notify_in_progress) {
		LOG_ERR("bluetooth notify already in progress");
    return false;
	}

	if (!conn) {
		LOG_ERR("bluetooth is not connected");
    return false;
	}

	if (!notify_enable) {
		LOG_ERR("bluetooth notify is not enabled");
    return false;
	}

	notify_in_progress = true;

	gatt_notify_params.uuid = &led_char_uuid.uuid;
	gatt_notify_params.attr = &stsensor_svc.attrs[1];
	gatt_notify_params.data = data;
	gatt_notify_params.len  = len;
	gatt_notify_params.func = bt_notify_callback;
	gatt_notify_params.user_data = data;
	int err =  bt_gatt_notify_cb(conn, &gatt_notify_params);
	if (err) {
		LOG_ERR("Bluetooth notify error: %d", err);
		notify_in_progress = false;
	} else {
		LOG_DBG("Bluetooth send notify ok");
		//but_val = (but_val == 0) ? 0x100 : 0;
		notify_in_progress = false;
	}

	return true;
}

bool m_ble_is_inited = false;
bool m_ble_is_started = false;

//static const struct bt_data sd[] = {
    //BT_DATA(BT_DATA_NAME_COMPLETE, "MyDevice", 8),
//};


//-----------------------------------------------------------------------------
// Bluetooth ready handler
static void hito_ble_bt_ready(int err)
{
	LOG_DBG("hito_ble_bt_ready");
  if (m_ble_is_started) {
    LOG_DBG("Bluetooth has already started");
    return;
  }
	if (err) {
		LOG_ERR("Bluetooth init failed (err %d)", err);
		m_hito_ble_has_error = true;
		return;
	}
	LOG_DBG("Bluetooth initialized");

	/* Start advertising */
	err = bt_le_adv_start(BT_LE_ADV_CONN, ad, ARRAY_SIZE(ad), sd, ARRAY_SIZE(sd));
	if (err) {
		LOG_ERR("Advertising failed to start (err %d)", err);
		m_hito_ble_has_error = true;
		return;
	}

	LOG_DBG("Configuration mode: waiting connections...");
  m_ble_is_inited = true;
  m_ble_is_started = true;

	m_hito_ble_data_len = 0;
	m_hito_ble_data_package_len = 0;
	m_hito_ble_has_error = false;

}

struct bt_conn_le_phy_param phy;
struct bt_conn_le_data_len_param le_conn_data_len;
struct bt_le_conn_param le_param;

//-----------------------------------------------------------------------------
// Bluetooth device connected handler
static void hito_ble_connected(struct bt_conn *connected, uint8_t err)
{
	//LOG_INF("222Connected");
	notify_in_progress = false;
	if (err) {
		LOG_ERR("Connection failed (err %u)", err);
		m_hito_ble_has_error = true;
	} else {
		LOG_DBG("Connected");
		hito_ble_data_package_clear();
		hito_ble_data_clear();
		if (!conn) {
			conn = bt_conn_ref(connected);

			phy.options     = BT_CONN_LE_PHY_OPT_NONE;
      phy.pref_rx_phy = BT_GAP_LE_PHY_2M;   
      phy.pref_tx_phy = BT_GAP_LE_PHY_2M; 
			
			le_conn_data_len.tx_max_len = 247;
			le_conn_data_len.tx_max_time = 247;

			le_param.interval_min = 0x140;
			le_param.interval_max = 0x140;
			le_param.latency = 0;
			le_param.timeout = 1;

	 	  bt_conn_le_phy_update(conn, &phy);
			bt_conn_le_data_len_update(conn, &le_conn_data_len);
			bt_conn_le_param_update(conn, &le_param);
		}
	}
}

//-----------------------------------------------------------------------------
// Bluetooth device disconnected callback
static void hito_ble_disconnected(struct bt_conn *disconn, uint8_t reason)
{
	if (conn) {
		bt_conn_unref(conn);
		conn = NULL;
	}
	hito_ble_data_package_clear();

	LOG_DBG("Disconnected (reason %u)", (int)reason);
}

static const char *phy2str(uint8_t phy)
{
	switch (phy) {
	case 0: return "No packets";
	case BT_GAP_LE_PHY_1M: return "LE 1M";
	case BT_GAP_LE_PHY_2M: return "LE 2M";
	case BT_GAP_LE_PHY_CODED: return "LE Coded";
	default: return "Unknown";
	}
}

static bool le_param_req(struct bt_conn *conn, struct bt_le_conn_param *param)
{
	printk("Connection parameters update request received.\n");
	printk("Minimum interval: %d, Maximum interval: %d\n",
	       param->interval_min, param->interval_max);
	printk("Latency: %d, Timeout: %d\n", param->latency, param->timeout);

	return true;
}

static void le_param_updated(struct bt_conn *conn, uint16_t interval,
			     uint16_t latency, uint16_t timeout)
{
	printk("Connection parameters updated.\n"
	       " interval: %d, latency: %d, timeout: %d\n",
	       interval, latency, timeout);

	//k_sem_give(&throughput_sem);
}

static void le_phy_updated(struct bt_conn *conn,
			   struct bt_conn_le_phy_info *param)
{
	printk("LE PHY updated: TX PHY %s, RX PHY %s\n",
	       phy2str(param->tx_phy), phy2str(param->rx_phy));

	//k_sem_give(&throughput_sem);
}

static void le_data_length_updated(struct bt_conn *conn,
				   struct bt_conn_le_data_len_info *info)
{
	//if (!data_length_req) {
  //		return;
  //	}

	printk("LE data len updated: TX (len: %d time: %d)"
	       " RX (len: %d time: %d)\n", info->tx_max_len,
	       info->tx_max_time, info->rx_max_len, info->rx_max_time);

	//data_length_req = false;
	//k_sem_give(&throughput_sem);
}

//-----------------------------------------------------------------------------
// Bluetooth connection callbacks
static struct bt_conn_cb hito_ble_conn_callbacks = {
	.connected      = hito_ble_connected,
	.disconnected   = hito_ble_disconnected,
	.le_param_req = le_param_req,
	.le_param_updated = le_param_updated,
	.le_phy_updated = le_phy_updated,
	.le_data_len_updated = le_data_length_updated
};

//-----------------------------------------------------------------------------
// Init Bluetooth
bool hito_ble_init() 
{
	m_hito_ble_has_error = false;

  if (m_ble_is_inited) {
    hito_ble_start();
    return true;
  }

  #if defined(NRF5340_XXAA_APPLICATION)
  	nrf_reset_network_force_off(NRF_RESET, true);
  	nrf_reset_network_force_off(NRF_RESET, false);
		LOG_DBG("reset network core");
  #endif
	k_sleep(K_MSEC(100));

	m_hito_ble_data_len = 0;
	notify_in_progress = false;
	conn = NULL;

	bt_conn_cb_register(&hito_ble_conn_callbacks);

	/* Initialize the Bluetooth Subsystem */
	//int err = bt_enable(hito_ble_bt_ready);
	int err = bt_enable(NULL);
	if (err) {
		LOG_ERR("Bluetooth init failed (err %d)", err);
		return false;
	}
	LOG_INF("Bluetooth enabed...");

	return true;
}

void hito_ble_start() 
{
	hito_ble_bt_ready(0);
}

void hito_ble_disconnect() 
{
  if (conn != NULL) {
    int err = bt_conn_disconnect(conn, BT_HCI_ERR_REMOTE_USER_TERM_CONN);
	  if (err) {
		  LOG_ERR("Disconnection failed (err %d)", err);
	  } else {
      LOG_INF("Bluetooth disconnected");
    }
  }
}

void hito_ble_stop() 
{
  if (!m_ble_is_started) {
    LOG_DBG("Bluetooth has already stopped");
    return;
  }

	hito_ble_data_package_clear();

  hito_ble_disconnect();
	bt_le_adv_stop();
  m_ble_is_started   = false;
	notify_in_progress = false;
  LOG_INF("Bluetooth stopped");
}

// eof !
