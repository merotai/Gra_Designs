
#include "esp_rom_crc.h"
#include "interact_task.h"

#include "bmi270_template.h"

#include "comqtt.h"


QueueHandle_t        main_data_queue           = NULL;      // 用于传递数据帧的队列
QueueHandle_t        main_uart_event_queue     = NULL;      // 用于接收服务端发来的UART事件的队列


// ---------------------------------暂未使用-------------------------------------------------------
//
//
//
   bool              sensor_task_working     = false;       // 传感器数据采集任务是否正常工作的标志
   bool              send_task_working       = false;       // 数据发送任务是否正常工作的标志

EventGroupHandle_t   main_event_group    = NULL;             // 用于事件通知的事件组
SemaphoreHandle_t    main_uart_mutex     = NULL;             // 用于保护UART访问的互斥锁


void init_task(void *pvParameters) {

    main_data_queue = xQueueCreate(20 ,sizeof(comqtt_payload_t)); // 创建一个长度为35，元素类型为comqtt_payload_t的数据队列
    main_uart_event_queue = xQueueCreate(15, sizeof(uart_event_t)); // 创建一个长度为15，元素类型为uart_event_t的事件队列
    

    comqtt_uart_config_t config = {
        .uart_port = UART_NUM_1,
        .tx_pin = 16,
        .rx_pin = 17,
        .baudrate = 115200,
        .rx_buffer_size = 512,
        .tx_buffer_size = 512,
    };
    
    // 初始化UART
    if (comqtt_uart_init(&config, &main_uart_event_queue)) {
        send_task_working = true;
        ESP_LOGI("COMQTT", "UART initialized successfully");
    } else {
        send_task_working = false;
        ESP_LOGE("COMQTT", "Failed to initialize UART");
    }

    // 初始化BMI270传感器1
    if (bmi270_init_1() != 0) {
        ESP_LOGE("BMI270_TASK", "Failed to initialize BMI270 template1");
        vTaskDelete(NULL);
        return;
    }

    if (bmi270_init_2() != 0) {
        ESP_LOGE("BMI270_TASK", "Failed to initialize BMI270 template2");
        vTaskDelete(NULL);
        return;
    }


    vTaskDelete(NULL); // 初始化完成后资源保留，删除该任务

}



/// BMI270传感器1号数据采集任务
void bmi270_task_1(void *pvParameters) {
    //
    comqtt_payload_t samples_h1;
    samples_h1.imu_num = 1; 
    bmi270_sample_t sample;
    while (1) {
        
        for (int i = 0; i < 10; i++) {
            //读取数据
            if (bmi270_template_read(&sample) != 0) {
                ESP_LOGE("BMI270_TASK1", "sensor data read from BMI270 <FAILED>");
                // 读取失败，稍后0.1s重试
                i = i - 1; // 读取失败，重试当前样本
                vTaskDelay(pdMS_TO_TICKS(100));
                continue;
            }
            samples_h1.imu_samples[i] = sample;
            vTaskDelay(pdMS_TO_TICKS(100)); // 每0.1秒采集一次数据
        }
        // 发送数据帧地址到队列，队满则等待0.1s再处理
        if(xQueueSend(main_data_queue, &samples_h1, pdMS_TO_TICKS(100)) == pdTRUE) {
            ESP_LOGI("BMI270_TASK1", "sensor data sent to queue <SUCCESS>");
        } else {
            ESP_LOGE("BMI270_TASK1", "sensor data send to queue <FAILED>");
            vTaskDelay(pdMS_TO_TICKS(100)); // 发送失败，稍后0.1s重试
        }
        
    }

}
/// BMI270传感器2号数据采集任务
void bmi270_task_2(void *pvParameters) {
    //
    comqtt_payload_t samples_h2;
    samples_h2.imu_num = 2; 
    bmi270_sample_t sample;
    while (1) {
        
        for (int i = 0; i < 10; i++) {
            //读取数据
            if (bmi270_template_read(&sample) != 0) {
                ESP_LOGE("BMI270_TASK2", "sensor data read from BMI270 <FAILED>");
                // 读取失败，稍后0.1s重试
                i = i - 1; // 读取失败，重试当前样本
                vTaskDelay(pdMS_TO_TICKS(100));
                continue;
            }
            samples_h2.imu_samples[i] = sample;
            vTaskDelay(pdMS_TO_TICKS(100)); // 每0.1秒采集一次数据
        }
        // 发送数据帧地址到队列，队满则等待0.1s再处理
        if(xQueueSend(main_data_queue, &samples_h2, pdMS_TO_TICKS(100)) == pdTRUE) {
            ESP_LOGI("BMI270_TASK2", "sensor data sent to queue <SUCCESS>");
        } else {
            ESP_LOGE("BMI270_TASK2", "sensor data send to queue <FAILED>");
            vTaskDelay(pdMS_TO_TICKS(100)); // 发送失败，稍后0.1s重试
        }
    
    }
}





