#!/usr/bin/env bash

set -e

NITRO_NODE_VERSION=offchainlabs/nitro-node:v2.1.1-e9d8842-dev
BLOCKSCOUT_VERSION=offchainlabs/blockscout:v1.0.0-c8db5b1

# mydir=`dirname $0`
# cd "$mydir"

run=true
devprivkey=b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
l1chainid=1337

NODES="sequencer poster staker-unsafe"


echo == Building..
docker-compose build --no-rm scripts tokenbridge


docker pull $NITRO_NODE_VERSION
docker tag $NITRO_NODE_VERSION nitro-node-dev-testnode

docker-compose build --no-rm $NODES scripts


echo == Removing old data..
docker-compose down
leftoverContainers=`docker container ls -a --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo`
if [ `echo $leftoverContainers | wc -w` -gt 0 ]; then
    docker rm $leftoverContainers
fi
docker volume prune -f --filter label=com.docker.compose.project=nitro-testnode
leftoverVolumes=`docker volume ls --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo`
if [ `echo $leftoverVolumes | wc -w` -gt 0 ]; then
    docker volume rm $leftoverVolumes
fi

echo == Generating l1 keys
docker-compose run scripts write-accounts
docker-compose run --entrypoint sh geth -c "echo passphrase > /datadir/passphrase"
docker-compose run --entrypoint sh geth -c "chown -R 1000:1000 /keystore"
docker-compose run --entrypoint sh geth -c "chown -R 1000:1000 /config"

docker-compose up -d geth

echo == Funding validator and sequencer
docker-compose run scripts send-l1 --ethamount 1000 --to validator --wait
docker-compose run scripts send-l1 --ethamount 1000 --to sequencer --wait

echo == create l1 traffic
docker-compose run scripts send-l1 --ethamount 1000 --to user_l1user --wait

echo == Writing l2 chain config
docker-compose run scripts write-l2-chain-config

echo == Deploying L2
sequenceraddress=`docker-compose run scripts print-address --account sequencer | tail -n 1 | tr -d '\r\n'`

docker-compose run --entrypoint /usr/local/bin/deploy poster --l1conn ws://geth:8546 --l1keystore /home/user/l1keystore --sequencerAddress $sequenceraddress --ownerAddress $sequenceraddress --l1DeployAccount $sequenceraddress --l1deployment /config/deployment.json --authorizevalidators 10 --wasmrootpath /home/user/target/machines --l1chainid=$l1chainid --l2chainconfig /config/l2_chain_config.json --l2chainname arb-dev-test --l2chaininfo /config/deployed_chain_info.json
docker-compose run --entrypoint sh poster -c "jq [.[]] /config/deployed_chain_info.json > /config/l2_chain_info.json"
echo == Writing configs
docker-compose run scripts write-config

echo == Initializing redis
docker-compose run scripts redis-init --redundancy 0

echo == Funding l2 funnel
docker-compose up -d sequencer
docker-compose run scripts bridge-funds --ethamount 100000 --wait

echo == Deploying token bridge
docker-compose run -e ARB_KEY=$devprivkey -e ETH_KEY=$devprivkey tokenbridge gen:network
docker-compose run --entrypoint sh tokenbridge -c "cat localNetwork.json"
echo

UP_FLAG="-d"

echo == Launching Sequencer
docker-compose up  $UP_FLAG $NODES
