#include "hito_nfc.h"

#include <nfc_t4t_lib.h>

#include <nfc/ndef/msg.h>
#include <nfc/t4t/ndef_file.h>
#include <nfc/ndef/uri_rec.h>
#include <nfc/ndef/uri_msg.h>

#include <logging/log.h>
LOG_MODULE_REGISTER(hito_nfc, LOG_LEVEL_DBG);

static uint8_t m_ndef_data_buf[CONFIG_NDEF_FILE_SIZE]; /**< Buffer for NDEF file. */

static uint8_t m_nfc_data[CONFIG_NDEF_FILE_SIZE];

static uint32_t m_nfc_data_len;
/**
 * @brief Callback function for handling NFC events.
 */
static void hito_nfc_callback(void *context,
			 nfc_t4t_event_t event,
			 const uint8_t *data,
			 size_t data_length,
			 uint32_t flags)
{
	ARG_UNUSED(context);
	ARG_UNUSED(data);
	ARG_UNUSED(flags);

	switch (event) {
	case NFC_T4T_EVENT_FIELD_ON:
		//dk_set_led_on(NFC_FIELD_LED);
		break;

	case NFC_T4T_EVENT_FIELD_OFF:
		//dk_set_leds(DK_NO_LEDS_MSK);
		break;

	case NFC_T4T_EVENT_NDEF_READ:
		//dk_set_led_on(NFC_READ_LED);
		break;

	case NFC_T4T_EVENT_NDEF_UPDATED:
		if (data_length > 0) {
			//dk_set_led_on(NFC_WRITE_LED);
			//m_data_length = data_length;
			//flash_buffer_prepare(data_length);
			memcpy(m_nfc_data, m_ndef_data_buf, data_length + NFC_NDEF_FILE_NLEN_FIELD_SIZE);
			m_nfc_data_len = data_length + NFC_NDEF_FILE_NLEN_FIELD_SIZE;
		}
		break;

	default:
		break;
	}
}

void hito_nfc_stop() 
{
	nfc_t4t_emulation_stop();
	nfc_t4t_done();
}

/** init nfc driver, returns true on success */
bool hito_nfc_start(const char * message) 
{
	memset(m_ndef_data_buf, 0, CONFIG_NDEF_FILE_SIZE);
	m_nfc_data_len = 0;

	/* Set up NFC */
	int err = nfc_t4t_setup(hito_nfc_callback, NULL);

	if (err < 0) {
		LOG_DBG("Cannot setup t4t library!\n");
		return false;
	}
	/* Run Read-Write mode for Type 4 Tag platform */
	if (nfc_t4t_ndef_rwpayload_set(m_ndef_data_buf, sizeof(m_ndef_data_buf)) < 0) 
  {
		LOG_DBG("Cannot set payload!\n");
		return false;
	}
	/* Start sensing NFC field */
	if (nfc_t4t_emulation_start() < 0) {
		LOG_DBG("Cannot start emulation!\n");
		return false;
	}

	hito_nfc_set_message(message);
	return true;
}

int hito_nfc_set_message(const char * msg)
{
	int err;
	uint32_t size = CONFIG_NDEF_FILE_SIZE;
	uint32_t ndef_size = nfc_t4t_ndef_file_msg_size_get(size);

	/* Encode URI message into buffer. */
	err = nfc_ndef_uri_msg_encode(NFC_URI_HTTPS,
				      msg,
				      strlen(msg),
				      nfc_t4t_ndef_file_msg_get(m_ndef_data_buf),
				      &ndef_size);
	if (err) {
		LOG_ERR("nfc_ndef_uri_msg_encode %d", err);
		return err;
	}

	err = nfc_t4t_ndef_file_encode(m_ndef_data_buf, &ndef_size);
	if (err) {
		LOG_ERR("nfc_t4t_ndef_file_encode %d", err);
		return err;
	}

	//*size = ndef_size;

	return 0;
}

/** returns true if has new data received */
bool hito_nfc_has_data() 
{
  return m_nfc_data_len != 0;
}

/** return received nfc data length */
uint32_t hito_nfc_data_len()
{
	if (m_nfc_data[5] == 0x54) {
		return m_nfc_data_len - 6;
	} else {
		return m_nfc_data_len - 9;
	}
}

/** returns nfc data received */
const uint8_t * hito_nfc_data()
{
	if (m_nfc_data[5] == 0x54) {
		return &m_nfc_data[6];
	} else {
		return &m_nfc_data[9];
	}
}

/** clears data in nfc buffer */
void hito_nfc_data_clear() 
{
  m_nfc_data_len = 0;
}


