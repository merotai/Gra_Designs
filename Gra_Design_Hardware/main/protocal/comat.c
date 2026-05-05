


#include "comat.h"


static comat_uart_config_t global_config_h;

/// 等待ML307R模块响应结果
static bool comat_uart_wait_for_result(uint32_t timeout_ms){ 
    uint8_t buffer[128];
    uint32_t elapsed = 0;

    int total_bytes = 0;
    while(elapsed < timeout_ms){
        int read = uart_read_bytes(global_config_h.uart_port, buffer, sizeof(buffer) - 1, pdMS_TO_TICKS(100));//ms为单位
        if (read > 0)
        {
            buffer[read] = '\0';
            total_bytes += read;
            ESP_LOGW("COMAT", "Received %d bytes: [%s]", read, buffer);
            if (strstr((const char *)buffer, "OK") != NULL)
            {
                ESP_LOGI("COMAT", "Received OK response");
                return true;
            }
            if (strstr((const char *)buffer, "ERROR") != NULL)
            {
                ESP_LOGI("COMAT", "Received ERROR response");
                return false;
            }
        }
        elapsed += 100;
    }
    ESP_LOGW("COMAT", "Timeout after %dms, total bytes received: %d", timeout_ms, total_bytes);
    return false;
}






///
static bool comat_uart_wait_for_prompt(uint32_t timeout_ms){
    uint8_t buffer[128];
    uint32_t elapsed = 0;

    while(elapsed < timeout_ms){
        int read = uart_read_bytes(global_config_h.uart_port, buffer, sizeof(buffer) - 1, pdMS_TO_TICKS(100));
        if (read > 0)
        {
            buffer[read] = '\0';
            if (strchr((const char *)buffer, '>') != NULL)
            {
                ESP_LOGI("COMAT", "Received prompt");
                return true;
            }
            if (strstr((const char *)buffer, "ERROR") != NULL)
            {
                ESP_LOGI("COMAT", "Received ERROR response while waiting for prompt");
                return false;
            }
        }
        elapsed += 100;
    }
    return false;
}



/// 形参at_cmd, AT指令; timeout_ms, 设置为3000ms
static bool coml_send_AT(const char* at_cmd, uint32_t timeout_ms) {
    uint8_t buffer[128];
    uint32_t elapsed_time = 0;

    if (at_cmd == NULL) {
        return false;
    }
    uart_flush(global_config_h.uart_port);
    int write_len = uart_write_bytes(global_config_h.uart_port, at_cmd, (int)strlen(at_cmd));

    if (write_len != (int)strlen(at_cmd)) {
        return false;
    }

    while(elapsed_time < timeout_ms){
        int read = uart_read_bytes(global_config_h.uart_port, buffer, sizeof(buffer) - 1, pdMS_TO_TICKS(100));//ms为单位
        if (read > 0)
        {
            buffer[read] = '\0';
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
        elapsed_time += 100;
    }
    ESP_LOGI("COML", "AT command response timeout");
    return false; // 超时后默认返回false
}




bool coml_udp_config() {
    return true;
}


/// 
bool coml_udp_open(void) {
   return true;
}

///
bool coml_udp_close() {
    return true;
}


///
bool coml_udp_send_connection_frame() {
    return true;
}

///
bool coml_udp_send_data_frame() {
    return true;
}

///
bool coml_udp_send_heartbeat_frame() {
    return true;
}

                     
/// -------------------------------------------------------------------------------------------------------------------











///

// bool coml_connect_test(){

//     char* at_cmd_1 = "AT\r\n";
//     char* at_cmd_2 = "AT+CGMR=1\r\n";
//     char* at_cmd_3 = "AT+MCCID\r\n";
//     char* at_cmd_4 = "AT+CEREG?\r\n";
//     char* at_cmd_5 = "AT+CSQ\r\n";
//     char* at_cmd_6 = "AT+CGDCONT=0,\"IP\",\"CMNET\"\r\n";
//     char* at_cmd_7 = "AT+CGACT=0,1\r\n";
//     char* at_cmd_8 = "AT+CGPADDR=0\r\n";
//     char at_cmd_9[64] = "AT+MIPOPEN=0,UDP,%s,%u\r\n";
//     char at_cmd_10[64] = "AT+MIPSEND=0,%d\r\n";
//     char* at_cmd_11 = "AT+MIPCLOSE=0\r\n";
    
//     // 1
//     uart_flush(global_config.uart_port);
//     int write_len = uart_write_bytes(global_config.uart_port, at_cmd_1, (int)strlen(at_cmd_1));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_1);
//     if (write_len != (int)strlen(at_cmd_1)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 1 sent successfully, waiting for response...");
//     bool result_1 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 1 response: %s", result_1 ? "OK" : "ERROR");

//     // 2
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_2, (int)strlen(at_cmd_2));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_2);
//     if (write_len != (int)strlen(at_cmd_2)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 2 sent successfully, waiting for response...");
//     bool result_2 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 2 response: %s", result_2 ? "OK" : "ERROR");

