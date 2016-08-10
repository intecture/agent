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
	sed 's~<CFGPATH>~$(PREFIX)/etc/intecture~' resources/agent.json > $(PREFIX)/etc/intecture/agent.json
	chmod 0644 $(PREFIX)/etc/intecture/agent.json
	install -m 0755 target/$(TARGET)/inagent $(PREFIX)/bin/

uninstall:
	rm -f $(PREFIX)/bin/inagent
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
