UNAME_S := $(shell uname -s)
TARGET = release

ifeq ($(UNAME_S), Linux)
	FEDORA := $(grep -qs Fedora /etc/redhat-release)
	ifeq ($$?, 0)
		USRPATH = /usr/local
		ETCPATH = /usr/local/etc
		export PATH := $(USRPATH)/bin:$(PATH)
	else
		USRPATH = /usr
		ETCPATH = /etc
	endif
else ifeq ($(UNAME_S), Darwin)
	USRPATH = /usr/local
	ETCPATH = /usr/local/etc
endif

all:
ifeq ($(TARGET), release)
	$(USRPATH)/bin/cargo build --release
else
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