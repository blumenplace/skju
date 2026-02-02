FROM ubuntu:24.04 as RENODE_BASE

ARG RENODE_VERSION=1.16.0
ARG RENODE_PKG=linux-portable-dotnet

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

RUN apt-get update && apt-get install -y \
  wget \
  curl \
  git \
  policykit-1 \
  uml-utilities \
  gdb-multiarch \
  python3 \
  python3-pip \
  dotnet8 \
  screen \
  xvfb \
  x11vnc && \
  rm -rf /var/lib/apt/lists/*

RUN mkdir -p /opt/renode
RUN wget -q https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.${RENODE_PKG}.tar.gz && \
    tar -xzf renode-${RENODE_VERSION}.${RENODE_PKG}.tar.gz -C /opt/renode --strip-components=1 && \
    rm renode-${RENODE_VERSION}.${RENODE_PKG}.tar.gz


FROM ubuntu:24.04 as BUILDER

ARG LLVM_VERSION=15

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

RUN apt-get update && apt-get install -y \
  wget curl git policykit-1\
  cmake pkg-config autoconf libtool texinfo libssl-dev \
  dotnet8 \
  gfortran llvm-${LLVM_VERSION} clang-${LLVM_VERSION} libclang1-${LLVM_VERSION} && \
  rm -rf /var/lib/apt/lists/*

RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-${LLVM_VERSION} 1
RUN update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-${LLVM_VERSION} 1

COPY ./rust-toolchain.toml /tmp/rust-toolchain.toml

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $(cat ./tmp/rust-toolchain.toml | grep "channel" | cut -f2 -d\")

ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
RUN cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.10.0+v0.29.4


WORKDIR /build

COPY . .
RUN make all BUILD=debug
RUN mkdir -p out
RUN cp ./target/debug/libdefmt_cs.so ./out/libdefmt_cs.so
RUN cd ./bindings && dotnet build -c Release -o ./../out


FROM RENODE_BASE

ENV TZ=UTC
ENV PATH=/opt/renode:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV LD_LIBRARY_PATH=/opt/renode:$LD_LIBRARY_PATH

COPY --from=BUILDER /build/out/libdefmt_cs.so /opt/renode/libdefmt_cs.so
COPY --from=BUILDER /build/out/DefmtBindings.dll /opt/renode/DefmtBindings.dll
COPY --from=BUILDER /build/out/DefmtBindings.pdb /opt/renode/DefmtBindings.pdb
COPY --from=BUILDER /build/scripts/DefmtPlugin.cs /opt/renode/scripts/blumenplace/DefmtPlugin.cs

WORKDIR /workspace

EXPOSE 3333
EXPOSE 5900

CMD ["/bin/bash"]
