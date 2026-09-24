// Copyright 2019 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-ChromiumOS file.

#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <pwd.h>
#include <signal.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <wasi/libc.h>
#include <wasi_ext_lib.h>
#include <third_party/termios/termios.h>

// Helper functions

__wasi_fd_t fd_dup(__wasi_fd_t oldfd) {
  int newfd = open("/dev/null", O_RDWR);
  if (newfd < 0) {
    return -1;
  }

  __wasi_errno_t error = __wasi_fd_renumber(oldfd, newfd);
  if (error != 0) {
    errno = error;
    return -1;
  }

  return newfd;
}

__wasi_fd_t fd_dup2(__wasi_fd_t oldfd, __wasi_fd_t newfd) {
  if (oldfd == newfd) {
    return newfd;
  }

  __wasi_errno_t error = __wasi_fd_renumber(oldfd, newfd);
  if (error != 0) {
    errno = error;
    return -1;
  }

  return newfd;
}

char* _readpassphrase(const char *prompt, char *buf, size_t buf_len, bool echo) {
  struct termios old_t_in;
  struct termios new_t_in;
  bool has_termios = (wasi_ext_tcgetattr(STDIN_FILENO, &old_t_in) == 0);

  if (has_termios) {
    new_t_in = old_t_in;
    if (echo) {
      new_t_in.c_lflag |= (ECHO | ECHOE | ECHOK);
    } else {
      new_t_in.c_lflag &= ~(ECHO | ECHOE | ECHOK | ECHONL);
    }

    new_t_in.c_lflag &= ~(ISIG | ICANON);

    wasi_ext_tcsetattr(STDIN_FILENO, TCSANOW, &new_t_in);
  }

  size_t i = 0;
  char c;
  int saved_errno = 0;
  bool eof = false;

  while (i < buf_len - 1) {
    ssize_t bytes_read = read(STDIN_FILENO, &c, 1);
    if (bytes_read < 0) {
      if (errno == EINTR)
        continue;
      saved_errno = errno;
      break;
    }
    if (bytes_read == 0) {
      eof = true;
      break;
    }

    // Handle Ctrl+C (\x03) for interrupt
    if (c == '\x03') {
      saved_errno = EINTR;
      break;
    }

    // Handle Ctrl+D (\x04) for EOF
    if (c == '\x04') {
        eof = true;
        break;
    }

    // Handle enter
    if (c == '\n' || c == '\r') {
      break;
    }

    // Handle backspace explicitly
    if (c == '\b' || c == 0x7F) {
      if (i > 0) {
        i--;
      }
      continue;
    }

    buf[i++] = c;
  }

  buf[i] = '\0';

  if (has_termios) {
    wasi_ext_tcsetattr(STDIN_FILENO, TCSANOW, &old_t_in);
  }

  if (saved_errno != 0 || (eof && i == 0)) {
    explicit_bzero(buf, buf_len); // Clean memory on failure
    errno = saved_errno ? saved_errno : EIO;
    return NULL;
  }

  return buf;
}

int sock_accept(__wasi_fd_t sock, __wasi_fd_t* newsock) {
  errno = ENOSYS;
  return -1;
}

int sock_bind(__wasi_fd_t sock,
              int domain,
              const uint8_t* addr,
              uint16_t port) {
  errno = ENOSYS;
  return -1;
}

int sock_listen(__wasi_fd_t sock, int backlog) {
  errno = ENOSYS;
  return -1;
}

int sock_sendto(__wasi_fd_t sock,
                const void* buf,
                size_t len,
                size_t* written,
                int flags,
                int domain,
                const uint8_t* addr,
                uint16_t port) {
  return write(sock, buf, len);
}

int sock_connect(__wasi_fd_t sock, int domain, const uint8_t *addr,
                 uint16_t port) {
    errno = ENOSYS;
    return -1;
}

int sock_create(int domain, int type, int protocol) {
  return open("/dev/null", O_RDWR);
}

// Sockets

