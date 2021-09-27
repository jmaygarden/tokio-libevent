CFLAGS = -Wall -Werror `pkg-config --cflags libevent`
TARGET = target/debug
LIBS = \
	-L$(TARGET) \
	-ltokio_libevent

all: $(TARGET)/time-test

$(TARGET)/hello-world: sample/hello-world.c $(TARGET)/libtokio_libevent.a
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

$(TARGET)/time-test: sample/time-test.c $(TARGET)/libtokio_libevent.a
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

$(TARGET)/libtokio_libevent.a: FORCE
	cargo build

FORCE: ;

clean:
	$(RM) $(TARGET)/hello-world $(TARGET)/time-test
