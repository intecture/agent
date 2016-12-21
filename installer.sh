#!/bin/sh
# Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
# top-level directory of this distribution and at
# https://intecture.io/COPYRIGHT.
#
# Licensed under the Mozilla Public License 2.0 <LICENSE or
# https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
# modified, or distributed except according to those terms.

# Undefined vars are errors
set -u

# Globals
prefix="{{prefix}}"
libdir="{{libdir}}"
libext="{{libext}}"
sysconfdir="{{sysconfdir}}"
pkgconf="{{pkgconf}}"
pkgconfdir="{{pkgconfdir}}"
os="{{os}}"

do_install() {
    need_cmd $pkgconf

    local _one=
    local _two=

    if ! $($pkgconf --exists libzmq); then
        if [ "$os" = "darwin" ]; then
            _one="5"
            _two=$libext
        else
            _one=$libext
            _two="5"
        fi
        install -m 755 lib/libzmq.$libext $libdir/libzmq.$_one.$_two
        ln -s $libdir/libzmq.$_one.$_two $libdir/libzmq.$libext
        install -m 644 lib/pkgconfig/libzmq.pc $pkgconfdir
        install -m 644 include/zmq.h $prefix/include/

        if [ "$os" = "freebsd" ]; then
            install -m 644 lib/libstdc++.so.6 $libdir/
        fi
    fi

    if ! $($pkgconf --exists libczmq); then
        if [ "$os" = "darwin" ]; then
            _one="4"
            _two=$libext
        else
            _one=$libext
            _two="4"
        fi
        install -m 755 lib/libczmq.$libext $libdir/libczmq.$_one.$_two
        ln -s $libdir/libczmq.$_one.$_two $libdir/libczmq.$libext
		install -m 644 lib/pkgconfig/libczmq.pc $pkgconfdir
        install -m 644 include/czmq.h $prefix/include/
        install -m 644 include/czmq_library.h $prefix/include/
        install -m 644 include/czmq_prelude.h $prefix/include/
        install -m 644 include/zactor.h $prefix/include/
        install -m 644 include/zarmour.h $prefix/include/
        install -m 644 include/zauth.h $prefix/include/
        install -m 644 include/zbeacon.h $prefix/include/
        install -m 644 include/zcert.h $prefix/include/
        install -m 644 include/zcertstore.h $prefix/include/
        install -m 644 include/zchunk.h $prefix/include/
        install -m 644 include/zclock.h $prefix/include/
        install -m 644 include/zconfig.h $prefix/include/
        install -m 644 include/zdigest.h $prefix/include/
        install -m 644 include/zdir.h $prefix/include/
        install -m 644 include/zdir_patch.h $prefix/include/
        install -m 644 include/zfile.h $prefix/include/
        install -m 644 include/zframe.h $prefix/include/
        install -m 644 include/zgossip.h $prefix/include/
        install -m 644 include/zhash.h $prefix/include/
        install -m 644 include/zhashx.h $prefix/include/
        install -m 644 include/ziflist.h $prefix/include/
        install -m 644 include/zlist.h $prefix/include/
        install -m 644 include/zlistx.h $prefix/include/
        install -m 644 include/zloop.h $prefix/include/
        install -m 644 include/zmonitor.h $prefix/include/
        install -m 644 include/zmsg.h $prefix/include/
        install -m 644 include/zpoller.h $prefix/include/
        install -m 644 include/zproxy.h $prefix/include/
        install -m 644 include/zrex.h $prefix/include/
        install -m 644 include/zsock.h $prefix/include/
        install -m 644 include/zstr.h $prefix/include/
        install -m 644 include/zsys.h $prefix/include/
        install -m 644 include/zuuid.h $prefix/include/
    fi

    case "$os" in
        centos | fedora | debian | ubuntu)
            if $(stat --format=%N /proc/1/exe|grep -qs systemd); then
                if [ -d $prefix/usr/systemd/system ]; then
                    install -m 644 systemd $prefix/lib/systemd/system/inagent.service
                elif [ -d /lib/systemd/system ]; then
                    install -m 644 systemd /lib/systemd/system/inagent.service
                fi
            else
                install -m 755 init $sysconfdir/init.d/inagent
            fi
            ;;
        freebsd)
            install -m 555 init $sysconfdir/rc.d/inagent;
            ;;
    esac

    mkdir -p $sysconfdir/intecture
    install -m 644 agent.json $sysconfdir/intecture/

    install -m 755 inagent $prefix/bin
}

install_certs() {
    install -m 600 $1 $sysconfdir/intecture/agent.crt
    install -m 600 $2 $sysconfdir/intecture/auth.crt_public
}

amend_conf() {
    local _confpath=$sysconfdir/intecture/agent.json
    if [ ! -f $_confpath ]; then
        echo "Agent conf file not found. Run `installer.sh install` first."
        exit 1
    fi

    # If value is int, don't enclose in quotes
    if [ $2 -eq $2 2> /dev/null ]; then
        local _quotes=""
    else
        local _quotes='"'
    fi

    local _q='"' # Need this for FreeBSD, which doesn't respect escapes
    local _line=$(grep -n "$_q$1$_q:" $_confpath | cut -f1 -d:)
    local _total=$(wc -l $_confpath | sed 's/^ *//' | cut -f1 -d' ')
    if [ `expr $_total - $_line` -gt 1 ]; then
        local _comma=","
    else
        local _comma=""
    fi

    local _tmpfile=$(mktemp 2>/dev/null || mktemp -t inagent-conf)
    sed "s~\"$1\": .*~\"$1\": $_quotes$2$_quotes$_comma~" < $_confpath > $_tmpfile || exit 1
    install -m 600 $_tmpfile $_confpath
}

start_daemon() {
    case "$os" in
        centos | fedora | debian | ubuntu)
            if $(stat --format=%N /proc/1/exe|grep -qs systemd); then
                systemctl start inagent
            else
                service inagent start
            fi
            ;;

        freebsd)
            printf '\ninagent_enable="YES"\n' >> /etc/rc.conf
            service inagent start
            ;;

        darwin)
            nohup /usr/local/bin/inagent &
            ;;

        *)
            echo "unrecognized OS type: $os" >&2
            exit 1
            ;;
    esac
}

do_uninstall() {
	rm -f $prefix/bin/inagent \
		  $sysconfdir/intecture/agent.json \
		  /lib/systemd/system/inagent.service \
		  $prefix/lib/systemd/system/inagent.service \
		  $sysconfdir/init.d/inagent \
		  $sysconfdir/rc.d/inagent

    if [ -d $sysconfdir/intecture -a -z "$(ls -A $sysconfdir/intecture)" ]; then
        rmdir "$sysconfdir/intecture"
    fi
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        echo "need '$1' (command not found)" >&2
        exit 1
    fi
}

usage() {
    echo "Usage:
    installer.sh <install|uninstall>
    installer.sh install_certs <agent_prikey_path> <auth_pubkey_path>
    installer.sh amend_conf <key> <value>
    installer.sh start_daemon"
}

main() {
	if [ $# -eq 0 ]; then
		usage
		exit 0
	fi

	case "$1" in
		install)
			do_install
			;;

		uninstall)
			do_uninstall
			;;

        install_certs)
            if [ -z $2 -o -z $3 ]; then
                usage
                exit 1
            fi

            install_certs $2 $3
            ;;

        amend_conf)
            if [ -z $2 -o -z $3 ]; then
                usage
                exit 1
            fi

            amend_conf $2 $3
            ;;

        start_daemon)
            start_daemon
            ;;

		*)
			echo "Unknown option $1"
			exit 1
			;;
	esac
}

main "$@"
