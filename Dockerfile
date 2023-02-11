FROM ubuntu:18.04 as ubuntu18builder

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
ENV WASMTIME_VERSION=v5.0.0
RUN apt-get update
RUN apt-get install -y curl make git gcc build-essential pkgconf libtool libsystemd-dev libprotobuf-c-dev libcap-dev libseccomp-dev libyajl-dev go-md2man libtool autoconf python3 automake xz-utils
RUN curl https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
RUN git clone --depth 1 --recursive https://github.com/containers/crun.git
WORKDIR /crun
RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmedge

RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmtime

WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b napi-libnode https://github.com/mmomtchev/node.git
WORKDIR /wasm_nodejs/node
RUN ./configure --shared
RUN make
WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b node-wasm-experiment https://github.com/mhdawson/crun.git
WORKDIR /wasm_nodejs/crun
RUN cp /wasm_nodejs/node/out/Release/libnode.so.*  /lib64/libnode.so
RUN cp /wasm_nodejs/node/src/js_native_api.h /usr/include/js_native_api.h
RUN cp /wasm_nodejs/node/src/js_native_api_types.h /usr/include/js_native_api_types.h
RUN cp /wasm_nodejs/node/src/node_api.h /usr/include/node_api.h
RUN cp /wasm_nodejs/node/src/node_api_types.h /usr/include/node_api_types.h
RUN ./autogen.sh
RUN ./configure --with-wasm_nodejs --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasm-nodejs

FROM ubuntu:20.04 as ubuntu20builder

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
ENV WASMTIME_VERSION=v5.0.0
RUN apt update --fix-missing
RUN apt-get install -y curl make git gcc build-essential pkgconf libtool libsystemd-dev libprotobuf-c-dev libcap-dev libseccomp-dev libyajl-dev go-md2man libtool autoconf python3 automake xz-utils
RUN curl https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
RUN git clone --depth 1 --recursive https://github.com/containers/crun.git
WORKDIR /crun
RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmedge

RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmtime

WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b napi-libnode https://github.com/mmomtchev/node.git
WORKDIR /wasm_nodejs/node
RUN ./configure --shared
RUN make
WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b node-wasm-experiment https://github.com/mhdawson/crun.git
WORKDIR /wasm_nodejs/crun
RUN cp /wasm_nodejs/node/out/Release/libnode.so.*  /lib64/libnode.so
RUN cp /wasm_nodejs/node/src/js_native_api.h /usr/include/js_native_api.h
RUN cp /wasm_nodejs/node/src/js_native_api_types.h /usr/include/js_native_api_types.h
RUN cp /wasm_nodejs/node/src/node_api.h /usr/include/node_api.h
RUN cp /wasm_nodejs/node/src/node_api_types.h /usr/include/node_api_types.h
RUN ./autogen.sh
RUN ./configure --with-wasm_nodejs --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasm-nodejs

FROM docker.io/rockylinux/rockylinux:8 as rhel8builder
ENV WASMTIME_VERSION=v5.0.0
RUN dnf update -y
RUN dnf install -y dnf-plugins-core
RUN dnf config-manager --set-enabled plus
RUN dnf config-manager --set-enabled devel
RUN dnf config-manager --set-enabled powertools
RUN dnf clean all
RUN dnf update -y
RUN dnf repolist --all
RUN dnf -y install epel-release

RUN dnf install -y git python3 which redhat-lsb-core
RUN curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all -p /usr/local --version=0.11.2
RUN dnf install -y systemd-devel yajl-devel libseccomp-devel pkg-config libgcrypt-devel \
    glibc-static python3-libmount libtool libcap-devel
WORKDIR "/"
RUN git clone --depth 1 --recursive https://github.com/containers/crun.git
WORKDIR /crun
RUN ./autogen.sh
RUN ./configure --with-wasmedge --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmedge
### wasmtime
RUN curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api.tar.xz | tar xJf - -C /
RUN cp -R /wasmtime-${WASMTIME_VERSION}-$(uname -m)-linux-c-api/* /usr/local/
WORKDIR /crun
RUN ./configure --with-wasmtime --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasmtime

### wasm_nodejs doesn't use the default crun so we are creating subfolders
WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b napi-libnode https://github.com/mmomtchev/node.git
WORKDIR /wasm_nodejs/node
RUN ./configure --shared
RUN make
WORKDIR /wasm_nodejs
RUN git clone --depth 1 -b node-wasm-experiment https://github.com/mhdawson/crun.git
WORKDIR /wasm_nodejs/crun
RUN cp /wasm_nodejs/node/out/Release/libnode.so.*  /lib64/libnode.so
RUN cp /wasm_nodejs/node/src/js_native_api.h /usr/include/js_native_api.h
RUN cp /wasm_nodejs/node/src/js_native_api_types.h /usr/include/js_native_api_types.h
RUN cp /wasm_nodejs/node/src/node_api.h /usr/include/node_api.h
RUN cp /wasm_nodejs/node/src/node_api_types.h /usr/include/node_api_types.h
RUN ./autogen.sh
RUN ./configure --with-wasm_nodejs --enable-embedded-yajl
RUN make
RUN ./crun --version
RUN mv crun crun-wasm-nodejs

RUN yum install -y gcc openssl-devel && \
    rm -rf /var/cache/dnf && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

COPY manager /app-build/

WORKDIR "/app-build"

ENV PATH=/root/.cargo/bin:${PATH}

RUN cargo build --release

RUN cargo test --release

FROM registry.access.redhat.com/ubi8:8.7-1054.1675788412

WORKDIR "/vendor/rhel8"

COPY --from=rhel8builder /usr/local/lib/libwasmedge.so.0 /lib64/libnode.so /crun/crun-wasmedge /crun/crun-wasmtime /wasm_nodejs/crun/crun-wasm-nodejs ./

WORKDIR "/vendor/ubuntu_20_04"
COPY --from=ubuntu20builder /usr/local/lib/libwasmedge.so.0 /lib64/libnode.so /crun/crun-wasmedge /crun/crun-wasmtime /wasm_nodejs/crun/crun-wasm-nodejs ./

WORKDIR "/vendor/ubuntu_18_04"
COPY --from=ubuntu18builder /usr/local/lib/libwasmedge.so.0 /lib64/libnode.so /crun/crun-wasmedge /crun/crun-wasmtime /wasm_nodejs/crun/crun-wasm-nodejs ./

WORKDIR "/app"
COPY --from=rhel8builder /app-build/target/release/manager ./

RUN /app/manager version

CMD ["/app/manager"]
