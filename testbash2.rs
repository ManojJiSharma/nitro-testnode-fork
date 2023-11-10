use std::process::Command;

fn main() {
    let nitro_node_version = "offchainlabs/nitro-node:v2.1.1-e9d8842-dev";

    let devprivkey = "b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
    let l1chainid = 1337;

    let nodes = "sequencer poster staker-unsafe";

    //Building
    println!("Building..");
    let build_command = Command::new("sh")
        .arg("-c")
        .arg("docker-compose build --no-rm scripts tokenbridge")
        .output()
        .expect("Failed to run command");

    let pull_command = format!("docker pull {}", nitro_node_version);
    Command::new("sh")
        .arg("-c")
        .arg(&pull_command)
        .output()
        .expect("Failed to run command");

    let tag_command = format!("docker tag {} nitro-node-dev-testnode", nitro_node_version);
    Command::new("sh")
        .arg("-c")
        .arg(&tag_command)
        .output()
        .expect("Failed to run command");

    let build_command = format!("docker-compose build --no-rm {} scripts", nodes);
    Command::new("sh")
        .arg("-c")
        .arg(&build_command)
        .output()
        .expect("Failed to run command");

    println!("Removing old data..");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose down")
        .output()
        .expect("Failed to run command");

    let leftover_containers = Command::new("sh")
        .arg("-c")
        .arg("docker container ls -a --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo")
        .output()
        .expect("Failed to run command");
    let container_ids = String::from_utf8(leftover_containers.stdout).expect("Invalid UTF-8");
    //Remove containers listed on container_ids
    Command::new("docker")
        .arg("rm")
        .args(container_ids.trim().split_whitespace()) // Split the IDs into separate arguments
        .output()
        .expect("Failed to run command");

    // prune Docker volumes with a specific label in the 'nitro-testnode' project
    Command::new("sh")
        .arg("-c")
        .arg("docker volume prune -f --filter label=com.docker.compose.project=nitro-testnode")
        .output()
        .expect("Failed to run command");

    //list Docker volumes with a specific label in the 'nitro-testnode' project
    let leftover_volumes = Command::new("sh")
        .arg("-c")
        .arg("docker volume ls --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo")
        .output()
        .expect("Failed to run command");
    // Extract the container IDs as a string
    let container_vol_ids = String::from_utf8(leftover_volumes.stdout).expect("Invalid UTF-8");
    //remove Docker volumes by their IDs
    let remove_leftover_vol_containers = Command::new("docker")
        .arg("rm")
        .args(container_vol_ids.trim().split_whitespace())
        .output()
        .expect("Failed to run command");

    // Generation l1 keys
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-accounts")
        .output()
        .expect("Failed to run command");
    //run Docker Compose with a custom entrypoint to write a passphrase file
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh geth -c \"echo passphrase > /datadir/passphrase\"")
        .output()
        .expect("Failed to run command");
    //run Docker Compose with a custom entrypoint to change ownership of the '/keystore' directory
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh geth -c \"chown -R 1000:1000 /keystore\"")
        .output()
        .expect("Failed to run command");
    // run Docker Compose with a custom entrypoint to change ownership of the '/config' directory
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh geth -c \"chown -R 1000:1000 /config\"")
        .output()
        .expect("Failed to run command");
    // run Docker Compose in detached mode to start the 'geth' service
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose up -d geth")
        .output()
        .expect("Failed to run command");

    //Funding validator and sequencer
    println!("Funding validator and sequencer");
    Command::new("sh")
        .arg("-c")
        .arg(" docker-compose run scripts send-l1 --ethamount 1000 --to validator --wait")
        .output()
        .expect("Failed to run command");

    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts send-l1 --ethamount 1000 --to sequencer --wait")
        .output()
        .expect("Failed to run command");

    //create l1 traffic
    println!("create l1 traffic");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts send-l1 --ethamount 1000 --to user_l1user --wait")
        .output()
        .expect("Failed to run command");

    //Writing l2 chain config
    println!(" Writing l2 chain config");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-l2-chain-config")
        .output()
        .expect("Failed to run command");

    // getting sequencer address for Deploying L2
    let sequenceraddress = Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-l2-chain-config")
        .output()
        .expect("Failed to run command");
    let sequenceraddress = String::from_utf8_lossy(&sequenceraddress.stdout);
    //Append sequenceraddress to command
    let entrypoint_cmd = format!("docker-compose run --entrypoint /usr/local/bin/deploy poster --l1conn ws://geth:8546 --l1keystore /home/user/l1keystore --sequencerAddress {0} --ownerAddress {0} --l1DeployAccount {0} --l1deployment /config/deployment.json --authorizevalidators 10 --wasmrootpath /home/user/target/machines --l1chainid=$l1chainid --l2chainconfig /config/l2_chain_config.json --l2chainname arb-dev-test --l2chaininfo /config/deployed_chain_info.json",sequenceraddress);
    Command::new("sh")
        .arg("-c")
        .arg(&entrypoint_cmd)
        .output()
        .expect("Failed to run command");

    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh poster -c \"jq [.[]] /config/deployed_chain_info.json > /config/l2_chain_info.json\"")
        .output()
        .expect("Failed to run command");

    //execute a Docker Compose command to run the 'scripts' service to perform 'write-config' operation.
    println!("Writing configs");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-config")
        .output()
        .expect("Failed to run command");

    // Initializing redis
    println!("Initializing redis");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts redis-init --redundancy 0")
        .output()
        .expect("Failed to run command");

    // start the 'sequencer' service in detached mode using Docker Compose.
    println!("Funding l2 funnel");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose up -d sequencer")
        .output()
        .expect("Failed to run command");
    // run the 'scripts' service to bridge funds with specified parameters.
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts bridge-funds --ethamount 100000 --wait")
        .output()
        .expect("Failed to run command");

    // Deploying token bridge
    // Docker Compose command to run the 'tokenbridge' service with environment variables for ARB_KEY and ETH_KEY.
    // The command generates a network configuration for the token bridge.
    println!("Deploying token bridge");
    let append_devprivkey_in_cmd = format!(
        "docker-compose run -e ARB_KEY= {0} -e ETH_KEY= {0} tokenbridge gen:network",
        devprivkey
    );
    Command::new("sh")
        .arg("-c")
        .arg(&append_devprivkey_in_cmd)
        .output()
        .expect("Failed to run command");

    // run the 'tokenbridge' service with a custom entrypoint.
    // Inside the service, execute a shell command to display the content of the 'localNetwork.json' file.
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh tokenbridge -c \"cat localNetwork.json\"")
        .output()
        .expect("Failed to run command");

    //Launching Sequencer
    println!("Launching Sequencer");
    let format_lauching_seq_cmd = format!(" docker-compose up {}", nodes);
    Command::new("sh")
        .arg("-c")
        .arg(&format_lauching_seq_cmd)
        .output()
        .expect("Failed to run command");
}
