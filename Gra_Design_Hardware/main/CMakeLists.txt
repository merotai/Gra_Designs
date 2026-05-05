idf_component_register(
    SRCS 
    "hello_world_main.c" 
    "bmi270/bmi270_template.c"
    "protocal/coml.c"
    "protocal/comat.c"
    "protocal/comqtt.c"
    "cordination/interact_task.c"

                       
    INCLUDE_DIRS  
        "." 
        "bmi270"
        "protocal"
        "cordination"
         
    REQUIRES 
        driver
        bmi270_sensor
        spi_flash
        esp_timer
)
