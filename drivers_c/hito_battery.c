#include <stdbool.h>

#include <zephyr.h>
#include <drivers/adc.h>
#include <hal/nrf_saadc.h>

#if !DT_NODE_EXISTS(DT_PATH(zephyr_user)) || \
  !DT_NODE_HAS_PROP(DT_PATH(zephyr_user), io_channels)
#error "No suitable devicetree overlay specified"
#endif

#define ADC_NUM_CHANNELS  DT_PROP_LEN(DT_PATH(zephyr_user), io_channels)

#if ADC_NUM_CHANNELS > 2
#error "Currently only 1 or 2 channels supported in this sample"
#endif

#if ADC_NUM_CHANNELS == 2 && !DT_SAME_NODE( \
  DT_PHANDLE_BY_IDX(DT_PATH(zephyr_user), io_channels, 0), \
  DT_PHANDLE_BY_IDX(DT_PATH(zephyr_user), io_channels, 1))
#error "Channels have to use the same ADC."
#endif

#define ADC_NODE    DT_PHANDLE(DT_PATH(zephyr_user), io_channels)

/* Common settings supported by most ADCs */
#define ADC_RESOLUTION    12
#define ADC_GAIN    ADC_GAIN_1_5
#define ADC_REFERENCE   ADC_REF_INTERNAL
#define ADC_ACQUISITION_TIME  ADC_ACQ_TIME_DEFAULT

/* Get the numbers of up to two channels */
static uint8_t channel_ids[ADC_NUM_CHANNELS] = {
  DT_IO_CHANNELS_INPUT_BY_IDX(DT_PATH(zephyr_user), 0),
#if ADC_NUM_CHANNELS == 2
  DT_IO_CHANNELS_INPUT_BY_IDX(DT_PATH(zephyr_user), 1)
#endif
};

static int16_t sample_buffer[ADC_NUM_CHANNELS];

struct adc_channel_cfg channel_cfg = {
  .gain = ADC_GAIN,
  .reference = ADC_REFERENCE,
  .acquisition_time = ADC_ACQUISITION_TIME,
  /* channel ID will be overwritten below */
  .channel_id = 0,
  .differential = 0
};

struct adc_sequence sequence = {
  /* individual channels will be added below */
  .channels    = 0,
  .buffer      = sample_buffer,
  /* buffer size in bytes, not number of samples */
  .buffer_size = sizeof(sample_buffer),
  .resolution  = ADC_RESOLUTION,
};

/* Common settings supported by most ADCs */
#define ADC_RESOLUTION    12
#define ADC_GAIN    ADC_GAIN_1_5
#define ADC_REFERENCE   ADC_REF_INTERNAL
#define ADC_ACQUISITION_TIME  ADC_ACQ_TIME_DEFAULT

const struct device *dev_adc = NULL;

void hito_battery_level_init() {
  if (dev_adc != NULL) {
    return;
  }

  dev_adc = DEVICE_DT_GET(ADC_NODE);
 
  if (!device_is_ready(dev_adc)) {
    printk("ADC device not found\n");
    return;
  }
 
  printk("Num channels: %d\n", ADC_NUM_CHANNELS);
 
  /*
   * Configure channels individually prior to sampling
   */
  for (uint8_t i = 0; i < ADC_NUM_CHANNELS; i++) {
    channel_cfg.channel_id = channel_ids[i];
    channel_cfg.input_positive = NRF_SAADC_INPUT_VDDHDIV5;
    //channel_cfg.input_positive = NRF_SAADC_INPUT_VDD;
    //channel_cfg.input_positive = NRF_SAADC_INPUT_AIN0
               //+ channel_ids[i];
 
    adc_channel_setup(dev_adc, &channel_cfg);
 
    sequence.channels |= BIT(channel_ids[i]);
  }
}

static int32_t mv_value;

int32_t hito_battery_level_mv() {
  return mv_value;
}

// returns hito battery level
int hito_battery_level() {
  hito_battery_level_init();

  int32_t adc_vref = adc_ref_internal(dev_adc);
  adc_vref = adc_ref_internal(dev_adc);

  int err = adc_read(dev_adc, &sequence);                                         
  if (err != 0) {                                                             
    printk("ADC reading failed with error %d.\n", err);                       
    return 0;                                                                   
  }                                                                           

  int32_t raw_value = sample_buffer[0]; 


  if (adc_vref > 0) {
     /*
     * Convert raw reading to millivolts if driver
     * supports reading of ADC reference voltage
     */
    mv_value = raw_value;

    adc_raw_to_millivolts(adc_vref, ADC_GAIN, ADC_RESOLUTION, &mv_value);

    mv_value *= 5;

    if (mv_value < 3010) {
      return 1;
    } 
    if (mv_value > 3620) {
      return 100;
    }

    double percentage;
    double voltage_V = mv_value;
    voltage_V /= 1000.0;
    percentage = - 428146.6039
                 + 528403.8094 * voltage_V
                 - 244052.3017 * voltage_V * voltage_V
                 + 49982.84633 * voltage_V * voltage_V * voltage_V
                 - 3828.794657 * voltage_V * voltage_V * voltage_V * voltage_V;
    int res = (int)percentage;
    if (res < 0) {
      res = 0;
    } else if (res > 100) {
      res = 100;
    }
    return res;

  } else {
    return 0;
  }
}
