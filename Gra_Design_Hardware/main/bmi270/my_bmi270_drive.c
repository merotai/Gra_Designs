

#include "stdio.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"


#include "bmi270_api.h"  // 报错疑点
#include "bmi2.h"        // 报错疑点
#include "my_bmi270_drive.h"


static const char *TAG_BMI270 = "BMI270";


#define MD_I2C_Master_Num  I2C_NUM_0
#define MD_I2C_Master_SDA_IO 20
#define MD_I2C_Master_SCL_IO 21
#define MD_I2C_Master_FREQ_HZ 400000
#define MD_BMI270_ADDR 0x68


/// i2c读函数
int8_t bmi270_i2c_read(uint8_t reg_addr,uint8_t *reg_data, uint32_t len, void *intf_ptr) {
    i2c_port_t i2c_num = *(i2c_port_t *)intf_ptr;

    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (MD_BMI270_ADDR << 1) | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, reg_addr, true);
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (MD_BMI270_ADDR << 1) | I2C_MASTER_READ, true);
    i2c_master_read(cmd, reg_data, len, I2C_MASTER_LAST_NACK);
    i2c_master_stop(cmd);
    esp_err_t ret = i2c_master_cmd_begin(i2c_num, cmd, pdMS_TO_TICKS(100));

    i2c_cmd_link_delete(cmd);

    return (ret == ESP_OK) ? BMI2_OK : BMI2_E_COM_FAIL;
}

/// i2c写函数
int8_t bmi270_i2c_write(uint8_t reg_addr, const uint8_t *reg_data, uint32_t len, void *intf_ptr) {
    i2c_port_t i2c_num = *(i2c_port_t *)intf_ptr;

    i2c_cmd_handle_t cmd = i2c_cmd_link_create();
    i2c_master_start(cmd);
    i2c_master_write_byte(cmd, (MD_BMI270_ADDR << 1) | I2C_MASTER_WRITE, true);
    i2c_master_write_byte(cmd, reg_addr, true);
    i2c_master_write(cmd, reg_data, len, true);
    i2c_master_stop(cmd);
    esp_err_t ret = i2c_master_cmd_begin(i2c_num, cmd, pdMS_TO_TICKS(100));

    i2c_cmd_link_delete(cmd);

    return (ret == ESP_OK) ? BMI2_OK : BMI2_E_COM_FAIL;
}

void bmi270_delay_us(uint32_t period, void *intf_ptr) {
    vTaskDelay(pdMS_TO_TICKS((period + 999) / 1000)); // 将微秒转换为毫秒，并进行适当的延迟
}

void bmi270_master_init(void) {
    i2c_config_t i2c_config = {
        .mode = I2C_MODE_MASTER,
        .sda_io_num = MD_I2C_Master_SDA_IO,
        .scl_io_num = MD_I2C_Master_SCL_IO,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = MD_I2C_Master_FREQ_HZ,
    };
    ESP_ERROR_CHECK(i2c_param_config(MD_I2C_Master_Num, &i2c_config));
    ESP_ERROR_CHECK(i2c_driver_install(MD_I2C_Master_Num, i2c_config.mode, 0, 0, 0));
    ESP_LOGI(TAG_BMI270, "I2C initialized successfully");
}