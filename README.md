# Elrond BUSD stablecoin
BUSD stablecoin smart contract Elrond implementation and tests

Current implementation directly translated from https://github.com/paxosglobal/busd-contract from Solidity to elrond-wasm Rust.

# How to build

## First, install erdpy

In short:
```
pip3 install --user --upgrade --no-cache-dir erdpy
```

More details here: https://github.com/ElrondNetwork/erdpy/blob/master/README.md

## Build command

```
erdpy build .
```

## Run tests

All tests:

```
erdpy --verbose test --directory="tests"
```

To debug individual tests (example):
```
erdpy --verbose test --directory="tests/init/create.scen.json"
```

## Deploy

```
erdpy --verbose deploy . --pem="./alice.pem" --proxy="https://wallet-api.elrond.com"
erdpy --verbose query erd1qqqqqqqqqqqqqpgq9sp6f9m9zzhepfl0e02cy9m9tnztry2kx2fsf96449 --function="name" --proxy="https://wallet-api.elrond.com"

```

