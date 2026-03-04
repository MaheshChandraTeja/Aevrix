#ifndef AEVRIX_HB_FFI_H
#define AEVRIX_HB_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>


#define HB_OK 0
#define HB_ERR_INVALID_ARG 1
#define HB_ERR_INTERNAL    2
#define HB_ERR_NOT_FOUND   3
#define HB_ERR_UNSUPPORTED 4


const char* hb_version(void);


int hb_init(uint32_t viewport_width, uint32_t viewport_height);


typedef uint32_t hb_doc_t;


hb_doc_t hb_load_html(const char* html_utf8);


hb_doc_t hb_load_url(const char* url_utf8);


struct hb_surface {
    uint8_t* pixels;     
    uint32_t width;
    uint32_t height;
    uint32_t stride;     
    size_t   len;        
};


int hb_render(hb_doc_t doc, uint32_t viewport_width, uint32_t viewport_height, struct hb_surface* out);


int hb_render_html(const char* html_utf8, uint32_t viewport_width, uint32_t viewport_height, struct hb_surface* out);


void hb_surface_release(struct hb_surface* surf);

#ifdef __cplusplus
}
#endif

#endif 