int accept(int sockfd, struct sockaddr* addr, socklen_t* addrlen) {
  int newsock;
  int ret = sock_accept(sockfd, &newsock);
  if (ret < 0)
    return ret;

  ret = newsock;

  if (addrlen) {
    *addrlen = 0;
  }

  return ret;
}

int bind(int sock, const struct sockaddr* addr, socklen_t addrlen) {

  // Only support IPv4 & IPv6.
  int sys_domain = addr->sa_family;
  const uint8_t* sys_addr;
  uint16_t sys_port;
  switch (addr->sa_family) {
    case AF_INET: {
      const struct sockaddr_in* sin = (void*)addr;
      sys_addr = (const uint8_t*)&sin->sin_addr.s_addr;
      sys_port = ntohs(sin->sin_port);
      break;
    }

    case AF_INET6: {
      const struct sockaddr_in6* sin6 = (void*)addr;
      if (sin6->sin6_flowinfo) {
        errno = EINVAL;
        return -1;
      }
      // This would be nice to support.
      if (sin6->sin6_scope_id) {
        errno = EINVAL;
        return -1;
      }
      sys_addr = (const uint8_t*)&sin6->sin6_addr.s6_addr;
      sys_port = ntohs(sin6->sin6_port);
      break;
    }

    default:
      errno = EINVAL;
      return -1;
  }

  int ret = sock_bind(sock, sys_domain, sys_addr, sys_port);
  return ret;
}

int listen(int sock, int backlog) {
  int ret = sock_listen(sock, backlog);
  return ret;
}

int connect(int sock, const struct sockaddr* addr, socklen_t addrlen) {
  // Only support IPv4 & IPv6.
  int sys_domain = addr->sa_family;
  const uint8_t* sys_addr;
  uint16_t sys_port;
  switch (addr->sa_family) {
    case AF_INET: {
      const struct sockaddr_in* sin = (void*)addr;
      sys_addr = (const uint8_t*)&sin->sin_addr.s_addr;
      sys_port = ntohs(sin->sin_port);
      break;
    }

    case AF_INET6: {
      const struct sockaddr_in6* sin6 = (void*)addr;
      if (sin6->sin6_flowinfo) {
        errno = EINVAL;
        return -1;
      }
      // This would be nice to support.
      if (sin6->sin6_scope_id) {
        errno = EINVAL;
        return -1;
      }
      sys_addr = (const uint8_t*)&sin6->sin6_addr.s6_addr;
      sys_port = ntohs(sin6->sin6_port);
      break;
    }

    case AF_UNIX: {
      const struct sockaddr_un* sun = (void*)addr;
      sys_addr = (const uint8_t*)&sun->sun_path;
      sys_port = sizeof(sun->sun_path);
      break;
    }

    default:
      errno = EINVAL;
      return -1;
  }

  int ret = sock_connect(sock, sys_domain, sys_addr, sys_port);
  return ret;
}

int getsockopt(
    int sockfd, int level, int optname, void* optval, socklen_t* optlen) {

  if (*optlen != 4) {
    errno = EINVAL;
    return -1;
  }

  return 0;
}

ssize_t recvfrom(int sockfd,
                 void* buf,
                 size_t len,
                 int flags,
                 struct sockaddr* addr,
                 socklen_t* addrlen) {

  int domain;
  uint8_t s_addr[16];
  uint16_t port;
  size_t written;
  int ret = read(sockfd, buf, len);
  if (ret != 0 || addr == NULL) {
    goto done;
  }

  if (addr != NULL && addrlen == NULL) {
    errno = EINVAL;
    goto done;
  }

  switch (domain) {
    case AF_INET: {
      struct sockaddr_in* sin = (void*)addr;
      if (*addrlen < sizeof(*sin)) {
        errno = EINVAL;
        goto done;
      }
      *addrlen = sizeof(*sin);
      sin->sin_port = htons(port);
      memcpy(&sin->sin_addr.s_addr, s_addr, sizeof(sin->sin_addr.s_addr));
      break;
    }

    case AF_INET6: {
      struct sockaddr_in6* sin6 = (void*)addr;
      if (*addrlen < sizeof(*sin6)) {
        errno = EINVAL;
        goto done;
      }
      *addrlen = sizeof(*sin6);
      sin6->sin6_flowinfo = 0;
      // This would be nice to support.
      sin6->sin6_scope_id = 0;
      sin6->sin6_port = htons(port);
      memcpy(&sin6->sin6_addr.s6_addr, s_addr, sizeof(sin6->sin6_addr.s6_addr));
      break;
    }

    default:
      errno = EINVAL;
      goto done;
  }
  addr->sa_family = domain;

done:
  return ret ? (ssize_t)ret : (ssize_t)written;
}

