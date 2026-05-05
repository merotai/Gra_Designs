

#ifndef _COMAT_H
#define _COMAT_H


#include <stdint.h>
#include <driver/uart.h>
#include "freertos/FreeRTOS.h"
#include "esp_log.h"


/// 该内容适用于AT命令控制的通信模块

typedef struct 
{
    int uart_port;              // UART端口号，例如UART_NUM_1
    int tx_pin;
    int rx_pin;
    int baudrate;
    int rx_buffer_size;
    int tx_buffer_size;
    const char *remote_host;     // 远程服务器IP地址
    uint16_t remote_port;        // 远程服务器端口号
} comat_uart_config_t;




#endif