#include "bmi270_template.h"

#include "esp_log.h"

#include "bmi270_api.h"
#include "bmi2.h"

#define BMI270_I2C_PORT_1      I2C_NUM_0
#define NMI270_I2C_ADDR        0x68
#define BMI270_I2C_PORT_2      I2C_NUM_1
#define NMI270_I2C_ADDR_2      0x69
#define BMI270_I2C_1_SCL_IO    4
#define BMI270_I2C_1_SDA_IO    5
#define BMI270_I2C_2_SCL_IO    6
#define BMI270_I2C_2_SDA_IO    7
#define BMI270_I2C_FREQ_HZ   400000

static const char *TAG = "BMI270_TPL";
static i2c_bus_handle_t s_i2c_bus = NULL;
static bmi270_handle_t s_bmi_handle = NULL;

static float lsb_to_g(int16_t val, float g_range, uint8_t bit_width)
{
    float half_scale = (float)(1 << (bit_width - 1));
    return (g_range * val) / half_scale;
}

static float lsb_to_dps(int16_t val, float dps_range, uint8_t bit_width)
{
    float half_scale = (float)(1 << (bit_width - 1));
    return (dps_range * val) / half_scale;
}

//
int8_t bmi270_init_1(void)
{
    int8_t rslt;
    const i2c_config_t i2c_bus_conf = {
        .mode = I2C_MODE_MASTER,
        .sda_io_num = BMI270_I2C_1_SDA_IO,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_io_num = BMI270_I2C_1_SCL_IO,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = BMI270_I2C_FREQ_HZ
    };

    s_i2c_bus = i2c_bus_create(BMI270_I2C_PORT_1, &i2c_bus_conf);
    if (s_i2c_bus == NULL) {
        ESP_LOGE(TAG, "i2c_bus_create failed");
        return BMI2_E_COM_FAIL;
    }
    
    if (bmi270_sensor_create(s_i2c_bus, &s_bmi_handle, bmi270_config_file,
                             BMI2_GYRO_CROSS_SENS_ENABLE | BMI2_CRT_RTOSK_ENABLE) != ESP_OK ||
        s_bmi_handle == NULL) {
        ESP_LOGE(TAG, "bmi270_sensor_create failed");
        return BMI2_E_DEV_NOT_FOUND;
    }

    struct bmi2_sens_config config[2];
    
    config[BMI2_ACCEL].type = BMI2_ACCEL;
    config[BMI2_GYRO].type = BMI2_GYRO;

    rslt = bmi2_get_sensor_config(config, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "get_sensor_config failed: %d", rslt);
        return rslt;
    }

    config[BMI2_ACCEL].cfg.acc.odr = BMI2_ACC_ODR_100HZ;
    config[BMI2_ACCEL].cfg.acc.range = BMI2_ACC_RANGE_16G;
    config[BMI2_ACCEL].cfg.acc.bwp = BMI2_ACC_NORMAL_AVG4;
    config[BMI2_ACCEL].cfg.acc.filter_perf = BMI2_PERF_OPT_MODE;

    config[BMI2_GYRO].cfg.gyr.odr = BMI2_GYR_ODR_100HZ;
    config[BMI2_GYRO].cfg.gyr.range = BMI2_GYR_RANGE_2000;
    config[BMI2_GYRO].cfg.gyr.bwp = BMI2_GYR_NORMAL_MODE;
    config[BMI2_GYRO].cfg.gyr.filter_perf = BMI2_PERF_OPT_MODE;
    config[BMI2_GYRO].cfg.gyr.noise_perf = BMI2_PERF_OPT_MODE;

    rslt = bmi2_set_sensor_config(config, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "set_sensor_config failed: %d", rslt);
        return rslt;
    }

    uint8_t sensor_list[2] = { BMI2_ACCEL, BMI2_GYRO };
    rslt = bmi2_sensor_enable(sensor_list, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "sensor_enable failed: %d", rslt);
        return rslt;
    }

    return BMI2_OK;
}


