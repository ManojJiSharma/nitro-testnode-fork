use std::process::Command;

fn main() {
    //  build Docker Compose services for 'scripts' and 'tokenbridge'
    let build_command = Command::new("sh")
        .arg("-c")
        .arg("docker-compose build --no-rm scripts tokenbridge")
        .output()
        .expect("Failed to run command");
    println!(
        "{:?}",
        String::from_utf8(build_command.stdout).expect("Invalid UTF-8")
    );
    // build Docker Compose services for 'sequencer', 'poster', 'staker-unsafe', and 'scripts'
    let build_nodes_command = Command::new("sh")
        .arg("-c")
        .arg("docker-compose build --no-rm sequencer poster staker-unsafe scripts")
        .output()
        .expect("Failed to run command");
    println!(
        "{:?}",
        String::from_utf8(build_nodes_command.stdout).expect("Invalid UTF-8")
    );
    //For start fresh services
    //tear down Docker Compose services and remove associated containers
    // let remove_old_data = Command::new("sh")
    //     .arg("-c")
    //     .arg("docker-compose down")
    //     .output()
    //     .expect("Failed to run command");
    // println!(
    //     "{:?}",
    //     String::from_utf8(remove_old_data.stdout).expect("Invalid UTF-8")
    // );
    // //list Docker containers with a specific label in the 'nitro-testnode' project
    // let leftover_containers = Command::new("sh")
    //     .arg("-c")
    //     .arg("docker container ls -a --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo")
    //     .output()
    //     .expect("Failed to run command");
    // // Extract the container IDs as a string
    // let container_ids = String::from_utf8(leftover_containers.stdout).expect("Invalid UTF-8");
    // //remove containers with container_ids
    // let remove_leftover_containers = Command::new("docker")
    //     .arg("rm")
    //     .args(container_ids.trim().split_whitespace()) // Split the IDs into separate arguments
    //     .output()
    //     .expect("Failed to run command");

    // // Print the output of the `docker rm` command
    // println!(
    //     ":::: {:?}",
    //     String::from_utf8(remove_leftover_containers.stdout).expect("Invalid UTF-8")
    // );
    // // prune Docker volumes with a specific label in the 'nitro-testnode' project
    // let prune_volume = Command::new("sh")
    //     .arg("-c")
    //     .arg("docker volume prune -f --filter label=com.docker.compose.project=nitro-testnode")
    //     .output()
    //     .expect("Failed to run command");
    // println!(
    //     "::->{:?}",
    //     String::from_utf8(prune_volume.stdout).expect("Invalid UTF-8")
    // );
    // //list Docker volumes with a specific label in the 'nitro-testnode' project
    // let leftover_volumes = Command::new("sh")
    //     .arg("-c")
    //     .arg("docker volume ls --filter label=com.docker.compose.project=nitro-testnode -q | xargs echo")
    //     .output()
    //     .expect("Failed to run command");
    // // Extract the container IDs as a string
    // let container_vol_ids = String::from_utf8(leftover_volumes.stdout).expect("Invalid UTF-8");
    // //remove Docker volumes by their IDs
    // let remove_leftover_vol_containers = Command::new("docker")
    //     .arg("rm")
    //     .args(container_vol_ids.trim().split_whitespace()) // Split the IDs into separate arguments
    //     .output()
    //     .expect("Failed to run command");

    // // Print the output of the `docker rm` command
    // println!(
    //     "volume:  {:?}",
    //     String::from_utf8(remove_leftover_vol_containers.stdout).expect("Invalid UTF-8")
    // );

    // Generating l1 keys
    println!("== Generating l1 keys");
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
    println!(" == Funding validator and sequencer");
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
    println!(" == create l1 traffic");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts send-l1 --ethamount 1000 --to user_l1user --wait")
        .output()
        .expect("Failed to run command");

    //Writing l2 chain config
    println!(" == Writing l2 chain config");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-l2-chain-config")
        .output()
        .expect("Failed to run command");
    // getting sequencer address for Deploying L2
    println!(" == Deploying L2");
    let _sequenceraddress = Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-l2-chain-config")
        .output()
        .expect("Failed to run command");
    //Deploying L2
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint /usr/local/bin/deploy poster --l1conn ws://geth:8546 --l1keystore /home/user/l1keystore --sequencerAddress $sequenceraddress --ownerAddress $sequenceraddress --l1DeployAccount $sequenceraddress --l1deployment /config/deployment.json --authorizevalidators 10 --wasmrootpath /home/user/target/machines --l1chainid=1337 --l2chainconfig /config/l2_chain_config.json --l2chainname arb-dev-test --l2chaininfo /config/deployed_chain_info.json")
        .output()
        .expect("Failed to run command");
    // execute a Docker Compose command to run the 'poster' service with a custom entrypoint.
    //Inside the service, execute a shell command using 'jq' to process a JSON file, redirect the output to create 'l2_chain_info.json'.
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh poster -c \"jq [.[]] /config/deployed_chain_info.json > /config/l2_chain_info.json\"")
        .output()
        .expect("Failed to run command");

    // execute a Docker Compose command to run the 'scripts' service to perform 'write-config' operation.
    println!("  == Writing configs");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts write-config")
        .output()
        .expect("Failed to run command");

    // Initializing redis
    println!("  == Initializing redis");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run scripts redis-init --redundancy 0")
        .output()
        .expect("Failed to run command");

    // start the 'sequencer' service in detached mode using Docker Compose.
    println!("  == Funding l2 funnel");
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
    println!("  == Deploying token bridge");
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run -e ARB_KEY=\"b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659\" -e ETH_KEY=\"b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659\" tokenbridge gen:network")
        .output()
        .expect("Failed to run command");

    // run the 'tokenbridge' service with a custom entrypoint.
    // Inside the service, execute a shell command to display the content of the 'localNetwork.json' file.
    Command::new("sh")
        .arg("-c")
        .arg("docker-compose run --entrypoint sh tokenbridge -c \"cat localNetwork.json\"")
        .output()
        .expect("Failed to run command");

    let mut command = Command::new("docker-compose");
    command.arg("up");
    command.arg("-d");
    command.arg("sequencer");
    command.arg("poster");
    command.arg("staker-unsafe");
    let output = command
        .output()
        .expect("Failed to run docker-compose command");
    println!("{:?}", output);
    if output.status.success() {
        println!("Docker Compose up successful!");
    } else {
        eprintln!("Docker Compose up failed!");
        eprintln!("Output: {:?}", output.stdout);
        eprintln!("Error: {:?}", output.stderr);
    }
}
