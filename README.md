# Intecture [![Build Status](https://travis-ci.org/intecture/agent.svg?branch=master)](https://travis-ci.org/intecture/agent) [![Coverage Status](https://coveralls.io/repos/github/Intecture/agent/badge.svg?branch=master)](https://coveralls.io/github/Intecture/agent?branch=master)

Intecture is a developer friendly, language agnostic configuration management tool for server systems.

* Extensible support for virtually any programming language
* Standard programming interface. No DSL. No magic.
* Rust API library (and bindings for popular languages)

You can find out more at [intecture.io](http://intecture.io).

# System Requirements

Intecture relies on [ZeroMQ](http://zeromq.org) for communication between your project and your managed hosts. The Intecture installer will install these dependencies automatically, however if you are building Intecture manually, you will need to install ZeroMQ and CZMQ before proceeding.

# Install

The best way to get up and running is by using the Intecture installer:

```
$ curl -sSf https://get.intecture.io/ | sh -s -- agent
```

# Uninstall

If you used the Intecture installer to install the Agent, you can also use it for removal:

```
$ curl -sSf https://get.intecture.io/ | sh -s -- -u agent
```

# Support

For enterprise support and consulting services, please email <mailto:support@intecture.io>.

For any bugs, feature requests etc., please ticket them on GitHub.
