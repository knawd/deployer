FROM ubuntu:18.04 as ubuntu18builder

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
ENV WASMTIME_VERSION=v5.0.0
RUN apt-get update
RUN apt-get install -y curl g++ make git gcc build-essential pkgconf libtool libsystemd-dev libprotobuf-c-dev libcap-dev libseccomp-dev libyajl-dev go-md2man libtool autoconf python3 automake xz-utils

ENV WASMTIME_VERSION=v5.0.0
RUN curl https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
WORKDIR /
RUN git clone --depth 1 -b 1.8 --recursive https://github.com/containers/crun.git
WORKDIR /crun

RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmedge

RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmtime

FROM ubuntu:20.04 as ubuntu20builder

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get dist-upgrade
RUN apt update --fix-missing
RUN apt-get install -y curl g++ make git gcc build-essential pkgconf libtool libsystemd-dev libprotobuf-c-dev libcap-dev libseccomp-dev libyajl-dev go-md2man libtool autoconf python3 automake xz-utils --fix-missing

ENV WASMTIME_VERSION=v5.0.0
RUN curl https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
WORKDIR /
RUN git clone --depth 1 -b 1.8 --recursive https://github.com/containers/crun.git
WORKDIR /crun
RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmedge

RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmtime

FROM docker.io/rockylinux/rockylinux:8 as rhel8builder
RUN dnf update -y
RUN dnf install -y dnf-plugins-core
RUN dnf config-manager --set-enabled plus
RUN dnf config-manager --set-enabled devel
RUN dnf config-manager --set-enabled powertools
RUN dnf clean all
RUN dnf update -y
RUN dnf repolist --all
RUN dnf -y install epel-release

RUN dnf install -y git python3 which redhat-lsb-core gcc-c++ gcc
ENV WASMTIME_VERSION=v5.0.0
RUN curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
RUN dnf install -y systemd-devel yajl-devel libseccomp-devel pkg-config libgcrypt-devel \
    glibc-static python3-libmount libtool libcap-devel
WORKDIR /
RUN git clone --depth 1 -b 1.8 --recursive https://github.com/containers/crun.git
WORKDIR /crun

RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmedge
### wasmtime
RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
# RUN ./crun --version
RUN mv crun crun-wasmtime

FROM registry.access.redhat.com/ubi9/ubi as ubi9build

RUN yum install -y gcc openssl-devel && \
    rm -rf /var/cache/dnf && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=/root/.cargo/bin:${PATH}

COPY manager /app-build/

WORKDIR "/app-build"

RUN cargo build --release

RUN cargo test --release

FROM registry.access.redhat.com/ubi9/ubi-minimal

RUN  microdnf update && microdnf install -y procps-ng

WORKDIR "/vendor/rhel8"

COPY --from=rhel8builder /usr/local/lib/libwasmedge.so.0 /usr/local/lib/libwasmtime.so /crun/crun-wasmedge /crun/crun-wasmtime ./

WORKDIR "/vendor/ubuntu_20_04"
COPY --from=ubuntu20builder /usr/local/lib/libwasmedge.so.0 /usr/local/lib/libwasmtime.so /crun/crun-wasmedge /crun/crun-wasmtime ./

WORKDIR "/vendor/ubuntu_18_04"
COPY --from=ubuntu18builder /usr/local/lib/libwasmedge.so.0 /usr/local/lib/libwasmtime.so /crun/crun-wasmedge /crun/crun-wasmtime ./

WORKDIR "/app"
COPY --from=ubi9build /app-build/target/release/manager ./

RUN /app/manager version

CMD ["/app/manager"]