//     // 3
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_3, (int)strlen(at_cmd_3));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_3);
//     if (write_len != (int)strlen(at_cmd_3)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 3 sent successfully, waiting for response...");
//     bool result_3 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 3 response: %s", result_3 ? "OK" : "ERROR");

//     // 4
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_4, (int)strlen(at_cmd_4));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_4);
//     if (write_len != (int)strlen(at_cmd_4)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 4 sent successfully, waiting for response...");
//     bool result_4 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 4 response: %s", result_4 ? "OK" : "ERROR");

//     // 5
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_5, (int)strlen(at_cmd_5));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_5);
//     if (write_len != (int)strlen(at_cmd_5)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 5 sent successfully, waiting for response...");
//     bool result_5 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 5 response: %s", result_5 ? "OK" : "ERROR");

//     // 6
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_6, (int)strlen(at_cmd_6));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_6);
//     if (write_len != (int)strlen(at_cmd_6)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 6 sent successfully, waiting for response...");
//     bool result_6 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 6 response: %s", result_6 ? "OK" : "ERROR");

//     // 7
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_7, (int)strlen(at_cmd_7));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_7);
//     if (write_len != (int)strlen(at_cmd_7)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 7 sent successfully, waiting for response...");
//     bool result_7 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 7 response: %s", result_7 ? "OK" : "ERROR");

//     // 8
//     uart_flush(global_config.uart_port);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_8, (int)strlen(at_cmd_8));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_8);
//     if (write_len != (int)strlen(at_cmd_8)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 8 sent successfully, waiting for response...");
//     bool result_8 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 8 response: %s", result_8 ? "OK" : "ERROR");




//     // 9 打开UDP连接,对接服务器IP地址和端口号
//     uart_flush(global_config.uart_port);
//     snprintf(at_cmd_9, sizeof(&at_cmd_9), at_cmd_9,"192.168.81.251" , 443);
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_9, (int)strlen(at_cmd_9));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_9);
//     if (write_len != (int)strlen(at_cmd_9)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 9 sent successfully, waiting for response...");
//     bool result_9 = coml_uart_wait_for_result(3000);
//     ESP_LOGI("COML", "AT command 9 response: %s", result_9 ? "OK" : "ERROR");




//     // 10 发送数据
//     uart_flush(global_config.uart_port);
//     snprintf(at_cmd_10, sizeof(&at_cmd_10), at_cmd_10, 5); // 发送5字节数据的AT命令
//     write_len = uart_write_bytes(global_config.uart_port, at_cmd_10, (int)strlen(at_cmd_10));
//     ESP_LOGI("COML", "Sent AT command: %s", at_cmd_10);
//     if (write_len != (int)strlen(at_cmd_10)) {
//         return false;
//     }
//     ESP_LOGI("COML", "AT command 10 sent successfully, waiting for response...");
//     bool result_10 = coml_uart_wait_for_prompt(3000);
//     ESP_LOGI("COML", "AT command 10 response: %s", result_10 ? "received" : "ERROR");

//     write_len = uart_write_bytes(global_config.uart_port, "Hello", 5); // 发送5字节数据
//     if (write_len != 5) {
//         ESP_LOGI("COML", "Failed to send data payload");
//         return false;
//     }
//     ESP_LOGI("COML", "Sent data payload: Hello");

//     return true;

// }