ssize_t recv(int sockfd, void* buf, size_t len, int flags) {
  return recvfrom(sockfd, buf, len, flags, NULL, NULL);
}

ssize_t recvmsg(int sockfd, struct msghdr* msg, int flags) {

  if (msg->msg_iovlen != 1) {
    errno = EINVAL;
    return -1;
  }

  return recvfrom(sockfd, msg->msg_iov->iov_base, msg->msg_iov->iov_len, flags,
                  msg->msg_name, &msg->msg_namelen);
}

ssize_t sendto(int sockfd,
               const void* buf,
               size_t len,
               int flags,
               const struct sockaddr* addr,
               socklen_t addrlen) {

  // Only support IPv4 & IPv6.
  int ret = -1;
  size_t written = 0;
  int sys_domain = addr->sa_family;
  const uint8_t* sys_addr;
  uint16_t sys_port;
  switch (sys_domain) {
    case AF_INET: {
      const struct sockaddr_in* sin = (void*)addr;
      if (addrlen < sizeof(*sin)) {
        errno = EINVAL;
        goto done;
      }
      sys_addr = (const uint8_t*)&sin->sin_addr.s_addr;
      sys_port = ntohs(sin->sin_port);
      break;
    }

    case AF_INET6: {
      const struct sockaddr_in6* sin6 = (void*)addr;
      if (addrlen < sizeof(*sin6)) {
        errno = EINVAL;
        goto done;
      }
      if (sin6->sin6_flowinfo) {
        errno = EINVAL;
        goto done;
      }
      // This would be nice to support.
      if (sin6->sin6_scope_id) {
        errno = EINVAL;
        goto done;
      }
      sys_addr = (const uint8_t*)&sin6->sin6_addr.s6_addr;
      sys_port = ntohs(sin6->sin6_port);
      break;
    }

    default:
      errno = EINVAL;
      goto done;
  }

  ret = sock_sendto(sockfd, buf, len, &written, flags, sys_domain, sys_addr,
                    sys_port);
done:
  return ret ? (ssize_t)ret : (ssize_t)written;
}

ssize_t send(int sockfd, const void* buf, size_t len, int flags) {
  size_t written;
  int ret = sock_sendto(sockfd, buf, len, &written, flags, 0, NULL, 0);
  return ret ? (ssize_t)ret : (ssize_t)written;
}

int setsockopt(
    int sockfd, int level, int optname, const void* optval, socklen_t optlen) {

  if (optlen != 4) {
    errno = EINVAL;
    return -1;
  }

  int value;
  memcpy(&value, optval, 4);

  return 0;
}

int socket(int domain, int c_type, int protocol) {
  // Make sure preopens have processed before we create a socket as that will
  // update the file descriptor table, and preopen logic will fail when it hits
  // a non-preopen fd.
  __wasilibc_populate_preopens();

  // We don't support much here currently.
  // 0: The default for most things.
  // -1: Our fake delayed hostname logic from getaddrinfo.
  if (protocol != 0 && protocol != -1 &&
      !(c_type == SOCK_STREAM && protocol == IPPROTO_TCP) &&
      !(c_type == SOCK_DGRAM && protocol == IPPROTO_UDP)) {
    errno = EINVAL;
    return -1;
  }

  // Only support UNIX sockets (ssh-agent), IPv4, and IPv6.
  switch (domain) {
    case AF_UNIX:
    case AF_INET:
    case AF_INET6:
      break;
    default:
      errno = EINVAL;
      return -1;
  }

  // Maybe add these if anyone wants them.
  if (c_type & (SOCK_NONBLOCK | SOCK_CLOEXEC)) {
    errno = EINVAL;
    return -1;
  }

  // Only support TCP & UDP protocols.
  int sys_type;
  switch (c_type) {
    case SOCK_DGRAM:
    case SOCK_STREAM:
      sys_type = c_type;
      break;
    default:
      errno = EINVAL;
      return -1;
  }

  int ret = sock_create(domain, sys_type, protocol);
  return ret;
}

