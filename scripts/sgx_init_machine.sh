#!/bin/bash

set -euo pipefail

Color_Off='\033[0m'  
Red='\033[0;31m'
Green='\033[0;32m'
Blue='\033[0;34m'
Purple='\033[0;35m'


AZURE_VM_NAME=${1:-my_sgx_machine}
SGX_IMG_NAME=${2:-my_sgx_function}
AZURE_VM_ADMIN_USER="azureuser"
AZURE_VM_LOCATION="uksouth" # az account list-locations -o table
AZURE_VM_SIZE="Standard_DC2s_v3" # https://learn.microsoft.com/en-us/azure/virtual-machines/dv3-dsv3-series
is_homebrew_installed=false
[[ "$(uname)" == "Darwin" && "$(command -v brew)" ]] && is_homebrew_installed=true


echo -e "AZURE_VM_NAME: ${Blue}$AZURE_VM_NAME${Color_Off}"
echo -e "SGX_IMG_NAME: ${Blue}$SGX_IMG_NAME${Color_Off}"

if ! command -v az &> /dev/null; then
    if $is_homebrew_installed; then
        brew update && brew install azure-cli
    else
        sudo apt install azure-cli
    fi
fi

if ! command -v jq &> /dev/null; then
    if $is_homebrew_installed; then
        brew update && brew install jq
    else
        sudo apt install jq
    fi
fi

# Get a Virtual Machine's IP from its name
function get_vm_ip() {
    local vm_name=$1
    if [ -z "$vm_name" ]; then
        echo -e "${Red}VM_NAME was not provided${Color_Off}"
        return 1
    fi

    local vm_ip;
    vm_ip=$(az vm list-ip-addresses --name "$AZURE_VM_NAME" | jq '.[0].virtualMachine.network.publicIpAddresses[0].ipAddress' | tr -d '"')
    if [ -z "$vm_ip" ]; then
        echo -e "${Red}Failed to find IP address for ${Blue}$vm_name${Color_Off}"
        return 1
    fi

    echo "$vm_ip"
}

# Check if a virtual machine with a given name exists
function vm_exists() {
    local vm_name=$1
    if [ -z "$vm_name" ]; then
        echo -e "${Red}VM_NAME was not provided${Color_Off}"
        return 1
    fi

    vm_list=$(az vm list)
    vm_count=$(echo "$vm_list" | jq --arg AZURE_VM_NAME "$AZURE_VM_NAME" '.[] | select(.name == $AZURE_VM_NAME) | .name' | wc -l)
    [ "$vm_count" -gt 0 ] && echo "true" || echo "false"
}

# If needed, create an SSH key and add to the ssh-agent
function setup_ssh_keys() {
    local user=${1:-sgx-ssh-key}
    ## Create SSH Keys
    # Check if the SSH key exists
    if [ ! -f ~/.ssh/id_rsa ]; then
        # Create the SSH key
        ssh-keygen -t rsa -b 4096 -C "$user" -f ~/.ssh/id_rsa
        echo -e "${Purple}SSH key generated at ~/ssh/id_rsa${Color_Off}"

        # Check if ssh-agent is running
        if [ -z "$SSH_AUTH_SOCK" ] || ! ssh-add -l >/dev/null 2>&1; then
            eval "$(ssh-agent -s)"
        fi

        # If the host OS is macOS, start the SSH agent if it's not running and add the key to the Apple Keychain
        if [ "$(uname)" == "Darwin" ]; then
            # Add the SSH key to the Apple Keychain
            ssh-add --apple-use-keychain ~/.ssh/id_rsa
        else
            ssh-add ~/.ssh/id_rsa
        fi
    fi
}

# Create a new Azure virtual machine with a given name
function create_vm() {
    local vm_name=$1
    if [ -z "$vm_name" ]; then
        echo -e "${Red}VM_NAME was not provided${Color_Off}"
        return 1
    fi

    create_vm_response=$(az vm create \
        -n "$vm_name" \
        -g Default \
        --admin-username "$AZURE_VM_ADMIN_USER" \
        --size "$AZURE_VM_SIZE" \
        --generate-ssh-keys \
        --location "$AZURE_VM_LOCATION" \
        --zone 2 \
        --image Canonical:0001-com-ubuntu-minimal-focal:minimal-20_04-lts-gen2:20.04.202303290)
    
    # Attempt to get the VM IP and add to our SSH known hosts
    vm_public_ip=$(echo "$create_vm_response" | jq -r '.publicIpAddress')
    if [ -n "$vm_public_ip" ]; then
        ssh-keyscan -H "$vm_public_ip" >> ~/.ssh/known_hosts
    fi
}

