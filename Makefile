TARGET = release
PREFIX = /usr/local

all:
ifeq ($(TARGET), release)
	cargo build --release
else
	cargo build
endif

install:
	mkdir -p $(PREFIX)/etc/intecture
	sed 's~<CFGPATH>~$(PREFIX)/etc/intecture~' resources/agent.json > $(PREFIX)/etc/intecture/agent.json
	chmod 0644 $(PREFIX)/etc/intecture/agent.json
	install -m 0755 target/$(TARGET)/inagent $(PREFIX)/bin/
	if [ -f /etc/rc.conf ]; then \
		install -m 555 resources/init/freebsd /etc/rc.d/inagent; \
	elif stat --format=%N /proc/1/exe|grep -qs systemd ; then \
		if [ -d /usr/lib/systemd/system ]; then \
			install -m 644 resources/init/systemd /usr/lib/systemd/system/inagent.service; \
		elif [ -d /lib/systemd/system ]; then \
			install -m 644 resources/init/systemd /lib/systemd/system/inagent.service; \
		fi; \
	elif [ -f /etc/redhat-release ]; then \
		install -m 755 resources/init/redhat /etc/init.d/inagent; \
	elif [ -f /etc/debian_version ]; then \
		install -m 755 resources/init/debian /etc/init.d/inagent; \
	fi;

uninstall:
	rm -f $(PREFIX)/bin/inagent
	rm -f $(PREFIX)/etc/intecture/agent.json
	rmdir --ignore-fail-on-non-empty $(PREFIX)/etc/intecture
	rm -f /lib/systemd/system/inagent.service
	rm -f /usr/lib/systemd/system/inagent.service
	rm -f /etc/init.d/inagent
	rm -f /etc/rc.d/inagent

test:
ifeq ($(TARGET), release)
	cargo test --release
else
	cargo test
endif

clean:
	cargo clean