ssize_t sendmsg(int sockfd, const struct msghdr* msg, int flags) {
  errno = ENOSYS;
  return -1;
}

int socketpair(int domain, int type, int protocol, int sv[2]) {
  errno = ENOSYS;
  return -1;
}

// Fds

int dup(int oldfd) {
  int ret = fd_dup(oldfd);
  return ret;
}

int dup2(int oldfd, int newfd) {
  int ret = fd_dup2(oldfd, newfd);
  return ret;
}

int ioctl(int fd, int request, ...) {
  int ret = -1;
  va_list ap;
  va_start(ap, request);

  switch (request) {
    case TIOCGWINSZ: {
      // Get terminal window size.
      struct winsize* ws = va_arg(ap, struct winsize*);
      ret = wasi_ext_tcgetwinsize(fd, ws);
      if (ret < 0) {
        errno = -ret;
        ret = -1;
      }
      break;
    }

    case TIOCSWINSZ: {
      // Set terminal window size.
      const struct winsize* ws = va_arg(ap, const struct winsize*);
      ret = wasi_ext_tcsetwinsize(fd, ws);
      if (ret < 0) {
        errno = -ret;
        ret = -1;
      }
      break;
    }

    default: {
      void* arg = va_arg(ap, void*);
      ret = wasi_ext_ioctl(fd, request, arg);
      if (ret != 0) {
        errno = ret;
        ret = -1;
      }
      break;
    }
  }

  va_end(ap);
  return ret;
}

int pipe(int pipefd[2]) {
  errno = ENOSYS;
  return -1;
}

int closefrom(int fd) {
  errno = ENOSYS;
  return 0;
}

// Error logging

// No header defines this unfortunately.
extern const char* __progname;

void vwarn(const char* format, va_list args) {
  fprintf(stderr, "%s: ", __progname);
  if (format) {
    vfprintf(stderr, format, args);
    fputs(": ", stderr);
  }
  perror(NULL);
}

void vwarnx(const char* format, va_list args) {
  fprintf(stderr, "%s: ", __progname);
  if (format) {
    vfprintf(stderr, format, args);
  }
  fprintf(stderr, "\n");
}

void verr(int status, const char* format, va_list args) {
  vwarn(format, args);
  exit(status);
}

void verrx(int status, const char* format, va_list args) {
  vwarnx(format, args);
  exit(status);
}

void warn(const char* format, ...) {
  va_list args;

  va_start(args, format);
  vwarn(format, args);
  va_end(args);
}

void warnx(const char* format, ...) {
  va_list args;

  va_start(args, format);
  vwarnx(format, args);
  va_end(args);
}

void err(int status, const char* format, ...) {
  va_list args;

  va_start(args, format);
  verr(status, format, args);
  va_end(args);
}

void errx(int status, const char* format, ...) {
  va_list args;

  va_start(args, format);
  verrx(status, format, args);
  va_end(args);
}

// Prompt

#ifndef RPP_ECHO_ON
#define RPP_ECHO_ON 1
#endif

