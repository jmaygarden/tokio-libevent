CFLAGS += -Iinclude
TARGET = target/debug
LIBS = \
	-L$(TARGET) \
	-ltokio_libevent

all: time-test

hello-world: sample/hello-world.c
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

time-test: sample/time-test.c
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

clean:
	$(RM) hello-world
