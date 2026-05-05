#ifndef _COMQTT_H
#define _COMQTT_H

#include <stdint.h>
#include "bmi270/bmi270_template.h"
#include <driver/uart.h>
#include <string.h>

#include "freertos/FreeRTOS.h"
#include "esp_log.h"
#include "esp_timer.h"

#define UART_NUM_1 1
#define UART_TX_PIN 16
#define UART_RX_PIN 17
#define UART_BUFFER_SIZE 512
#define UART_BAUDRATE 115200

/// MQTT协议payload结构定义
#pragma pack(push, 1)               // 设置结构体为1字节对齐，确保数据帧的紧凑性
typedef struct {                    // 总计 241 bytes
    uint8_t imu_num;
    bmi270_sample_t imu_samples[10]; // 假设每帧包含10个样本
} comqtt_payload_t;
#pragma pack(pop)                   // 恢复默认对齐方式




typedef struct {
    int uart_port;              // UART端口号，例如UART_NUM_1
    int tx_pin;
    int rx_pin;
    int baudrate;
    int rx_buffer_size;
    int tx_buffer_size;
    
} comqtt_uart_config_t;

// ---------------------------------------------------------------------------------------- //

comqtt_payload_t*  payload_init();

bool comqtt_uart_init(const comqtt_uart_config_t* config, QueueHandle_t* event_queue);

bool comqtt_uart_send_payload(const comqtt_payload_t *payload);
#endif // _COMQTT_H