
#include <event2/event_struct.h>

#include "evhack.h"

#ifdef __APPLE__
#include <sys/queue.h>
#include <sys/event.h>
struct kqop {
  void /*struct kevent*/ *changes;
  int changes_size;

  void /*struct kevent*/ *events;
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


int base_fd(const struct event_base* base)
{
  return ((struct kqop*)((struct event_base_internal*)base)->evbase)->kq;
}

