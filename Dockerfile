FROM rust:latest

ARG USER_ID
ARG GROUP_ID
RUN echo "root:root" | chpasswd
RUN groupadd -g $GROUP_ID payments
RUN useradd -m -r -u $USER_ID -g $GROUP_ID payments

WORKDIR /home/payments

RUN rustup toolchain install stable-x86_64
RUN rustup default stable-x86_64
RUN rustup component add rustfmt


USER payments
CMD ["/bin/bash"]
