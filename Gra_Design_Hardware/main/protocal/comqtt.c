

#include "comqtt.h"

static comqtt_uart_config_t global_config;
/// MQTT协议payload初始化函数 
comqtt_payload_t*  payload_init() {
    comqtt_payload_t *payload = (comqtt_payload_t*)malloc(sizeof(comqtt_payload_t));
    if(payload == NULL) {
        return NULL;
    }
    memset(payload, 0, sizeof(comqtt_payload_t)); // 初始化为0

    payload->imu_num = 0; // 初始时没有样本，发送时根据实际样本数量填入

    // imu_samples数组的内容在封装数据帧时填入
    return payload;
}


///
bool comqtt_uart_init(const comqtt_uart_config_t* config, QueueHandle_t* event_queue)
{
    // UART初始化
    if (config == NULL){
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


bool comqtt_uart_send_payload(const comqtt_payload_t *payload) {
    if (payload == NULL) {
        return false;
    }

    int write_len = uart_write_bytes(global_config.uart_port, payload, sizeof(comqtt_payload_t));
    if (write_len != sizeof(comqtt_payload_t)) {
        ESP_LOGI("COMQTT", "Failed to send payload");
        return false;
    }
    ESP_LOGI("COMQTT", "Sent payload successfully");
    return true;

}










