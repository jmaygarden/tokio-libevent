CFLAGS = -Wall -Werror `pkg-config --cflags libevent`
TARGET = target/debug
LIBS = \
	-L$(TARGET) \
	-ltokio_libevent
SAMPLES_SRC = \
	libevent/sample/hello-world.c \
	libevent/sample/time-test.c
SAMPLES_BIN = $(patsubst %.c,%,$(SAMPLES_SRC))

all: $(SAMPLES_BIN)

%: %.c $(TARGET)/libtokio_libevent.a
	$(CC) $(CFLAGS) -o $@ $(LIBS) $<

$(TARGET)/libtokio_libevent.a: FORCE
	cargo build

FORCE: ;

clean:
	$(RM) $(SAMPLES_BIN)
