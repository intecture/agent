UNAME_S := $(shell uname -s)
TARGET = release;

ifneq ($(grep -qs Fedora /etc/redhat-release || echo 1), 1)
	USRPATH = /usr/local
	ETCPATH = /usr/local/etc
	export PATH := $(USRPATH)/bin:$(PATH)
else ifeq ($(UNAME_S), Linux)
	USRPATH = /usr
	ETCPATH = /etc
else ifeq ($(UNAME_S), Darwin)
	USRPATH = /usr/local
	ETCPATH = /usr/local/etc
endif

all:
ifeq ($(TARGET), release)
	$(USRPATH)/bin/cargo build --release
else
	# XXX This symlink is to fix a bug with building zmq crate
	mkdir -p $(shell pwd)/lib
	ln -s /usr/local/lib $(shell pwd)/lib/x86_64-unknown-linux-gnu
	$(USRPATH)/bin/cargo build
endif

install:
	mkdir -p $(ETCPATH)/intecture
	install -m 0644 resources/agent.json $(ETCPATH)/intecture
	install -m 0755 target/$(TARGET)/inagent $(USRPATH)/bin

uninstall:
	rm -f $(USRPATH)/bin/inagent
	rm -f $(ETCPATH)/intecture/agent.json
	if [ ! "$(ls -A /tmp)" ]; then\
		rmdir $(ETCPATH)/intecture; \\
	fi

clean:
	$(USRPATH)/bin/cargo clean