char *readpassphrase(const char *prompt, char *buf, size_t buf_len, int flags) {
  struct termios termios = {0};
  wasi_ext_tcgetattr(STDERR_FILENO, &termios);

  size_t old_len = strlen(prompt);
  char *new_prompt = NULL;
  const char *prompt_to_write = prompt;
  size_t prompt_len = old_len;

  if ((termios.c_oflag & OPOST) && (termios.c_oflag & ONLCR)) {
    new_prompt = malloc((old_len * 2) + 1);
    if (new_prompt) {
      size_t new_i = 0;
      for (size_t old_i = 0; old_i < old_len; old_i++) {
        char c = prompt[old_i];
        if (c == '\n') {
          new_prompt[new_i++] = '\r';
        }
        new_prompt[new_i++] = c;
      }
      new_prompt[new_i] = '\0';
      prompt_to_write = new_prompt;
      prompt_len = new_i;
    }
  }

  // Write prompt to stderr
  const char *p = prompt_to_write;
  size_t remaining = prompt_len;
  while (remaining > 0) {
    ssize_t written = write(STDERR_FILENO, p, remaining);
    if (written < 0) {
      if (errno == EINTR)
        continue;
      break;
    }
    p += written;
    remaining -= written;
  }

  if (new_prompt) {
    free(new_prompt);
  }

  char *ret = _readpassphrase(prompt_to_write, buf, buf_len,
                                   !!(flags & RPP_ECHO_ON));

  write(STDERR_FILENO, "\r\n", 2);

  return ret;
}

// Signal handling

void wasi_ext_signal_deliver(int signum) {
  if (signum < 0 || signum >= NSIG)
    return;

  // There is no API to just read the current signal handler. So we have to set
  // it to get the old value, use it, then restore it.
  int old_errno = errno;
  sighandler_t handler = signal(signum, SIG_IGN);
  errno = old_errno;
  if (handler == SIG_IGN)
    return;

  // Signals that have SIG_IGN as their default disposition don't register it as
  // such initially. So we have to handle it ourselves.
  if (handler == SIG_DFL &&
      (signum == SIGCHLD || signum == SIGURG || signum == SIGWINCH)) {
    goto done;
  }

  // SIG_ERR only happens with signals we can't catch.
  if (handler == SIG_DFL || handler == SIG_ERR) {
    errx(128 + signum, "Terminated by signal %i handler %p: %s", signum,
         handler, strsignal(signum));
  }

  // Call the custom handler.
  handler(signum);

done:
  // Restore the handler.
  signal(signum, handler);
}

#define sigmask(sig) (1 << ((sig) - 1))

int sigemptyset(sigset_t* set) {
  if (set == NULL) {
    errno = EINVAL;
    return -1;
  }

  *set = 0;
  return 0;
}

int sigfillset(sigset_t* set) {
  if (set == NULL) {
    errno = EINVAL;
    return -1;
  }

  *set = -1;
  return 0;
}

int sigaddset(sigset_t* set, int signum) {
  if (set == NULL) {
    errno = EINVAL;
    return -1;
  }

  *set |= sigmask(signum);
  return 0;
}

int sigdelset(sigset_t* set, int signum) {
  if (set == NULL) {
    errno = EINVAL;
    return -1;
  }

  *set &= ~sigmask(signum);
  return 0;
}

int sigismember(const sigset_t* set, int signum) {
  if (set == NULL) {
    errno = EINVAL;
    return -1;
  }

  return *set | sigmask(signum);
}

int sigaction(int signum,
              const struct sigaction* act,
              struct sigaction* oldact) {
  if (oldact)
    memset(oldact, 0, sizeof(*oldact));

  if (act->sa_flags & SA_SIGINFO)
    errx(1, "sigaction(%i): SA_SIGINFO not supported", signum);

  return signal(signum, act->sa_handler) == SIG_ERR ? -1 : 0;
}

// Network

struct servent* getservbyname(const char* name, const char* proto) {
  errno = ENOSYS;
  return NULL;
}
struct servent* getservbyport(int port, const char* proto) {
  errno = ENOSYS;
  return NULL;
}

int gethostname(char* name, size_t len) {
  strncpy(name, "localhost", len);
  return 0;
}

char* if_indextoname(unsigned ifindex, char* ifname) {
  errno = ENOSYS;
  return NULL;
}
unsigned if_nametoindex(const char* ifname) {
  errno = ENOSYS;
  return 1;
}

// Syslog

void openlog(const char* ident, int option, int facility) {}

void syslog(int priority, const char* format, ...) {
  va_list ap;
  fprintf(stderr, "syslog: ");
  va_start(ap, format);
  vprintf(format, ap);
  va_end(ap);
}

void closelog(void) {}

// Ptys

