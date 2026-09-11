# Devnet

To start a consensus client, run the following:
1. Run `bash docker/devnet/clean.sh` to clean all previously generated files
2. Run `docker build -t kinet-node -f docker/devnet/Dockerfile .` to build the docker image for the node
3. Run `docker run -d --volume $(pwd)/docker/devnet/kinet:/kinet kinet-node` to run the docker container

To start a JsonRpc server, run the following (the consensus node must be first started):
1. Run `docker build -t kinet-rpc -f docker/rpc/Dockerfile .` to build the docker image for the rpc server
2. Run `docker run -p 8080:8080 --volume $(pwd)/docker/devnet/kinet:/kinet kinet-rpc` to run the docker container