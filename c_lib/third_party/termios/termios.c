
/* SPDX-License-Identifier: MIT */

#include "termios.h"
#include <errno.h>

#include "../../wasi_ext_lib.h"

speed_t wasi_ext_cfgetospeed (const struct termios * termios_p)
{
    return -ENOTSUP;
}

speed_t wasi_ext_cfgetispeed (const struct termios * termios_p)
{
    return -ENOTSUP;
}

int wasi_ext_cfsetospeed (struct termios * termios_p, speed_t speed)
{
    return -ENOTSUP;
}

int wasi_ext_cfsetispeed (struct termios * termios_p, speed_t speed)
{
    return -ENOTSUP;
}

int wasi_ext_tcgetattr(int fd, struct termios *tio)
{
    return -wasi_ext_ioctl(fd, TCGETS, tio);
}

int wasi_ext_tcsetattr(int fd, int act, const struct termios *tio)
{
    unsigned int cmd;
    switch (act)
    {
    case TCSANOW: {
        cmd = TCSETS;
        break;
    }
    case TCSADRAIN:
    case TCSAFLUSH: {
        return -ENOTSUP;
        break;
    }
    default: {
        return -EINVAL;
    }
    }

    return -wasi_ext_ioctl(fd, cmd, (void*)tio);
}

int wasi_ext_tcgetwinsize (int fd, struct winsize * winsize_p)
{
    return -ENOTSUP;
}

int wasi_ext_tcsetwinsize (int fd, const struct winsize * winsize_p)
{
    return -ENOTSUP;
}

int wasi_ext_tcsendbreak (int fd, int duration)
{
    return -ENOTSUP;
}

int wasi_ext_tcdrain (int fd)
{
    return -ENOTSUP;
}

int wasi_ext_tcflush (int fd, int duration)
{
    return -ENOTSUP;
}

int wasi_ext_tcflow (int fd, int action)
{
    return -ENOTSUP;
}

pid_t wasi_ext_tcgetsid (int fd)
{
    return -ENOTSUP;
}

void wasi_ext_cfmakeraw(struct termios * termios_p)
{
    termios_p->c_iflag &= ~(IGNBRK|BRKINT|PARMRK|ISTRIP|INLCR|IGNCR|ICRNL|IXON);
    termios_p->c_oflag &= ~OPOST;
    termios_p->c_lflag &= ~(ECHO|ECHONL|ICANON|ISIG|IEXTEN);
    termios_p->c_cflag &= ~(CSIZE|PARENB);
    termios_p->c_cflag |= CS8;

    return;
}

int wasi_ext_cfsetspeed(struct termios * termios_p, speed_t speed)
{
    return -ENOTSUP;
}