FROM docker.io/paritytech/ci-linux:production as builder
WORKDIR /substrate
COPY . /substrate
RUN cargo build --locked --release
RUN pwd
RUN ls /substrate
RUN ls /substrate/target
RUN ls /substrate/target/release

FROM docker.io/library/ubuntu:20.04

COPY --from=builder /substrate/target/release/ares-collator /usr/local/bin
WORKDIR /usr/local/bin

RUN apt-get update && \
apt-get install ca-certificates -y && \
update-ca-certificates && \
mkdir -p /root/.local/share/ares-collator  && \
ln -s /root/.local/share/ares-collator /data && \
/usr/local/bin/ares-collator --version

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
