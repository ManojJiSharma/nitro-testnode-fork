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

project_name="nitro-testnode-1"

echo == Building..
docker-compose --project-name="$project_name" build --no-rm scripts tokenbridge


docker pull $NITRO_NODE_VERSION
docker tag $NITRO_NODE_VERSION nitro-node-dev-testnode

docker-compose --project-name="$project_name" build --no-rm $NODES scripts


echo == Removing old data..
docker-compose --project-name="$project_name" down
leftoverContainers=`docker container ls -a --filter label=com.docker.compose.project=nitro-testnode-1 -q | xargs echo`
if [ `echo $leftoverContainers | wc -w` -gt 0 ]; then
    docker rm $leftoverContainers
fi
docker volume prune -f --filter label=com.docker.compose.project=nitro-testnode-1
leftoverVolumes=`docker volume ls --filter label=com.docker.compose.project=nitro-testnode-1 -q | xargs echo`
if [ `echo $leftoverVolumes | wc -w` -gt 0 ]; then
    docker volume rm $leftoverVolumes
fi

echo == Generating l1 keys
docker-compose --project-name="$project_name" run scripts write-accounts
docker-compose --project-name="$project_name" run --entrypoint sh geth -c "echo passphrase > /datadir/passphrase"
docker-compose --project-name="$project_name" run --entrypoint sh geth -c "chown -R 1000:1000 /keystore"
docker-compose --project-name="$project_name" run --entrypoint sh geth -c "chown -R 1000:1000 /config"

docker-compose --project-name="$project_name" up -d geth

echo == Funding validator and sequencer
docker-compose --project-name="$project_name" run scripts send-l1 --ethamount 1000 --to validator --wait
docker-compose --project-name="$project_name" run scripts send-l1 --ethamount 1000 --to sequencer --wait

echo == create l1 traffic
docker-compose --project-name="$project_name" run scripts send-l1 --ethamount 1000 --to user_l1user --wait

echo == Writing l2 chain config
docker-compose --project-name="$project_name" run scripts write-l2-chain-config

echo == Deploying L2
sequenceraddress=`docker-compose --project-name="$project_name" run scripts print-address --account sequencer | tail -n 1 | tr -d '\r\n'`

docker-compose --project-name="$project_name" run --entrypoint /usr/local/bin/deploy poster --l1conn ws://geth:8546 --l1keystore /home/user/l1keystore --sequencerAddress $sequenceraddress --ownerAddress $sequenceraddress --l1DeployAccount $sequenceraddress --l1deployment /config/deployment.json --authorizevalidators 10 --wasmrootpath /home/user/target/machines --l1chainid=$l1chainid --l2chainconfig /config/l2_chain_config.json --l2chainname arb-dev-test --l2chaininfo /config/deployed_chain_info.json
docker-compose --project-name="$project_name" run --entrypoint sh poster -c "jq [.[]] /config/deployed_chain_info.json > /config/l2_chain_info.json"
echo == Writing configs
docker-compose --project-name="$project_name" run scripts write-config

echo == Initializing redis
docker-compose --project-name="$project_name" run scripts redis-init --redundancy 0

echo == Funding l2 funnel
docker-compose --project-name="$project_name" up -d sequencer
docker-compose --project-name="$project_name" run scripts bridge-funds --ethamount 100000 --wait

echo == Deploying token bridge
docker-compose --project-name="$project_name" run -e ARB_KEY=$devprivkey -e ETH_KEY=$devprivkey tokenbridge gen:network
docker-compose --project-name="$project_name" run --entrypoint sh tokenbridge -c "cat localNetwork.json"
echo

UP_FLAG="-d"

echo == Launching Sequencer
docker-compose --project-name="$project_name" up  $UP_FLAG $NODES
