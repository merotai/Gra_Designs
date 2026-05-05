#ifndef BMI270_TEMPLATE_H
#define BMI270_TEMPLATE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float ax_g;
    float ay_g;
    float az_g;
    float gx_dps;
    float gy_dps;
    float gz_dps;
} bmi270_sample_t; // 24bytes

int8_t bmi270_init_1(void);
int8_t bmi270_init_2(void);
int8_t bmi270_init_3(void);
int8_t bmi270_template_read(bmi270_sample_t *sample);

#ifdef __cplusplus
}
#endif

#endif /* BMI270_TEMPLATE_H */
