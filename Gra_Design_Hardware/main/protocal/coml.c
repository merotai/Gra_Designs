///
///
///
#include "coml.h"

static coml_uart_config_t global_config;
/// --------------------------------------------------------------------------

/// 
uint64_t default_timestamp_ms(void)
{
    return (uint64_t)(esp_timer_get_time() / 1000ULL);
}



data_frame_t* coml_data_frame_init() {
    data_frame_t *frame = (data_frame_t*)malloc(sizeof(data_frame_t));
    if(frame == NULL) {
        return NULL;
    }
    memset(frame, 0, sizeof(data_frame_t)); // 初始化为0

    frame->header = FRAME_DATA_HEADER;
    frame->device_id = DEVICE_ID;
    frame->frame_type = DATA_FRAME_TYPE_SENSOR_1;

    // 以下部分须在数据封装时填入
    // frame->frame_len = sizeof(frame->samples);
    // frame->timestamp = default_timestamp_ms();
    // frame->samples
    // frame->crc32_check 
    
    return frame;
}


connection_frame_t* coml_connection_frame_init() {
    connection_frame_t *frame = (connection_frame_t*)malloc(sizeof(connection_frame_t));
    if(frame == NULL) {
        return NULL;
    }
    memset(frame, 0, sizeof(connection_frame_t)); // 初始化为0

    frame->header = FRAME_CONNECTION_HEADER;
    frame->device_id = DEVICE_ID;
    frame->frame_type = DATA_FRAME_TYPE_CONNECTION;

    frame->frame_len = 0; // 连接帧没有数据部分
    frame->protocol_version = PROTOCOL_VERSION;
    frame->reserved = 0; // 保留字段填0

    // 以下部分须在连接帧封装时填入
    // frame->timestamp = default_timestamp_ms();   // 发送时写入
    // frame->order = 1;                            // 1表示申请连接，2表示断开连接 发送时写入
    // frame->crc32_check                           // 发送时写入
    
    return frame;
}



///
bool coml_uart_init(const coml_uart_config_t* config, QueueHandle_t* event_queue)
{
    // UART初始化
    if (config == NULL || config->remote_host == NULL){
        return false;
    }

    global_config = *config;

    uart_config_t uart_config = {
        .baud_rate = config->baudrate,
        .data_bits = UART_DATA_8_BITS,
        .parity    = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };
    if (uart_param_config(global_config.uart_port, &uart_config) != ESP_OK)
    {
        return false;
    }

    if (uart_set_pin(global_config.uart_port,   global_config.tx_pin, 
                     global_config.rx_pin,      UART_PIN_NO_CHANGE, 
                     UART_PIN_NO_CHANGE
                    ) != ESP_OK)
    {
        return false;
    }

    if (uart_driver_install(global_config.uart_port,        global_config.rx_buffer_size, 
                            global_config.tx_buffer_size,   15, 
                            event_queue,                    0
                        ) != ESP_OK)
    {
        return false;
    }

    return true;
}

/// 根据DTU固件特性,这边直接发送数据
bool coml_uart_send_data_frame(const data_frame_t *frame) {
    if (frame == NULL) {
        return false;
    }

    int write_len = uart_write_bytes(global_config.uart_port, frame, sizeof(data_frame_t));
    if (write_len != sizeof(data_frame_t)) {
        ESP_LOGI("COML", "Failed to send data frame");
        return false;
    }
    ESP_LOGI("COML", "Sent data frame successfully");
    return true;
}

bool coml_uart_send_connection_frame(const connection_frame_t *frame) {
    if (frame == NULL) {
        return false;
    }

    int write_len = uart_write_bytes(global_config.uart_port, frame, sizeof(connection_frame_t));
    if (write_len != sizeof(connection_frame_t)) {
        ESP_LOGI("COML", "Failed to send connection frame");
        return false;
    }
    ESP_LOGI("COML", "Sent connection frame successfully");
    return true;
}

///
bool coml_uart_get_response(uint32_t timeout_ms) {
    uint8_t buffer[128];
    uint32_t elapsed = 0;

    int total_bytes = 0;
    while(elapsed < timeout_ms){
        int read = uart_read_bytes(global_config.uart_port, buffer, sizeof(buffer) - 1, pdMS_TO_TICKS(100));//ms为单位
        if (read > 0)
        {
            buffer[read] = '\0';
            total_bytes += read;
            ESP_LOGW("COML", "Received %d bytes: [%s]", read, buffer);
            if (strstr((const char *)buffer, "OK") != NULL)
            {
                ESP_LOGI("COML", "Received OK response");
                return true;
            } 
            if (strstr((const char *)buffer, "ERROR") != NULL)
            {
                ESP_LOGI("COML", "Received ERROR response");
                return false;
            }
        }
        elapsed += 100;
    }

    ESP_LOGW("COML", "Timeout after %dms, total bytes received: %d", timeout_ms, total_bytes);
    return false;

}




bool coml_connect_test_1(){
    char* at_cmd_1 = "Hello World";
     // 1
    
    int write_len = uart_write_bytes(global_config.uart_port, at_cmd_1, (int)strlen(at_cmd_1));
    ESP_LOGI("COML", "Sent command: %s", at_cmd_1);
    if (write_len != (int)strlen(at_cmd_1)) {
        return false;
    }
    ESP_LOGI("COML", "Command 1 sent successfully");
    bool result = coml_uart_get_response(5000);
    if (result) {
        ESP_LOGI("COML", "Received OK response for command 1");
    } else {
        ESP_LOGI("COML", "Did not receive OK response for command 1");
    }
    return true;
}