/// MQTT数据发送任务
void send_task(void *pvParameters){
    ESP_LOGI("SEND_TASK", "Send task started");
    comqtt_payload_t samples_send;
    while(1) {
        for (int i = 0; i < 10; i++) {
            comqtt_payload_t samples;
            if (xQueueReceive(main_data_queue, &samples, portMAX_DELAY) == pdTRUE) {
                ESP_LOGI("SEND_TASK", "Received sample %d from queue", i);
            } else {
                ESP_LOGE("SEND_TASK", "Failed to receive sample %d from queue", i);
            }
            samples_send = samples;
        }

        if (comqtt_uart_send_payload(&samples_send)) {
            ESP_LOGI("SEND_TASK", "Payload sent successfully");
        } else {
            ESP_LOGE("SEND_TASK", "Failed to send payload");
        }
    }
}



/// 数据接收任务 
void receive_task(void *pvParameters) {
    uart_event_t event;
    char rx_buffer[128];
    while (1) {
        if (xQueueReceive(main_uart_event_queue, &event, portMAX_DELAY) == pdTRUE) {

            switch (event.type){
                //
                case UART_DATA: {
                    int len = uart_read_bytes(UART_NUM_0, rx_buffer, event.size, pdMS_TO_TICKS(100));
                    if (len > 0 && len < sizeof(rx_buffer) - 1) {
                        rx_buffer[len] = '\0';
                        if (strstr(rx_buffer,"0xbcef019156040fecb-CONNECTION-OK")) {
                            sensor_task_working = true;
                            ESP_LOGI("RECEIVE_TASK", "Connection established with server");
                        } else if (strstr(rx_buffer,"0xbcef019156040fecb-CONNECTION-FAIL")) {
                            sensor_task_working = false;
                            ESP_LOGW("RECEIVE_TASK", "Connection lost with server");
                        } else {
                            ESP_LOGI("RECEIVE_TASK", "Received UART data: %s", rx_buffer);
                        }
                    } else {
                        ESP_LOGE("RECEIVE_TASK", "Failed to read UART data");
                    }
                    break;
                }
                //
                case UART_BUFFER_FULL: {
                    // 发生溢出，清空缓冲区
                    uart_flush_input(UART_NUM_0);
                    xQueueReset(main_uart_event_queue);
                    ESP_LOGW("RECEIVE_TASK", "UART buffer full and flushed");
                    vTaskDelay(pdMS_TO_TICKS(500)); // 等待0.5s再处理
                    break;
                }
                //
                default: {
                    ESP_LOGW("RECEIVE_TASK", "Unknown UART event type");
                    break;
                }
                    
            }
        }

    }
}







// /// 数据发送任务
// void send_task(void *pvParameters) {
//     ESP_LOGI("SEND_TASK", "Send task started, waiting for data frames...");
//     while(1) {


//         if (send_task_working == true) {
//             // 创建一个新的数据帧，填充3/7的数据字段
//             data_frame_t *frame = coml_data_frame_init(); 
//             if (frame == NULL) {
//                 ESP_LOGE("SEND_TASK", "Failed to allocate memory for data frame");
//                 continue;
//             }
            
//             //从队列取十次数据
//             for(int i = 0; i < 10; i++) {
//                 if (xQueueReceive(main_data_queue, &frame->samples[i], portMAX_DELAY) == pdTRUE) {
//                     ESP_LOGI("SEND_TASK", "Received sample %d from queue", i);
//                 } else {
//                     ESP_LOGE("SEND_TASK", "Failed to receive sample %d from queue", i);
//                 }
            
//             }// 填充4/7的数据字段
            
//             // 填充数据帧的其他字段
//             // 获取当前时间戳，单位为毫秒   5/7
//             frame->timestamp = default_timestamp_ms(); 
//             // 数据部分长度为10个样本的大小   6/7
//             frame->frame_len = sizeof(frame->samples); 
//             // 计算CRC32校验码，填入frame->crc32_check 7/7

//             uint8_t *data_crc32_check = (uint8_t*)frame->samples; // 数据部分的起始地址
//             uint32_t data_crc32_len = sizeof(frame->samples); // 数据部分的长度

//             frame->crc32_check = esp_rom_crc32_le(0, data_crc32_check, data_crc32_len); // 计算CRC32校验码
            
//             // 从数据队列中获取数据帧，并通过UART发送给DTU模块
//             if (coml_uart_send_data_frame(frame)) {
//                 ESP_LOGI("SEND_TASK", "Data frame sent successfully");
//             } else {
//                 ESP_LOGE("SEND_TASK", "Failed to send data frame");
//             }

//             // 发送完成后释放数据帧内存
//             free(frame); 

//         } else {
//             ESP_LOGW("SEND_TASK", "UART not initialized or not connected, waiting for connection...");
//             vTaskDelay(pdMS_TO_TICKS(1000)); // 每1秒检查一次连接状态
//         }
        
//     }
    
// }





