# Install Ubuntu dependencies in an Azure virtual machine
function setup_vm_deps() {
    local vm_ip=$1
    if [ -z "$vm_ip" ]; then
        echo -e "${Red}VM_IP was not provided${Color_Off}"
        return 1
    fi


    # Install docker buildx
    ssh "$AZURE_VM_ADMIN_USER"@"$vm_ip" << "EOF"
        sudo apt-get update -y
        sudo apt-get upgrade -y
        sudo apt-get install -y git vim nano unzip curl ca-certificates gnupg
        sudo install -m 0755 -d /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        sudo chmod a+r /etc/apt/keyrings/docker.gpg
        echo "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
        sudo apt-get update -y
        sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
EOF
    # Start docker on boot and add the azueruser to the docker group
    ssh "$AZURE_VM_ADMIN_USER"@"$vm_ip" "sudo systemctl enable docker.service && sudo systemctl enable containerd.service && sudo usermod -a -G docker $AZURE_VM_ADMIN_USER"
}

# Setup the SGX measurement within an Azure virtual machine
function setup_vm_measurement() {
    local vm_ip=$1
    if [ -z "$vm_ip" ]; then
        echo -e "${Red}VM_IP was not provided${Color_Off}"
        return 1
    fi

    local img_name=$2
    if [ -z "$img_name" ]; then
        echo -e "${Red}SGX_IMG_NAME was not provided${Color_Off}"
        return 1
    fi

   ssh "$AZURE_VM_ADMIN_USER"@"$vm_ip" << EOF
        printf '\n\nStarting docker service ...\n'
        sudo systemctl start docker
        sudo systemctl status docker
        docker ps
        if [ ! -d sbv3-example1 ]; then
            git clone https://github.com/switchboard-xyz/sbv3-example1
            cd sbv3-example1
        else
            cd sbv3-example1
            git pull origin
        fi
        printf '\n\nRunning sbv3-example1 build script ...\n'
        bash build.sh "$img_name"
        printf '\n\nStarting docker container ...\n'
        docker run -it -d --rm $img_name
EOF
}

############################################################
## Login to Azure and verify the SSH keys are setup
############################################################
az_account=$(az account show -o json)
az_user=$(echo "$az_account" | jq -r '.user.name')
az_account_state=$(echo "$az_account" | jq -r '.state')
if [[ $az_account_state != "Enabled" ]]; then
    az login
fi

setup_ssh_keys "$az_user"

############################################################
## Get or create the Azure VM
############################################################
if [ "$(vm_exists "$AZURE_VM_NAME")" == "true" ]; then
    echo "Found an existing Azure VM ($AZURE_VM_NAME) ..."
else
    echo "Creating a new Azure VM ($AZURE_VM_NAME) ..."
    create_vm "$AZURE_VM_NAME" "$SGX_IMG_NAME"
fi

############################################################
## Setup the Azure VM
############################################################
AZURE_VM_IP=$(get_vm_ip "$AZURE_VM_NAME")
setup_vm_deps "$AZURE_VM_IP"

# Run the setup_vm_measurement and grep for the MR_ENCLAVE measurement
mr_enclave=$(setup_vm_measurement "$AZURE_VM_IP" "$SGX_IMG_NAME" 2>&1 | tee /dev/tty | grep "MR_ENCLAVE:" | cut -d ' ' -f 2)

if [ -z "$mr_enclave" ]; then
    echo -e "${Red}✗ Failed to get the measurement from the virtual machine${Color_Off}"
    echo "Try running the script again to rebuild the dependencies"
    printf '\n\t%s %s\n' "$0" "$*"
    exit 1
fi

echo -e "${Green}✓ Finished initializing the SGX Virtual Machine${Color_Off}"
separator="----------------------"
printf '%s\nVM_NAME: %s\nIP: %s\nLocation: %s\nImage: %s\nMR_ENCLAVE: %s\n%s\n\n' "$separator" "$AZURE_VM_NAME" "$AZURE_VM_IP" "$AZURE_VM_LOCATION" "$SGX_IMG_NAME" "$mr_enclave" "$separator"
echo Run the following command to ssh into the container:
printf '\n\tssh %s@%s\n' "$AZURE_VM_ADMIN_USER" "$AZURE_VM_IP"