
#ifndef _COML_H
#define _COML_H



#include <stdint.h>
#include <driver/uart.h>
#include <string.h>
#include "bmi270/bmi270_template.h"
#include "freertos/FreeRTOS.h"
#include "esp_log.h"
#include "esp_timer.h"

#define FRAME_CONNECTION_HEADER     0xEC13
#define FRAME_DATA_HEADER           0xEC14
#define FRAME_DATA_TAIL             0xEC15
#define FRAME_HEARTBEAT_HEADER      0xEC16



#define PROTOCOL_VERSION            0x01
#define DEVICE_ID                   0x0001
#define DATA_FRAME_TYPE_CONNECTION  0x0001
#define DATA_FRAME_TYPE_SENSOR_1    0x0002
#define DATA_FRAME_TYPE_SENSOR_2    0x0003
#define DATA_FRAME_TYPE_HEARTBEAT   0x0004

/// 该内容基于DTU固件的通信模块设计

/// 设计的通信协议如下：
/// 三种数据帧类型：
/// 1. 建立连接帧：包含设备ID等信息，用于建立通信连接，同时确认连接状态。
/// 2. 数据帧：包含传感器数据、时间戳等信息，用于传输实际的传感器数据。
/// 3. 心跳帧：定期发送，包含设备ID和帧类型，用于保持连接活跃，监测设备状态。
/// 建立连接后，5ms发送一次数据帧，包含最新的传感器数据和时间戳。
/// 

#pragma pack(push, 1)               // 设置结构体为1字节对齐，确保数据帧的紧凑性
typedef struct 
{
    uint16_t header;                // 帧头，固定值0xEC13           2 bytes
    uint16_t device_id;             // 设备ID                       2 bytes
    uint16_t frame_type;            // 数据帧类型，固定值0xEC13        2 bytes
    uint16_t frame_len;             // 数据帧长度 (不包含帧头和CRC32校验码)，2 bytes
    uint64_t timestamp;             // 时间戳 (单位：毫秒)                 8 bytes
    
    uint8_t protocol_version;       // 协议版本                         1 byte  
    uint8_t order;                  // 连接序号，递增的数字，用于区分不同的连接请求, 1为申请连接，2为断开连接  1 byte
    uint16_t reserved;              // 保留字段，预留给未来使用，当前填充为0 2 bytes
    uint32_t crc32_check;           // CRC32校验码，计算方式为对前面所有字段（从帧头到数据部分）进行CRC32计算得到的结果，
                                    // 用于数据完整性验证 4 bytes
} connection_frame_t;               // 总长度为 2 + 2 + 2 + 2 + 8 + 1 + 1 + 2 + 4 = 24 bytes
#pragma pack(pop)                   // 恢复默认对齐方式



#pragma pack(push, 1)               
typedef struct 
{
    uint16_t header;                // 帧头，固定值0xEC14 2 bytes
    uint16_t device_id;             // 设备ID, 2 bytes
    uint16_t frame_type;            // 数据帧类型，1表示传感器数据帧，2表示心跳帧，2 bytes
    uint16_t frame_len;             // 数据帧长度 (不包含帧头和CRC32校验码)，2 bytes
    uint64_t timestamp;             // 时间戳 (单位：毫秒)，8 bytes
    
    bmi270_sample_t samples[10];    // 传感器数据，包含加速度和陀螺仪,数据长度 float * 6 * 10 = 240 bytes
    uint32_t crc32_check;           // CRC32校验码，计算方式为对前面所有字段（从帧头到数据部分）进行CRC32计算得到的结果，
                                    // 用于数据完整性验证 4 bytes

} data_frame_t;                     // 总长度为 2 + 2 + 2 + 2 + 240 + 8 + 4  = 260 bytes
#pragma pack(pop)                   



#pragma pack(push, 1)
typedef struct 
{
    uint16_t header;                // 帧头，固定值0xEC15                   2 bytes
    uint16_t device_id;             // 设备ID                               2 bytes
    uint16_t frame_type;            // 数据帧类型，固定值                      2 bytes
    uint16_t frame_len;             // 数据帧长度 (不包含帧头和CRC32校验码)      2 bytes
    uint64_t timestamp;             // 时间戳 (单位：毫秒)                      8 bytes

    uint32_t crc32_check;           // CRC32校验码，计算方式为对前面所有字段（从帧头到数据部分）进行CRC32计算得到的结果，4 bytes

} heartbeat_frame_t;                // 总长度为 2 + 2 + 2 + 2 + 8 + 4 = 20 bytes
#pragma pack(pop)


/// 
///
///

#define DTU_UART_NUM            UART_NUM_1      //使用UART1进行通信
#define DTU_UART_TX_PIN         4               //MCU的4号引脚连接ML307R的TXD引脚
#define DTU_UART_RX_PIN         5               //MCU的5号引脚连接ML307R的RXD引脚
#define DTU_UART_BUFFER_SIZE    512             //UART缓冲区大小
#define DTU_UART_BAUDRATE       115200          //待定

#define AT_CMD_UDP_OPEN         "AT+MIPOPEN=1,UDP,%s,%u\r\n"   //打开UDP连接的AT命令格式，%s为远程服务器IP地址，%u为远程服务器端口号

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

}coml_uart_config_t;


///---------------------------------------------------------------------------------------------------------------------///
///func


uint64_t default_timestamp_ms();

data_frame_t* coml_data_frame_init();
connection_frame_t* coml_connection_frame_init();

bool coml_uart_init(const coml_uart_config_t *config, QueueHandle_t *event_queue);

bool coml_uart_send_data_frame(const data_frame_t *frame);
bool coml_uart_send_connection_frame(const connection_frame_t *frame);

bool coml_connect_test_1();


#endif
