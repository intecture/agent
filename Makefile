UNAME_S := $(shell uname -s)
CARGO := $(shell which cargo)
TARGET = release
PREFIX = /usr/local

all:
ifeq ($(TARGET), release)
	$(CARGO) build --release
else
	$(CARGO) build
endif

install:
	mkdir -p $(PREFIX)/etc/intecture
	install -m 0644 resources/agent.json $(PREFIX)/etc/intecture/
	install -m 0755 target/$(TARGET)/inagent-api $(PREFIX)/bin/
	install -m 0755 target/$(TARGET)/inagent-file $(PREFIX)/bin/
	install -m 0644 target/$(TARGET)/libinagent.rlib $(PREFIX)/lib/

uninstall:
	rm -f $(PREFIX)/bin/inagent-api
	rm -f $(PREFIX)/bin/inagent-file
	rm -f $(PREFIX)/lib/libinagent.rlib
	rm -f $(PREFIX)/etc/intecture/agent.json
	if [ ! "$(ls -A /$(PREFIX)/etc/intecture)" ]; then\
		rmdir $(PREFIX)/etc/intecture; \\
	fi

test:
ifeq ($(TARGET), release)
	$(CARGO) test --release
else
	$(CARGO) test
endif

clean:
	$(CARGO) clean
