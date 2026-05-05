/*
 * SPDX-FileCopyrightText: 2010-2022 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: CC0-1.0
 */



#include "interact_task.h"



void app_main(void) {
    
    xTaskCreate(init_task, "INIT_TASK", 4096, NULL, 4, NULL);
    vTaskDelay(pdMS_TO_TICKS(1000)); // 等待初始化任务完成，实际项目中可通过事件组或信号量来更精确地同步任务启动顺序
    xTaskCreate(bmi270_task_1, "BMI270_TASK_1", 4096, NULL, 3, NULL);
    xTaskCreate(bmi270_task_2, "BMI270_TASK_2", 4096, NULL, 3, NULL);
    xTaskCreate(send_task, "SEND_TASK", 4096, NULL, 2, NULL);

}
























// while(1) {
    //     if (coml_connect_test_1()) {
    //         ESP_LOGI("COML", "Connection test 1 passed");
    //     } else {
    //         ESP_LOGE("COML", "Connection test 1 failed");
    //     }
    //     vTaskDelay(pdMS_TO_TICKS(5000)); // 每5秒执行一次连接测试
    // }

// void uart_loopback_test() {
//     // GPIO16 和 GPIO17 用杜邦线短接在一起
    
//     uint8_t test_data[] = "TEST123\r\n";
//     int sent = uart_write_bytes(UART_NUM_1, test_data, strlen((char*)test_data));
//     ESP_LOGI("LOOPBACK", "Sent %d bytes", sent);
    
//     vTaskDelay(pdMS_TO_TICKS(200));  // 等待数据返回
    
//     uint8_t rx_buffer[128];
//     int rx_len = uart_read_bytes(UART_NUM_1, rx_buffer, sizeof(rx_buffer)-1, pdMS_TO_TICKS(1000));
    
//     if (rx_len > 0) {
//         rx_buffer[rx_len] = '\0';
//         ESP_LOGI("LOOPBACK", "SUCCESS! Received %d bytes: %s", rx_len, rx_buffer);
//     } else {
//         ESP_LOGE("LOOPBACK", "FAILED! No data received. UART config problem.");
        
//         // 打印 UART 状态
//         esp_err_t err = uart_get_buffered_data_len(UART_NUM_1, (size_t*)&rx_len);
//         ESP_LOGI("LOOPBACK", "UART buffer status: err=%d, len=%d", err, rx_len);
//     }
// }

// void app_main(void) {
//     coml_uart_config_t config = {
//         .uart_port = UART_NUM_1,
//         .tx_pin = 16,
//         .rx_pin = 17,
//         .baudrate = 115200,
//         .rx_buffer_size = 512,
//         .tx_buffer_size = 512,
//         .remote_host = "192.168.81.251",
//         .remote_port = 443
//     };

//     if (coml_uart_init(&config)) {
//         ESP_LOGI("COML", "UART initialized successfully");
//     } else {
//         ESP_LOGE("COML", "Failed to initialize UART");
//         return;
//     }
    
//     uart_loopback_test();  // 先只做回环测试
// }



/*
void app_main(void)
{
    if (bmi270_template_init() != 0) {
        return;
    }

    while (1) {
        bmi270_sample_t s;
        if (bmi270_template_read(&s) == 0) {
            // TODO: replace with your logging or processing

            printf("Accel: X = %.3f g ,Y = %.3f g ,Z = %.3f g\n", s.ax_g, s.ay_g, s.az_g);
            printf("Gyro: X = %.1f dps , Y = %.1f dps , Z = %.1f dps\n", s.gx_dps, s.gy_dps, s.gz_dps);
        }
        vTaskDelay(pdMS_TO_TICKS(100));
    }
}



*/



// #include <stdio.h>

// #include "sdkconfig.h"
// #include "freertos/FreeRTOS.h"
// #include "freertos/task.h"
// #include "esp_chip_info.h"
// #include "esp_flash.h"
// #include "esp_system.h"


// #include "bmi270_api.h"  // 报错疑点
// #include "bmi2.h"        // 报错疑点
// #include "my_bmi270_drive.h"


// static const char *TAG = "BMI270_Main";

// static struct bmi2_dev bmi270_dev;

// void app_main(void) {
//     ESP_LOGI(TAG,"==========BMI270 6-axis Sensor Test==========");

//     bmi270_master_init();

//     int8_t rslt;

//     i2c_port_t i2c_num = I2C_NUM_0;

//     bmi270_dev.intf = BMI2_I2C_INTF;
//     bmi270_dev.read = bmi270_i2c_read;
//     bmi270_dev.write = bmi270_i2c_write;
//     bmi270_dev.delay_us = bmi270_delay_us;
//     bmi270_dev.intf_ptr = &i2c_num;
    
//     // 初始化传感器
//     rslt = bmi270_init(&bmi270_dev);
//     if (rslt != BMI2_OK) {
//         ESP_LOGE(TAG, "Failed to initialize the sensor (code %+d)", rslt);
//         return;
//     }
//     ESP_LOGI(TAG, "Sensor initialized successfully");

//     struct bmi2_sens_config config;
//     config.type = BMI2_ACCEL;
//     config.cfg.acc.odr = BMI2_ACC_ODR_100HZ;
//     config.cfg.acc.range = BMI2_ACC_RANGE_16G;
//     config.cfg.acc.bwp = BMI2_ACC_NORMAL_AVG4;
//     rslt = bmi270_set_sensor_config(&config, 1, &bmi270_dev);
//     if (rslt != BMI2_OK) {
//         ESP_LOGE(TAG, "Failed to configure accelerometer!");
        
//     }
//     config.type = BMI2_GYRO;
//     config.cfg.gyr.odr = BMI2_GYR_ODR_100HZ;
//     config.cfg.gyr.range = BMI2_GYR_RANGE_2000;
//     config.cfg.gyr.bwp = BMI2_GYR_NORMAL_MODE;
//     rslt = bmi270_set_sensor_config(&config, 1, &bmi270_dev);
//     if (rslt != BMI2_OK) {
//         ESP_LOGE(TAG, "Failed to configure gyroscope!");
        
//     }

//     // 主循环
//     while(1) {
//         struct bmi2_sensor_data accel_data = {0};
//         struct bmi2_sensor_data gyro_data = {0};

//         accel_data.type = BMI2_ACCEL;
//         gyro_data.type = BMI2_GYRO;

//         rslt = bmi270_get_sensor_data(&accel_data,&gyro_data, &bmi270_dev);

//         if (rslt == BMI2_OK) {

//             // 加速度计
//             float ax = accel_data.sens_data.acc.x / (float) (1 << 11);
//             float ay = accel_data.sens_data.acc.y / (float) (1 << 11);
//             float az = accel_data.sens_data.acc.z / (float) (1 << 11);
//             // 陀螺仪
//             float gx = gyro_data.sens_data.gyr.x / 16.4f;
//             float gy = gyro_data.sens_data.gyr.y / 16.4f;
//             float gz = gyro_data.sens_data.gyr.z / 16.4f;
//             ESP_LOGI(TAG, "Accel: X = %.3f% g ,Y = %.3f,Z = %.3f", ax, ay, az);
//             ESP_LOGI(TAG, "Gyro: X = %.1f dps , Y = %.1f dps , Z = %.1f dps", gx, gy, gz);

//         } else {
//             ESP_LOGE(TAG, "Failed to read sensor data (code %+d)", rslt);
//         }

//         vTaskDelay(pdMS_TO_TICKS(500)); // 延迟0.5秒



//     }



// }


