export NODE="--node $RPC"
export TXFLAG="${NODE} --chain-id ${CHAIN_ID} --gas-prices 0.25${FEE_DENOM} --gas auto --gas-adjustment 1.3"

# zsh
export NODE="--node $RPC"
export TXFLAG="$NODE --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3"
