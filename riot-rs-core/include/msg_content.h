#ifndef MSG_CONTENT_T
#define MSG_CONTENT_T

typedef union {
    uint32_t value;
    void *ptr;
} msg_content_t;

#endif /* MSG_CONTENT_T */
