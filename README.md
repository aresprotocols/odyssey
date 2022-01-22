
# Run local testnet with `Relaychain` and `Parachain`
Build Relaychain and Parachain local testnet to develop.

```bash
cd launch

# install dependencies
yarn

# generate docker-compose.yml and genesis
# NOTE: If the docker image is not the latest, need to download it manually.
# e.g.: docker pull acala/karura-node:latest
# NOTE: If no mars imags, need to build it first.
# e.g : cp builder.Dockerfile to project root and rename Dockerfile && `docker build -t mars .`
yarn run start generate

# start relaychain and parachain
cd output
# NOTE: If regenerate the output directory, need to rebuild the images.
`docker-compose up -d --build` or `docker-compose up --build`

# list all of the containers.
docker ps -a

# track container logs
docker logs -f [container_id/container_name]

# stop all of the containers.
docker-compose stop

# remove all of the containers.
docker-compose rm

# NOTE: If you want to clear the data and restart, you need to clear the volumes.
# remove volume
docker volume ls
# e.g : docker volume rm output_parachain-2000-0 output_parachain-2000-1 output_relaychain-alice output_relaychain-bob
# e.g : docker rmi output_parachain-2000-0 output_parachain-2000-1 output_relaychain-alice output_relaychain-bob
docker volume rm [volume_name]
# prune all volumes
docker volume prune
```

