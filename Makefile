CFLAGS = -Wall -Werror `pkg-config --cflags libevent`
TARGET = target/debug
LIBS = \
	-L$(TARGET) \
	-ltokio_libevent

all: $(TARGET)/time-test

$(TARGET)/hello-world: sample/hello-world.c
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

$(TARGET)/time-test: sample/time-test.c
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

clean:
	$(RM) $(TARGET)/hello-world $(TARGET)/time-test
