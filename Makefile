TARGET = release
PREFIX = /usr/local
SYSCONFDIR = "$(PREFIX)/etc"

all:
ifeq ($(TARGET), release)
	cargo build --release
else
	cargo build
endif

install:
	mkdir -p $(SYSCONFDIR)/intecture
	sed 's~{{sysconfdir}}~$(SYSCONFDIR)~' resources/agent.json.tpl > $(SYSCONFDIR)/intecture/agent.json
	chmod 0644 $(SYSCONFDIR)/intecture/agent.json
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
	rm -f $(PREFIX)/bin/inagent \
		  $(SYSCONFDIR)/intecture/agent.json \
		  /lib/systemd/system/inagent.service \
		  /usr/lib/systemd/system/inagent.service \
		  /etc/init.d/inagent \
		  /etc/rc.d/inagent;
	rmdir --ignore-fail-on-non-empty $(SYSCONFDIR)/intecture

test:
ifeq ($(TARGET), release)
	cargo test --release
else
	cargo test
endif

clean:
	cargo clean