int8_t bmi270_init_2(void){
    int8_t rslt;
    const i2c_config_t i2c_bus_conf = {
        .mode = I2C_MODE_MASTER,
        .sda_io_num = BMI270_I2C_2_SDA_IO,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_io_num = BMI270_I2C_2_SCL_IO,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = BMI270_I2C_FREQ_HZ
    };

    s_i2c_bus = i2c_bus_create(BMI270_I2C_PORT_2, &i2c_bus_conf);
    if (s_i2c_bus == NULL) {
        ESP_LOGE(TAG, "i2c_bus_create failed");
        return BMI2_E_COM_FAIL;
    }
    
    if (bmi270_sensor_create(s_i2c_bus, &s_bmi_handle, bmi270_config_file,
                             BMI2_GYRO_CROSS_SENS_ENABLE | BMI2_CRT_RTOSK_ENABLE) != ESP_OK ||
        s_bmi_handle == NULL) {
        ESP_LOGE(TAG, "bmi270_sensor_create failed");
        return BMI2_E_DEV_NOT_FOUND;
    }

    struct bmi2_sens_config config[2];
    
    config[BMI2_ACCEL].type = BMI2_ACCEL;
    config[BMI2_GYRO].type = BMI2_GYRO;

    rslt = bmi2_get_sensor_config(config, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "get_sensor_config failed: %d", rslt);
        return rslt;
    }

    config[BMI2_ACCEL].cfg.acc.odr = BMI2_ACC_ODR_100HZ;
    config[BMI2_ACCEL].cfg.acc.range = BMI2_ACC_RANGE_16G;
    config[BMI2_ACCEL].cfg.acc.bwp = BMI2_ACC_NORMAL_AVG4;
    config[BMI2_ACCEL].cfg.acc.filter_perf = BMI2_PERF_OPT_MODE;

    config[BMI2_GYRO].cfg.gyr.odr = BMI2_GYR_ODR_100HZ;
    config[BMI2_GYRO].cfg.gyr.range = BMI2_GYR_RANGE_2000;
    config[BMI2_GYRO].cfg.gyr.bwp = BMI2_GYR_NORMAL_MODE;
    config[BMI2_GYRO].cfg.gyr.filter_perf = BMI2_PERF_OPT_MODE;
    config[BMI2_GYRO].cfg.gyr.noise_perf = BMI2_PERF_OPT_MODE;

    rslt = bmi2_set_sensor_config(config, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "set_sensor_config failed: %d", rslt);
        return rslt;
    }

    uint8_t sensor_list[2] = { BMI2_ACCEL, BMI2_GYRO };
    rslt = bmi2_sensor_enable(sensor_list, 2, s_bmi_handle);
    if (rslt != BMI2_OK) {
        ESP_LOGE(TAG, "sensor_enable failed: %d", rslt);
        return rslt;
    }

    return BMI2_OK;
}



//
int8_t bmi270_template_read(bmi270_sample_t *sample)
{
    if (sample == NULL) {
        return BMI2_E_INVALID_SENSOR;
    }

    struct bmi2_sens_data data = {0};
    int8_t rslt = bmi2_get_sensor_data(&data, s_bmi_handle);
    if (rslt != BMI2_OK) {
        return rslt;
    }

    if ((data.status & BMI2_DRDY_ACC) == 0 || (data.status & BMI2_DRDY_GYR) == 0) {
        return BMI2_W_PARTIAL_READ;
    }

    sample->ax_g = lsb_to_g(data.acc.x, BMI2_ACC_RANGE_16G_VAL, s_bmi_handle->resolution);
    sample->ay_g = lsb_to_g(data.acc.y, BMI2_ACC_RANGE_16G_VAL, s_bmi_handle->resolution);
    sample->az_g = lsb_to_g(data.acc.z, BMI2_ACC_RANGE_16G_VAL, s_bmi_handle->resolution);

    sample->gx_dps = lsb_to_dps(data.gyr.x, BMI2_GYR_RANGE_2000_VAL, s_bmi_handle->resolution);
    sample->gy_dps = lsb_to_dps(data.gyr.y, BMI2_GYR_RANGE_2000_VAL, s_bmi_handle->resolution);
    sample->gz_dps = lsb_to_dps(data.gyr.z, BMI2_GYR_RANGE_2000_VAL, s_bmi_handle->resolution);

    return BMI2_OK;
}