char* ptsname(int fd) {
  static char path[10];
  strncpy(path, "/dev/tty", sizeof(path));
  return path;
}

int openpty(int* amaster,
            int* aslave,
            char* name,
            const struct termios* termp,
            const struct winsize* winp) {
  errno = ENOENT;
  return -1;
}

// Passwd

struct passwd* getpwuid(uid_t uid) {
  static struct passwd pwd;
  struct passwd* result;
  getpwuid_r(uid, &pwd, NULL, 0, &result);
  return result;
}

int getpwuid_r(uid_t uid,
               struct passwd* pwd,
               char* buffer,
               size_t buflen,
               struct passwd** result) {
  const char* user = getenv("USER");
  const char* home = getenv("HOME");
  pwd->pw_name = user ? (char*)user : (char*)"";
  pwd->pw_passwd = (char*)"";
  pwd->pw_uid = 0;
  pwd->pw_gid = 0;
  pwd->pw_dir = home ? (char*)home : (char*)"";
  pwd->pw_shell = (char*)"";
  *result = pwd;
  return 0;
}

struct passwd* getpwnam(const char* name) {
  return getpwuid(0);
}

// Pids

pid_t getppid(void) {
  errno = ENOSYS;
  return 1;
}

pid_t getpgrp(void) {
  errno = ENOSYS;
  return 1;
}

pid_t waitpid(pid_t pid, int* status, int options) {
  errno = ENOSYS;
  return -1;
}

// Exec

int execv(const char* path, char* const argv[]) {
  errno = ENOSYS;
  return -1;
}
int execve(const char* path, char* const argv[], char* const envp[]) {
  errno = ENOSYS;
  return -1;
}
int execvp(const char* file, char* const argv[]) {
  errno = ENOSYS;
  return -1;
}
int system(const char* command) {
  errno = ENOSYS;
  return -1;
}
int execl(const char* path, const char* arg, ...) {
  errno = ENOSYS;
  return -1;
}
int execlp(const char* file, const char* arg, ...) {
  errno = ENOSYS;
  return -1;
}

// File permissions

mode_t umask(mode_t mask) {
  errno = ENOSYS;
  return 0;
}

int chown(const char* path, uid_t uid, gid_t gid) {
  errno = ENOSYS;
  return 0;
}

// C++ exception handling

// C++ exceptions are fatal and never caught. Which is OK if the codebase only
// throws exceptions to abort rather than dynamic recovery.
void* __cxa_allocate_exception(size_t thrown_size) {
  fprintf(stderr, "\r\nC++ (allocate) exceptions are disabled.\r\n");
  abort();
}
void __cxa_throw(void* thrown_exception, void* tinfo, void (*dest)(void*)) {
  fprintf(stderr, "\r\nC++ (throw) exceptions are disabled.\r\n");
  abort();
}

// Termios

speed_t cfgetispeed(const struct termios* termios_p) {
  return wasi_ext_cfgetispeed(termios_p);
}

speed_t cfgetospeed(const struct termios* termios_p) {
  return wasi_ext_cfgetospeed(termios_p);
}

int cfsetispeed(struct termios* termios_p, speed_t speed) {
  int err = wasi_ext_cfsetispeed(termios_p, speed);
  if (err < 0) {
    errno = -err;
    return -1;
  }
  return 0;
}

int cfsetospeed(struct termios* termios_p, speed_t speed) {
  int err = wasi_ext_cfsetospeed(termios_p, speed);
  if (err < 0) {
    errno = -err;
    return -1;
  }
  return 0;
}

int tcgetattr(int fd, struct termios* termios_p) {
  int err = wasi_ext_tcgetattr(fd, termios_p);
  if (err < 0) {
    errno = -err;
    return -1;
  }
  return 0;
}

int tcsetattr(int fd, int optional_actions, const struct termios* termios_p) {
  int err = wasi_ext_tcsetattr(fd, optional_actions, termios_p);
  if (err < 0) {
    errno = -err;
    return -1;
  }
  return 0;
}

void cfmakeraw(struct termios* termios_p) {
  wasi_ext_cfmakeraw(termios_p);
}

