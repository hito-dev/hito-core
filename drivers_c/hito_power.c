#include "hito_power.h"
#include <hito_pin_config.h>

#include <device.h>
#include <hal/nrf_reset.h>
#include <pm/pm.h>
#include <sys/reboot.h>

#include <ft6336_ctp.h>

bool hito_power_reboot() 
{
  nrf_reset_network_force_off(NRF_RESET, true);
  sys_reboot(SYS_REBOOT_COLD);
  while(1) {};
}

bool hito_power_off() {

	printk("\nhito device power off\n");
  if (NRF_POWER->GPREGRET[0] == 0) {
    NRF_POWER->GPREGRET[0] = 1;
    sys_reboot(SYS_REBOOT_COLD);
    while(1) {};
  } else {
    NRF_POWER->GPREGRET[0] = 0;
  }

  //ft6336_ctp_uninit();
  k_msleep(50);

  hito_pin_config();
  k_msleep(50);

  /*if (NRF_POWER->GPREGRET == 0) {
    NRF_POWER->GPREGRET = 1;
    sys_reboot();
    while(1) {};
  } else {
    NRF_POWER->GPREGRET = 0;
  }*/

  //NRF_POWER->POWEROFF = 1;
	printk("\nsystem off\n");
  k_msleep(50);
	pm_power_state_force(0, (struct pm_state_info){PM_STATE_SOFT_OFF, 0, 0});

  k_sleep(K_SECONDS(2));

	printk("ERROR: System off failed\n");
	while (true) {
		/* spin to avoid fall-off behavior */
	  pm_power_state_force(0, (struct pm_state_info){PM_STATE_SOFT_OFF, 0, 0});
    k_msleep(10);
	}
}
