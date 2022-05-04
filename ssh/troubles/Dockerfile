# https://troubles.noblogs.org/post/2020/06/26/openssh-keys-on-a-fido2-dongle/

FROM archlinux:latest

ARG uid=1000
ARG gid=1000
ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C
ENV LANGUAGE=C
ENV LC_ALL=C


RUN yes | pacman -Sy openssh \
    && sed -i -E 's/#(PermitRootLogin without-password)/\1/g' /etc/ssh/sshd_config \
    && sed -i 's/PermitRootLogin prohibit-password/PermitRootLogin without-password/g' /etc/ssh/sshd_config \
    && sed -i -E 's/#(PermitRootLogin without-password)/\1/g' /etc/ssh/sshd_config \
    && sed -i 's/#MaxAuthTries 6/MaxAuthTries 600/g' /etc/ssh/sshd_config \
    && mkdir /run/sshd \
    && groupadd -g ${gid} uzer \
    && useradd -u ${uid} -g ${gid} -m -d /uzer uzer \
    && mkdir /uzer/.ssh \
    && chown uzer:uzer /uzer/.ssh \
    && chmod 700 /uzer/.ssh
RUN ssh-keygen -A
# set passwd "asdasd" for user "uzer"
RUN echo "uzer:asdasd" | chpasswd
RUN echo "root:asdasd" | chpasswd
COPY --chown=uzer:uzer key.pub /uzer/.ssh/authorized_keys
COPY entrypoint /entrypoint
COPY works /works

EXPOSE 22

ENTRYPOINT ["/entrypoint"]
CMD [""]

