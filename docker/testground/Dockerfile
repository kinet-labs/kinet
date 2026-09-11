FROM rust:1-buster AS builder

WORKDIR /usr/src/kinet
RUN apt update
RUN apt install -y python3 clang

ARG GIT_COMMIT=""
ARG GIT_BRANCH=""
ARG GIT_TAG=""
ARG GIT_MODIFIED="false"

ENV GIT_COMMIT=$GIT_COMMIT
ENV GIT_BRANCH=$GIT_BRANCH
ENV GIT_TAG=$GIT_TAG
ENV GIT_MODIFIED=$GIT_MODIFIED

# Builder
COPY . .
RUN cargo build --release --bin kinet-testground && \
    mv target/release/kinet-testground testground
RUN python3 docker/testground/topology-gen.py topo.json
RUN python3 kinet-scripts/tc/tc-gen.py topo.json tc.sh addresses

# Runner
FROM debian:buster-slim
SHELL ["/bin/bash", "-c"]
WORKDIR /usr/src/kinet

RUN apt update
RUN apt install -y iproute2
RUN apt clean
COPY --from=builder /usr/src/kinet/testground /usr/local/bin/kinet-testground
COPY --from=builder /usr/src/kinet/tc.sh .
COPY --from=builder /usr/src/kinet/addresses .

ENV RUST_LOG=kinet_testground=DEBUG
CMD source tc.sh && KINET_MEMPOOL_RNDUDS=true kinet-testground -o http://jaeger:4317 --addresses $(<addresses)
