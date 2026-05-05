
#ifndef INTERACT_TASK_H
#define INTERACT_TASK_H

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "freertos/event_groups.h"
#include "freertos/semphr.h"


extern QueueHandle_t        main_data_queue;           // 用于传递数据帧的队列
extern QueueHandle_t        main_uart_event_queue;     // 用于接收服务端发来的UART事件的队列

extern bool                 sensor_task_working;       // 传感器数据采集任务是否正常工作的标志
extern bool                 send_task_working;         // 数据发送任务是否正常工作的标志

extern EventGroupHandle_t   main_event_group;          // 用于事件通知的事件组
extern SemaphoreHandle_t    main_uart_mutex;           // 用于保护UART访问的互



/// 任务响应设计：
/// 1. BMI270传感器数据采集任务：负责从BMI270传感器读取数据，并将数据封装成数据帧，发送到数据队列。
/// 2. 数据发送任务：从数据队列中获取数据帧，通过UART发送给DTU模块。
/// 3. 数据接收任务：负责从UART接收DTU模块的响应数据，解析响应内容，并根据需要触发相应的事件或处理逻辑。

/// 其中数据接受任务


void init_task(void *pvParameters);
void bmi270_task_1(void *pvParameters);
void bmi270_task_2(void *pvParameters);
void send_task(void *pvParameters);



#endif