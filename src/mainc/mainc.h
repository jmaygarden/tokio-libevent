//
// Created by Jon Magnuson on 11/16/19.
//

#ifndef TOKIO_LIBEVENT_MAINC_H
#define TOKIO_LIBEVENT_MAINC_H

#include <event2/event.h>

int mainc_init(struct event_base* base, evutil_socket_t tokio_fd);
int base_fd(const struct event_base* base);
int mainc_destroy(struct event_base* base);

#endif //TOKIO_LIBEVENT_MAINC_H
