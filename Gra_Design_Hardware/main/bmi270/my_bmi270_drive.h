
#ifndef MY_BMI270_DRIVE_H
#define MY_BMI270_DRIVE_H



// reference




// static parameter


// function
int8_t bmi270_i2c_read(uint8_t reg_addr,uint8_t *reg_data, uint32_t len, void *intf_ptr);
int8_t bmi270_i2c_write(uint8_t reg_addr, const uint8_t *reg_data, uint32_t len, void *intf_ptr);
void bmi270_delay_us(uint32_t period, void *intf_ptr);

void bmi270_master_init();




#endif /* MY_BMI270_DRIVE_H */

