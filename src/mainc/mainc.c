
#include <event2/event_struct.h>

#include "mainc.h"

#ifdef __APPLE__
#include <sys/queue.h>
#include <sys/event.h>
struct kqop {
  struct kevent *changes;
  int changes_size;

  struct kevent *events;
  int events_size;
  int kq;
  int notify_event_added;
  pid_t pid;
};
#else
struct /*epollop*/ kqop {
  void *events;
  int nevents;
  //int epfd;
  int kq;
//#ifdef USING_TIMERFD
//  int timerfd;
//#endif
};
#endif

struct event_base_internal {
  /** Function pointers and other data to describe this event_base's
   * backend. */
  void *evsel;
  /** Pointer to backend-specific data. */
  void *evbase;
};

static size_t counter = 0;

static void timer_cb(evutil_socket_t fd, short event, void *ptr)
{
  struct event* ev = (struct event*)ptr;

  printf("hi from callback\n");
  if (++counter > 30)
  {
    //event_base_loopbreak((struct event_base*)(ev->ev_base));
    event_del(ev);
  }
}

static void timer_cb_forever(evutil_socket_t fd, short event, void *ptr)
{
  struct event* ev = (struct event*)ptr;

  printf("hi from forever callback\n");
}

int mainc_init(struct event_base *base)
{
  //struct event_base *base = NULL;

  //base = event_base_new();

  if (base != NULL)
  {
    printf("base ain't null after init\n");
  }

  struct timeval one_sec = { 1, 0 };
  struct timeval hundred_ms = { 0, 100*1000 };
  struct event *ev, *ev2;
  ev = event_new(base, -1, EV_PERSIST, timer_cb_forever, event_self_cbarg());
  event_add(ev, &one_sec);
  ev2 = event_new(base, -1, EV_PERSIST, timer_cb, event_self_cbarg());
  event_add(ev2, &hundred_ms);

  return 0;
}

int base_fd(const struct event_base* base)
{
  return ((struct kqop*)((struct event_base_internal*)base)->evbase)->kq;
}

int mainc_destroy(struct event_base* base)
{
  event_base_free(base);

  if (base == NULL)
  {
    printf("base became null\n");
  }

  return 0;
}
