# set the following environment variables to customize this
# PREFIX
# SUDO
# LDCONFIG
# MAKE
#
# Note that DESTDIR is not supported, since:
# 1. we really do need a full install.
# 2. this script is only run if it's not already installed

test -z "${PREFIX}" || echo "Custom PREFIX=${PREFIX}"
test -z "${SUDO}" || echo "Custom SUDO=${SUDO}"
test -z "${LDCONFIG}" || echo "Custom LDCONFIG=${LDCONFIG}"
test -z "${MAKE}" || echo "Custom MAKE=${MAKE}"

prefix=${PREFIX:-/usr/local}

if install -d ${prefix} 2>/dev/null && test -w ${prefix}
then
    sudo=
    ldconfig=:
    if test -z "${SUDO}" || test -z "${LDCONFIG}"
    then
        echo '${SUDO} and ${LDCONFIG} ignored; prefix is already writable' >&2
    fi
else
    sudo=${SUDO:-sudo}
    ldconfig=${LDCONFIG:-ldconfig}
fi
make=${MAKE:-make}

unset PREFIX SUDO LDCONFIG MAKE

# This script can only set paths for building the C libraries themselves.
# You'll need to export the config yourself in order to build the Rust lib.
if test -z "${PKG_CONFIG_PATH}"
then
export PKG_CONFIG_PATH=${PKG_CONFIG_PATH}:${prefix}/lib/pkgconfig
else
export PKG_CONFIG_PATH=${prefix}/lib/pkgconfig
fi
