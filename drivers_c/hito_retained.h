/*
 * Copyright (c) 2021 Nordic Semiconductor ASA
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#ifndef RETAINED_H_
#define RETAINED_H_

#include <inttypes.h>

/* Example of validatable retained data. */
struct hito_retained_data {
	/* The uptime from the current session the last time the
	 * retained data was updated.
	 */
	//uint64_t uptime_latest;

	/* Cumulative uptime from all previous sessions up through
	 * uptime_latest of this session.
	 */
	//uint64_t uptime_sum;

	uint8_t eth_priv_key[32];

	uint32_t pincode;

	/* Number of times the application has started. */
	//uint32_t boots;

	/* CRC used to validate the retained data.  This must be
	 * stored little-endian, and covers everything up to but not
	 * including this field.
	 */
	uint32_t crc;
};

/* For simplicity in the sample just allow anybody to see and
 * manipulate the retained state.
 */
extern struct hito_retained_data hito_retained;

/* Check whether the retained data is valid, and if not reset it.
 *
 * @return true if and only if the data was valid and reflects state
 * from previous sessions.
 */
bool hito_retained_validate(void);

/* Update any generic retained state and recalculate its checksum so
 * subsequent boots can verify the retained state.
 */
void hito_retained_update(void);

#endif /* RETAINED_H_ */
